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
| `source` | 2 | yes | `weft://{tenant}/{device}/{component}` |
| `type` | 3 | yes | e.g. `org.theweft.ota.status.v1` |
| `time` | 4 | no | Hybrid logical clock timestamp (HLC), not just wall clock |
| `subject` | 5 | no | What it is about (sensor, app, job) |
| `datacontenttype` | 6 | no | Default `application/cbor` |
| `dataschema` | 7 | no | Schema or semantic reference (OPC UA type, AAS semantic ID, IPSO id) |
| `weftcorrelation` | 8 | no | Correlation ID for request/reply |
| `weftreplyto` | 9 | no | Where replies go (if transport has no native request/reply) |
| `weftseq` | 10 | no | Per-source sequence number (gap detection, not strict ordering) |
| `weftsig` | 11 | no | Detached COSE_Sign1 signature over payload + attributes |
| `weftexpires` | 12 | no | Expiry (issued_at + duration); required for commands. See [15-pubsub-contract.md](15-pubsub-contract.md) |

Encoding: **binary content mode** where the transport has headers, payload untouched. Without headers (MQTT 3.1.1, DDS, raw links): **structured mode**, a CBOR array `[attributes, payload]`. See [15-pubsub-contract.md](15-pubsub-contract.md) §3.4.

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
weft/{tenant}/{device}/tel/{path...}            telemetry          e.g. weft/acme/dev-042/tel/pump/temperature
weft/{tenant}/{device}/evt/{type...}            events
weft/{tenant}/{device}/cmd/{component}/{name}   commands (Zenoh query/queryable, MQTT 5 response-topic)
weft/{tenant}/{device}/state/desired            desired-state documents (pushed; also pullable by hash)
weft/{tenant}/{device}/state/reported           reported state
weft/{tenant}/{device}/desc                     self-description (published on connect, queryable)
weft/{tenant}/group/{group}/state/desired       documents for many devices at once
weft/{tenant}/{device}/live                     liveliness (Zenoh liveliness token; LWT on MQTT); `@` is reserved in Zenoh
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
apiVersion: theweft.org/v1
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

Implemented in `weft-verifier` (`resolve` module). Priorities are numbers; level names come from
the tenant's signed **scope profile** (default: fleet 10, group 20, site 30, device 40, local 50),
which authoring tools and backends use. The verifier never sees names.

1. Collect all fragments whose signature, delegation and selector are valid for this device.
2. Per (priority, issuer, component) keep only the **highest version** (rollback protection).
3. Apply priorities in ascending order with **JSON Merge Patch** semantics (RFC 7396).
4. **Ties**: different issuers at the same priority with incompatible changes to the same path (or a
   path and its descendant) → *neither* applies, the lower-priority value stays, a `Tie` conflict
   is reported. Identical changes are not a conflict.
5. **Locks**: owned by the issuers of the lowest-priority fragments that lock a path. Others cannot
   change it at any priority unless their rights include `locks.override`; attempts are reported as
   `Locked` conflicts.
6. Deployments: highest priority wins; different artifacts at that priority → `DeploymentTie`,
   nothing installed, current version keeps running.
7. Validate the result against the component's JSON Schema (plugin, outside the verifier).

Cross-cutting dimensions (e.g. device model vs. site) are decided by the tenant's profile: give one a
higher number, or the same number to force conflicts instead of silent overrides.

Precedent: Windows Group Policy (layers + "Enforced" + explicit link order), Puppet Hiera
(user-defined hierarchy), CSS cascade layers and `!important`, Kubernetes Server-Side Apply
(field ownership, conflicts are errors), XACML "only-one-applicable", CRDT multi-value registers.

### 2.3b Surfacing conflicts

- **Verifier**: returns conflicts with a stable `id` (same conflict on many devices = same id),
  kind, component, path, priority, issuers, document digests and value *hashes* (values may be secret).
- **Device**: reported state (level-based, current set of conflicts), conflict appeared/resolved
  events (edge-based), audit log, local UI/CLI (works offline).
- **Fleet**: read model aggregates by `id`, notifies both issuers, shows both fragments and the ways
  out (withdraw, decide from higher priority, lock); OpenTelemetry metric for open conflicts.
- **Before rollout**: authoring tools and GitOps CI run the same verifier against representative
  devices; new conflicts fail the review.

### 2.4 Apply, verify, roll back

Config changes are treated like small updates:

1. Render the effective config and hand it to the component through its plugin
   (file under `/etc/weft/...`, environment, or a local API call).
2. Reload or restart per the component's declared policy.
3. Run the component's health check.
4. On failure or schema error: keep or restore the **last-known-good** config and report the failure.

### 2.5 Reported state

The device reports per component: hash of the effective config, applied fragment versions, and
**per-field provenance** ("`alarm.threshold_c` = 85 from site-vienna v17"), plus conflicts
("device fragment tried to change a locked field"). Changes made outside weft are reported as
**drift**; policy per component decides: revert, or adopt as a `local` fragment.

### 2.6 Authoring

Same documents from every entry point:
- **GitOps**: fragments as YAML in Git, CI validates against schemas and signs.
- **API / UI**: backend produces and signs fragments with the operator's delegated key.
- **Local UI / CLI** on the device or a service laptop: produces `local` fragments with a technician key.
- **USB bundle**: fragments + snapshot + artifacts in an OCI image layout.

## Open points

- ~~Canonical encoding for signing~~ Decided: **deterministic CBOR** (RFC 8949 §4.2). Schemas for all
  documents and envelopes in **CDDL** (RFC 8610); human-readable form in CBOR diagnostic notation;
  YAML/JSON only in authoring tools, converted before signing. Rust: `minicbor` (no_std, derive) in the
  verifier, own minimal COSE_Sign1 on minicbor, a CDDL validator (e.g. the `cddl` crate) only in CI/tests.
- Whether to reuse TUF metadata directly for snapshots/delegations or define our own on COSE.
- Batch signing scheme for telemetry.
- Schema evolution rules for config units across component versions (migrations).
