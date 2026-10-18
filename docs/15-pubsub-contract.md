# Pub/sub transport contract (draft)

Goal: Zenoh, MQTT (3.1.1 and 5), NATS, DDS or any other pub/sub system can be the transport. To make that possible, weft asks very little of the transport and builds the rest itself, end to end.

**Main rule: transport guarantees are hop by hop, and weft guarantees are end to end.** Delivery across a relay, an uplink and a bridge to another transport only holds if weft provides it itself. "QoS 1 on MQTT" says nothing once a message has crossed into Zenoh and back. weft therefore *never relies* on transport QoS for correctness. It only uses it to be efficient.

## 1. What every transport must provide

A transport is supported when it provides these, natively or through a thin adapter:

| # | Requirement | Notes |
|---|---|---|
| T1 | **Publish to a hierarchical key** and **subscribe by prefix**, plus a single-level wildcard | MQTT `+` / `#`, Zenoh `*` / `**`, NATS `*` / `>`, DDS topics plus content filter |
| T2 | **Opaque binary payload of at least 4 KiB**, preferably 64 KiB | Large data never goes through pub/sub as one message (see §4) |
| T3 | **Best-effort delivery** (at most once) | That's all. Loss, duplicates and reordering must be tolerated by weft |
| T4 | **Local connection state** (link up or down) | Drives store and forward and resync |
| T5 | **Authenticated peers or clients**, per node identity | TLS/mTLS, Zenoh TLS/QUIC, or the overlay. Used for transport ACLs, not for trust |
| T6 | **ACLs by key prefix** per identity, if the transport has a broker or router | Generated from the weft policy (docs/06). Zenoh ACL, MQTT broker ACL, NATS permissions |

Everything else is optional.

## 2. Optional capabilities: used when present, emulated otherwise

The adapter advertises which of these it has. weft uses the native feature where it exists and otherwise falls back to its own mechanism, with the same behaviour for apps.

| Capability | Zenoh | MQTT 3.1.1 | MQTT 5 | NATS core | NATS JetStream | DDS | weft fallback |
|---|---|---|---|---|---|---|---|
| Message attributes (headers) | Attachment | ✕ | User properties | Headers | Headers | ✕ (in payload) | Envelope inside the payload (structured mode, §3) |
| Acknowledged delivery | Reliable channel per hop | QoS 1/2 per hop | QoS 1/2 per hop | ✕ | Ack | Reliable QoS | Sync protocol (§3.2) and acks for events |
| Last value per key ("retained") | Storage / publication cache | Retained | Retained | ✕ | KV bucket | Durability TRANSIENT_LOCAL | Last-value store in site services, queryable |
| Request/response | `get` / queryable | ✕ (topic pairs) | Response topic + correlation data | Request/reply | Request/reply | RPC over DDS | Correlation id + reply topic in the envelope |
| Persistence while a subscriber is offline | Storage plugin | Persistent session | Session expiry | ✕ | Stream | Durability / history | Store and forward in the uplink (§3.3) |
| Liveliness / presence | Liveliness tokens | Last will | Last will + delay | ✕ | ✕ | Liveliness QoS | Heartbeat events plus timeout |
| Priorities | 7 levels | ✕ | ✕ | ✕ | ✕ | Transport priority | Local queues per traffic class |
| Congestion control | Drop or block per publisher | ✕ | Receive maximum | Slow consumer drop | Flow control | History depth | Drop oldest telemetry first |
| Message expiry | ✕ (in attachment) | ✕ | Message expiry interval | ✕ | Max age | Lifespan QoS | Expiry in the envelope, checked by the receiver |
| Load-balanced consumers | Queryable selection | ✕ | Shared subscriptions | Queue groups | Consumers | ✕ | Backend concern (ingestion) |
| Peer-to-peer without a broker | Yes (peer mode) | ✕ | ✕ | ✕ | ✕ | Yes | The broker becomes the uplink / site services role |
| Runs on MCUs | zenoh-pico | Many clients | Some | nats.c (big) | ✕ | micro-ROS / XRCE | Proxy |

Values as of 2026-10. Zenoh's "advanced" publisher/subscriber (history, sample-miss detection) is also used where available.

## 3. What weft builds on top (the end-to-end guarantees)

### 3.1 Traffic classes

Every message belongs to a class, given by its topic segment (docs/05 §1.2). The class decides delivery, not the transport.

| Class | Topics | Delivery | Duplicates and order | Under congestion | Offline |
|---|---|---|---|---|---|
| **Telemetry** | `tel/` | Best effort, latest value matters | Drop old values, order by sample time | Drop or downsample first | Buffer with a size cap, downsample, oldest dropped |
| **Events and alarms** | `evt/` | At least once, end to end (acked by the receiving ingestion) | Dedupe by `id` | Keep, higher priority | Store and forward, persistent |
| **Commands** | `cmd/` | Request/response with expiry | Executed at most once (idempotency by `id`) | High priority | **Not queued** past their expiry: a stale command is dropped |
| **Documents** (desired state, delegations, revocations) | `state/desired`, `desc` | Eventually delivered, through the sync protocol (§3.2) | Idempotent by digest | Small, high priority | Pull on reconnect |
| **Reported state** | `state/reported` | Last value per device and component | Newest version wins | Keep latest only | Latest kept, sent on reconnect |
| **Logs and traces** | OpenTelemetry export | Best effort, buffered | – | Lowest priority | Ring buffer |
| **Bulk** (artifacts, crash dumps, files) | Not pub/sub | OCI/HTTP with range requests, or chunked (§4) | By digest | Background | Resumable |

