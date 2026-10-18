# Transport protocols: TLS, QUIC, HTTP, WebSockets (draft)

**Starting point: weft's integrity does not depend on the transport.** Documents and commands are signed end to end (COSE), so TLS, QUIC or a tunnel is *not* what makes a message trustworthy. Transport security still matters, for:

- **confidentiality**;
- **access control and DoS protection**: only known nodes may connect;
- **getting through real networks**: firewalls, proxies, NAT, cellular.

That makes most choices here operational, and keeps all of it outside the trusted code. The verifier never sees TLS.

## 1. Rules

1. **TLS 1.3 only** for weft components. TLS 1.2 is allowed only where an existing broker or registry requires it, through an adapter.
2. **Mutual authentication with the device key.**
   - Prefer **raw public keys** (RFC 7250) or pinned keys, matching the weft key id model. X.509 is supported for existing PKIs and brokers.
   - The key lives in the TPM or secure element where possible.
3. **Certificate expiry must not lock out a device with a bad clock.** X.509 validity checks need wall time, and offline devices often don't have it (docs/11).
   - With raw or pinned keys, revocation comes from weft documents, not from a validity period.
   - With X.509, the check uses the device's *time floor*. A device whose clock is behind may still connect and repair its time. It never trusts a certificate whose *not_after* is below its time floor.
4. **Post-quantum key exchange by default where the stack supports it:** hybrid X25519 + ML-KEM-768 (`X25519MLKEM768`) in rustls. QUIC uses the TLS 1.3 handshake, so it gets the same protection.
   - **Cost:** ML-KEM adds about 1 KB to each side of the handshake. That's fine on Linux, but a concern on MCUs and on narrow links (handshake fragmentation, LoRa/NB-IoT).
   - **MCUs** therefore keep classic ECDHE, or rely on the overlay (Rosenpass) or on application-level encryption, until a PQ handshake fits.
5. **Inside the overlay**, transport TLS may be switched off per conduit, because the overlay already encrypts and authenticates. This saves CPU on small devices. It is explicit, signed config, and the default is TLS on (defence in depth).
6. **TLS inspection proxies** in corporate networks break mutual TLS. The fallback is server-only TLS through the proxy plus weft signatures. Integrity still holds, but confidentiality towards the proxy owner is lost. That must be visible in the device's reported state.

## 2. Getting through networks

What OT and corporate networks typically allow, from easiest to hardest. This is our judgment from common practice:

| Network allows | Works | Doesn't work |
|---|---|---|
| Anything outbound | Everything | – |
| Outbound UDP and TCP, NAT | QUIC, WireGuard/Nebula (with hole punching or relays), TCP+TLS | Inbound connections (use relays or uplinks) |
| **Outbound TCP 443 only** (very common) | TLS on 443, WebSocket over TLS, HTTP/2 | QUIC/HTTP/3, WireGuard, Nebula (all UDP) |
| **HTTP proxy only** (CONNECT, maybe authentication) | TLS or WebSocket through CONNECT | Anything not going through the proxy |
| TLS-inspecting proxy | Server-side TLS, WebSocket (if the proxy allows upgrades) | Mutual TLS, pinned server keys |
| Nothing outbound (air gap) | Local network, USB/laptop bundles | Upstream |

Consequences:

- **Every upstream connection needs a TCP 443 fallback.** The default order is:
  1. QUIC (UDP 443);
  2. TLS over TCP (443);
  3. WebSocket over TLS (443);
  4. the same through an HTTP CONNECT proxy.

  Which ones are tried and in what order is per-site config.
- **One port, several protocols:** the upstream side multiplexes on TCP/UDP 443 using ALPN, so firewall rules stay simple:
  - `h2`/`http/1.1` for OCI and HTTP;
  - a Zenoh or MQTT ALPN for pub/sub;
  - WebSocket upgrade.
- **The overlay also needs this fallback.** Nebula and WireGuard are UDP only. Behind TCP-only firewalls, the overlay needs a TCP/WebSocket relay (for example wstunnel, or an uplink with a TCP relay), or it is simply not used there and pub/sub runs over TLS on 443 directly.

## 3. QUIC vs TLS over TCP

| | QUIC (HTTP/3, Zenoh QUIC) | TLS over TCP |
|---|---|---|
| Connection migration (IP changes) | **Yes**: cellular handover, Wi-Fi roaming, failover between links | No, so reconnect and TLS handshake again |
| Several streams without head-of-line blocking | **Yes**: bulk and control share one connection without blocking each other | No, a lost packet stalls everything |
| Handshake | 1-RTT, 0-RTT resumption (0-RTT only for idempotent requests, replay risk) | TCP + TLS: 2 RTT (1 with TLS 1.3 plus TCP fast open) |
| Lossy or long links (satellite, cellular) | Better loss recovery, configurable congestion control | Worse, but tunable |
| Blocked by firewalls | **Often** (UDP) | Rarely |
| NAT keepalive cost | Higher: UDP mappings time out sooner (often 30 s–2 min) | Lower |
| CPU and memory | Higher (user-space stack) | Lower, kernel TCP |
| MCUs | Rare | Common (mbedTLS, embedded-tls, wolfSSL) |
| Rust stacks | quinn (MIT/Apache-2.0), s2n-quic (Apache-2.0) | rustls (Apache-2.0/MIT/ISC) |

