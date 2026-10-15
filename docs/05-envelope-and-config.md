# Message envelope and configuration (draft design)

## 1. Three kinds of traffic, three envelope weights

One heavy envelope for everything would kill telemetry on small devices; no envelope would kill
interoperability and trust. So the envelope scales with the kind of traffic:

| Kind | Examples | Rate | Envelope | Signed |
|---|---|---|---|---|
| **Telemetry** | Sensor samples, metrics | High | None per message: the topic carries source and type, the description (§1.3) carries schema and units | No per message; optional periodic batch signature (hash chain / Merkle root) |
| **Messages** | Events, commands, replies, alarms | Low–medium | Full envelope (§1.1) | Optional, required for commands |
| **Documents** | Desired state, config fragments, descriptions, rollout plans | Rare | Signed document (§2.2) | Always |

### 1.1 Envelope attributes (CloudEvents-compatible)

| Attribute | Short key | Req. | Meaning |
|---|---|---|---|
| `specversion` | 0 | yes | CloudEvents version |
| `id` | 1 | yes | Unique per source (dedup) |
| `source` | 2 | yes | `fl://{tenant}/{device}/{component}` |
| `type` | 3 | yes | e.g. `io.fleetloom.ota.status.v1` |
| `time` | 4 | no | Hybrid logical clock timestamp (HLC), not just wall clock |
| `subject` | 5 | no | What it is about (sensor, app, job) |
| `datacontenttype` | 6 | no | Default `application/cbor` |
| `dataschema` | 7 | no | Schema or semantic reference (OPC UA type, AAS semantic ID, IPSO id) |
| `flcorrelation` | 8 | no | Correlation ID for request/reply |
| `flreplyto` | 9 | no | Where replies go (if transport has no native request/reply) |
| `flseq` | 10 | no | Per-source sequence number (gap detection, not strict ordering) |
| `flsig` | 11 | no | Detached COSE_Sign1 signature over payload + attributes |

Encoding: **binary content mode** everywhere, payload untouched.

| Transport | Attributes travel in |
|---|---|
| Zenoh | Attachment, as a CBOR map with the short integer keys |
| MQTT 5 | User properties (CloudEvents MQTT binding names) |
| Kafka | Headers (CloudEvents Kafka binding) |
| HTTP | `ce-*` headers |
| MCU links | CBOR map with short keys (same as Zenoh) |

Signatures use **COSE** (RFC 9052): CBOR-based, small, runs on MCUs, algorithm-agile (EdDSA today,
ML-DSA / hybrid as they are registered).

### 1.2 Topic namespace

```
fl/{tenant}/{device}/tel/{path...}            telemetry          e.g. fl/acme/dev-042/tel/pump/temperature
fl/{tenant}/{device}/evt/{type...}            events
fl/{tenant}/{device}/cmd/{component}/{name}   commands (Zenoh query/queryable, MQTT 5 response-topic)
fl/{tenant}/{device}/state/desired            desired-state documents (pushed; also pullable by hash)
fl/{tenant}/{device}/state/reported           reported state
fl/{tenant}/{device}/desc                     self-description (published on connect, queryable)
fl/{tenant}/group/{group}/state/desired       documents for many devices at once
@fl/{tenant}/{device}                         liveliness (Zenoh liveliness token; LWT on MQTT)
```

Location hierarchies (ISA-95 / Unified Namespace paths) and Sparkplug/OPC UA topics are
**projections** produced by adapters, never the primary name.

### 1.3 Self-description

On connect, each device publishes a description (and serves it as a query): components, telemetry
paths with type, unit (UCUM) and semantic reference, commands with argument/result schemas, events,
plugin capabilities. This generalizes Sparkplug's birth message and can be exported as AsyncAPI or a
W3C WoT Thing Description.

## 2. Configuration

### 2.1 Model

- Configuration is part of the desired state. Each **component** (app, OS service, agent plugin) owns
  a **config unit** with a **JSON Schema**, shipped in its package.
- Config is written as **fragments**: small signed documents from one **scope**, targeting one
  component, carrying partial values.
- Scopes, lowest to highest priority: `fleet` → `group` (label selector) → `site` → `device` → `local`
  (technician on site). `local` is not special: it is just another delegated key with limited rights.

### 2.2 A fragment

```yaml
kind: ConfigFragment
apiVersion: fleetloom.io/v1
scope: site
issuer: acme/site-vienna          # key delegated by the tenant root, limited to this site
selector: { labels: { line: line-3 } }
component: app/pump-monitor
version: 17                       # monotonic per (scope, issuer, component)
values:
  sampling_ms: 500
  alarm:
    threshold_c: 85
locks: [ alarm.threshold_c ]      # see 2.3: no other scope may change locked fields
secrets:
  db_password: { ref: "sealed:...", recipient: device-key }   # encrypted to the device (HPKE)
validFrom: 2026-10-20T00:00:00Z   # optional
# + COSE signature over the canonical CBOR encoding
```

### 2.3 Resolution on the device (deterministic, no coordination)

1. Collect all fragments for the component whose signature, delegation and selector are valid for this device.
2. For each (scope, issuer, component), keep only the **highest version** (rollback protection).
3. Merge in scope order with **JSON Merge Patch** semantics (RFC 7396): objects merge per field, arrays and
   scalars are replaced, `null` deletes.
4. **Locks**: a field locked by a scope cannot be changed by any other scope, unless that scope's
   delegation explicitly allows overriding locks (e.g. tenant root).
5. Validate the result against the component's JSON Schema.

Every node with the same fragments computes the same effective config, so this works peer-to-peer
and offline. A top-level signed **desired-state snapshot** lists the expected fragment versions and
artifact hashes per device or group, preventing mix-and-match of old and new fragments (TUF snapshot idea).

### 2.4 Apply, verify, roll back

Config changes are treated like small updates:

1. Render the effective config and hand it to the component through its plugin
   (file under `/etc/fleetloom/...`, environment, or a local API call).
2. Reload or restart per the component's declared policy.
3. Run the component's health check.
4. On failure or schema error: keep or restore the **last-known-good** config and report the failure.

### 2.5 Reported state

The device reports per component: hash of the effective config, applied fragment versions, and
**per-field provenance** ("`alarm.threshold_c` = 85 from site-vienna v17"), plus conflicts
("device fragment tried to change a locked field"). Changes made outside fleetloom are reported as
**drift**; policy per component decides: revert, or adopt as a `local` fragment.

### 2.6 Authoring

Same documents from every entry point:
- **GitOps**: fragments as YAML in Git, CI validates against schemas and signs.
- **API / UI**: backend produces and signs fragments with the operator's delegated key.
- **Local UI / CLI** on the device or a service laptop: produces `local` fragments with a technician key.
- **USB bundle**: fragments + snapshot + artifacts in an OCI image layout.

## Open points

- Exact canonical encoding for signing: deterministic CBOR (RFC 8949 §4.2) vs. JCS for JSON (RFC 8785).
- Whether to reuse TUF metadata directly for snapshots/delegations or define our own on COSE.
- Batch signing scheme for telemetry.
- Schema evolution rules for config units across component versions (migrations).