### 3.2 Document sync

Documents are what keeps a fleet consistent, so they must not depend on retained messages or sessions.

1. Push: new documents are published as they appear (fast path, may be lost).
2. Pull, as anti-entropy:
   - On connect and periodically, a node publishes a **digest summary** of the documents it has for a tenant and selector. A hash of the set comes first, the list only when the hashes differ.
   - A peer, uplink or backend that has missing documents sends them through request/response.
3. Documents are content-addressed and verified on arrival, so it doesn't matter who sends them: any peer, an uplink, a USB stick. The verifier ignores duplicates.

This works over every transport that meets T1–T4, including across bridges.

### 3.3 Store and forward

- Each uplink, and each endpoint with a direct upstream, keeps an outbox per traffic class on persistent storage, with the limits from the class table.
- Events are deleted only after an end-to-end ack, from ingestion or the receiving site service. Transport acks are not enough.
- Replay in order of `weftseq` per source. Receivers detect gaps by sequence and ask for missing ranges if needed.

### 3.4 Envelope rules that make transports interchangeable

- **Attributes travel in binary mode** (transport headers) when the transport has headers. Otherwise they travel in **structured mode**: a CBOR array `[attributes, payload]` as the message payload. A bridge converts between the two without changing the signature, because the signature covers payload plus attributes, not the framing.
- **Dedupe** by (`source`, `id`). **Gap detection** by `weftseq`. **Expiry** by `weftexpires`: issued_at plus a duration, checked against the receiver's time floor. Commands with no reliable time use the time floor (docs/11).
- **Topic charset** valid on every transport: segments are `[a-z0-9_-]{1,64}`, separated by `/`.
  - No `+ # * > . $ ? @` inside segments, because each of those is special in at least one of MQTT, Zenoh or NATS.
  - Maximum depth 16 and total length 256 bytes.
  - The adapter maps `/` to `.` for NATS.
- **Liveliness topic**: `weft/{tenant}/{device}/live` instead of `@weft/...`, since `@` is reserved in Zenoh key expressions. Zenoh liveliness tokens and MQTT last will are mapped onto it.

## 4. Size limits and large data

- Minimum payload size a transport must accept: 4 KiB. weft messages other than bulk stay under 4 KiB by default, and telemetry batches stay under 4 KiB as well (this also keeps AWS IoT billing at one message per 5 KiB step).
- Bulk data is fetched by digest over HTTP/OCI when IP is available. Otherwise it is sent as chunked transfer over pub/sub (fixed-size chunks, each with digest and index, resumable). Chunking covers serial, BLE and LoRa-class links through a proxy.

## 5. The adapter interface (sketch, part of G10)

```rust
trait Transport {
    fn capabilities(&self) -> Capabilities;                         // §2 table, as flags
    fn publish(&self, key: &Key, attrs: &Attrs, payload: &[u8], class: Class) -> Result<()>;
    fn subscribe(&self, prefix: &KeyPattern) -> Result<Subscription>;
    fn request(&self, key: &Key, attrs: &Attrs, payload: &[u8], timeout: Duration) -> Result<Replies>;
    fn serve(&self, prefix: &KeyPattern) -> Result<RequestStream>;   // queryable / responder
    fn link_state(&self) -> LinkStateStream;                        // T4
}
```

Everything in §3 (classes, sync, outbox, dedupe, chunking) sits *above* this trait and is written once. An adapter for a new transport is then mainly key mapping and attribute framing: a few hundred lines.

Rules for the interface:

- It is outside the trusted code. Anything coming from a transport is untrusted input to the verifier.
- `request` and `serve` are emulated with correlated topics on transports without request/response.

## 6. Bridges between transports

- A bridge is just a node with two adapters: it subscribes on one transport and publishes on the other.
- It keeps the envelope and signatures unchanged and only converts the framing (binary or structured).
- Because the guarantees are end to end (§3), bridging does not weaken them.
- Existing bridges (zenoh-plugin-mqtt, zenoh-bridge-mqtt, NATS MQTT support) can be used if they pass the attributes through. That needs MQTT 5 user properties. With MQTT 3.1.1 only structured mode works.

## 7. Conformance tests (part of G17)

Every adapter passes the same suite:

- key mapping and the charset;
- framing round trip;
- loss, duplicates and reordering injected (weft must still converge);
- document sync after a partition;
- outbox replay;
- request/response timeout;
- a bridge to and from the reference transport (Zenoh).

## 8. Open points

- Choosing the sync summary structure: hash of a sorted digest list vs. Merkle tree vs. range-based set reconciliation, depending on document counts.
- End-to-end acks for events across several uplinks: who acknowledges, and when (the ingestion service or the first site service).
- Encryption of payloads to groups across untrusted relays (docs/06).
- Telemetry signing: per batch hash chain (docs/05 §1) interplay with downsampling at uplinks (a downsampled batch is a new batch, signed by the uplink).