**Decision proposal:**

- **QUIC is preferred** where UDP is allowed, for moving nodes (robots, vehicles, ships with link switching) and for uplinks with multiple links.
- **TLS over TCP is the guaranteed fallback** and the default for fixed, simple sites.
- **MCUs use TCP+TLS, DTLS or no transport security** behind a proxy.
- Choices are per link, not global. Zenoh supports TCP, TLS, QUIC, WebSocket, UDP and serial, so this is configuration.

## 4. HTTP versions (artifacts and APIs)

- **Artifacts (OCI):**
  - HTTP/1.1 and HTTP/2 are required, since that's what all registries speak;
  - range requests are mandatory for resumable downloads;
  - HTTP/3 is optional (CDNs offer it, and it helps on lossy links);
  - artifacts are verified by digest, so caches, mirrors and CDNs need no trust.
- **APIs (backend, local UI):** HTTP/2 or 1.1 with TLS 1.3. gRPC only at the edges where a product needs it. The weft-native API is CBOR over HTTP plus pub/sub.
- **Local network:** plain HTTP is allowed only for the local UI on a link-local interface, and only behind an explicit conduit. The default is TLS with a device-local certificate (a trust-on-first-use or QR-code key fingerprint).

## 5. WebSockets and WebTransport

- **WebSocket** is needed for two things:
  - **browsers**: the web console and local UI subscribe to live data. MQTT over WebSocket and Zenoh WebSocket transports exist, as does NATS WebSocket;
  - **getting through proxies and firewalls** (§2).
- **WebTransport** (over HTTP/3) is the successor for browsers: several streams, datagrams. Optional and later. Same UDP caveat as QUIC.
- Browsers cannot do raw TCP, UDP or mutual TLS with device keys easily. A browser is therefore a client of an **uplink or site services node**, never a full weft node. It authenticates as a user, and its actions are signed by user keys (WebAuthn/passkeys, later).

## 6. Constrained links and MCUs

| Link | Suggested stack |
|---|---|
| Ethernet/Wi-Fi MCU | zenoh-pico or MQTT over TCP + TLS (mbedTLS / embedded-tls), or over the overlay through a proxy |
| NB-IoT / LTE-M | UDP: CoAP + DTLS 1.3 or OSCORE (+ EDHOC for keys), LwM2M where an operator platform requires it; MQTT-SN as an alternative |
| BLE, Thread, CAN, serial | No IP security; the link to the proxy is local. Payloads signed (COSE) end to end; encryption with COSE_Encrypt if needed |
| LoRaWAN | LoRaWAN's own AES keys up to the network server; weft starts at the LNS (proxy) |

OSCORE is attractive because it protects the *message* (COSE-based, like weft) rather than the connection. It therefore survives proxies and works across store and forward. This is a candidate to evaluate for MCU commands.

## 7. Keepalives and cellular cost

- Keepalive and idle timeout are per link class (docs/12, docs/15).
- Cellular NAT timeouts force keepalives, and every keepalive costs battery and data. Defaults: 60–120 s TCP keepalive on metered links. On sleepy devices, no persistent connection at all: they connect on wake and pull (G12).
- QUIC idle timeouts must sit below the carrier's NAT UDP timeout, otherwise every packet after idle re-establishes the session.

## 8. Libraries (all permissive)

| Purpose | Library | Licence |
|---|---|---|
| TLS | rustls with the aws-lc-rs provider (PQ hybrids) or ring | Apache-2.0 / MIT / ISC (aws-lc-rs: Apache-2.0 AND ISC; to check with cargo-deny) |
| QUIC | quinn | MIT / Apache-2.0 |
| HTTP | hyper, reqwest | MIT |
| WebSocket | tokio-tungstenite | MIT |
| MCU TLS | embedded-tls (Rust), mbedTLS (Apache-2.0) | |
| CoAP/OSCORE | to evaluate (libcoap is BSD-2; Rust crates immature) | |

None of these is in the verifier. All live in transport adapters (docs/15) and the agent's network process (docs/07, privilege separation).

## 9. Open points

- Default fallback order and timeouts per site; detecting TLS inspection reliably.
- Overlay behind TCP-only firewalls: build a TCP relay or recommend wstunnel / Nebula alternatives.
- PQ handshake on MCUs: measure the ML-KEM cost with embedded-tls / mbedTLS 4.
- OSCORE/EDHOC evaluation for MCU commands.
- Browser user keys (passkeys) for signing documents from the web console.
