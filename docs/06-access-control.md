# Access control (draft design)

## Principle: one policy model, many enforcement points

Policy is **signed data** like config (scoped, versioned, delegated). The agent and backend compile it
into whatever each enforcement point understands. Default is **deny**.

| Question | Enforcement point | Mechanism |
|---|---|---|
| Who may change desired state / config / rollouts? | Every device, offline | Signed documents + scoped delegations (see 05) |
| Who may publish / subscribe / query which topics? | Zenoh routers and peers, MQTT bridge | Generated Zenoh ACL config, broker ACLs |
| Who may invoke which command? | The target device's agent (end-to-end) | Signed command + capability token, checked on the device |
| Who may reach which device/port over IP? | Overlay network | Generated Nebula groups/firewall or WireGuard AllowedIPs + nftables |
| What may a local app do on its device? | Agent's local API / local Zenoh session | Permissions declared in the app manifest, granted in desired state |
| What may a human do in the backend/UI? | Backend | OIDC login (Keycloak) + relationship-based authorization (OpenFGA/SpiceDB) |

Two layers on purpose:
- **Transport ACLs** (Zenoh, broker, overlay) are coarse and protect the network and resources.
- **End-to-end checks** (signatures + capabilities verified by the receiver) are fine-grained and still
  hold when traffic crosses untrusted peers or relays.

## Model

- **Principals**: devices, apps (on a device), users, services, keys. Each has an identity (certificate /
  key) and **attributes** (tenant, site, labels, role).
- **Resources**: topic key expressions, commands, components, config fields, artifacts, network ports.
- **Actions**: `publish`, `subscribe`, `query`, `reply`, `invoke`, `config.write`, `deploy`, `connect`, ...
- **Rules** are attribute-based, e.g.
  *"principal.role == technician && principal.site == resource.device.site may invoke `cmd/pump/*`"*.

## Safe defaults that need no policy at all

- **Identity-bound namespace**: a device may only publish under `fl/{tenant}/{its-own-id}/...`
  (derived from its certificate). It can subscribe to its own `state/desired`, its groups' topics and
  `desc` queries addressed to it.
- Apps on a device get nothing beyond their own sub-namespace unless their manifest asks and policy grants.
- Commands and config writes are always end-to-end checked; no "trusted network" shortcut.

## Delegation and offline operation

- Authority flows by **delegation**: tenant root → site admin → technician, each step can only
  **narrow** rights (fewer devices, fewer actions, shorter validity). No server is needed to check it.
- **Capability tokens that can be attenuated offline** fit this well: a site admin can derive a
  narrower token for a technician on the spot.
- **Revocation** (non-monotonic, see CALM): short-lived capabilities renewed when connected; long
  offline sites get a **site-level issuer** that renews locally; revocation lists spread as additive facts.
- Every decision on commands, config writes and deployments is written to the append-only audit log
  and reported upstream.

## How the verifier and transport ACLs work together

```
 signed policy / delegations ──► verifier (trusted) ──► verified grants: (principal, action, key expression, expiry)
                                                              │
                                                              ▼
                                     ACL compiler (untrusted, outside the TCB, only translates formats)
                                       │                   │                         │
                                       ▼                   ▼                         ▼
                              Zenoh ACL config     MQTT plugin allow/deny     external broker ACL
                              (routers, peers,     + bridge credentials       (e.g. Mosquitto acl_file,
                               local app router)                               pattern fl/acme/%c/#)
```

**Division of labour:**

| Concern | Protected by | Why |
|---|---|---|
| **Causing effects** (commands, config, deployments) | Verifier, end to end on the target device | Holds even if a router, bridge or ACL config is wrong or compromised |
| **Reading data** (who may subscribe/query) | Transport ACLs | Without payload encryption, subscription ACLs are the confidentiality mechanism |
| **Network abuse** (flooding, injecting into foreign namespaces) | Transport ACLs at the first hop | Stops it early, before it spreads through the mesh |
| **Telemetry authenticity** | Identity-bound namespace at the first hop; optional batch signatures end to end | Per-message signatures are too expensive for telemetry |

**Rules:**
- **Identity binding**: Zenoh links use mTLS (TLS or QUIC); the certificate's identity is the ACL subject.
  The first rule every node gets: a subject may only `put` under `fl/{tenant}/{its-id}/**`.
  If Zenoh's ACL has no per-identity substitution, the compiler generates one rule per device.
- **Every node enforces its own sessions**: routers for their links, peers for their direct peer links,
  the device's local router for its local apps. There is no single choke point, matching the
  decentralized design.
- **Local apps** get their own credentials on the device's local Zenoh router (or Unix socket
  credentials); their ACL comes from the permissions in their manifest that the policy granted.
- **Legacy MQTT clients** authenticate at the bridge (mTLS or username/password). They cannot sign,
  so a **gateway principal** signs on their behalf, with a delegation limited to exactly those devices
  and actions.
- **Policy updates**: new verified grants → recompile → reload ACL configs. Until a node reloads,
  the verifier still blocks unauthorized effects; only read access may lag.
- **A compiler bug only weakens the coarse layer**: it can expose data or let junk through, but never
  authorize a command, config change or deployment. That is why the compiler can stay outside the TCB.

**Known gaps:**
- A compromised relay sees the traffic it forwards and can drop it. Confidentiality across untrusted
  relays needs payload encryption to groups (open point below).
- To verify: Zenoh ACL features (subjects by certificate name, per-identity templates, hot reload) and
  the MQTT plugin's per-client authorization granularity.

## Candidate building blocks (all Rust-friendly, permissive)

| Tool | Role | Notes |
|---|---|---|
| **Cedar** (AWS, Apache-2.0, Rust) | Policy language + evaluation | Readable, fast, formally analyzable (can prove "no technician can ever deploy OS images") |
| **Biscuit** (Eclipse Biscuit, Apache-2.0, Rust) | Offline-attenuable capability tokens with Datalog checks | Verify ed25519/P-256 today; PQ support to check (crypto agility) |
| X.509 + SPIFFE-style IDs | Identity of devices, apps, services | Already in the trust model |
| **Zenoh access control** | Transport ACLs by certificate name, username, interface; per key expression and action | Generated from policy |
| **Nebula** groups / WireGuard + nftables | Network ACLs | Generated from policy |
| **Keycloak**, **OpenFGA** / **SpiceDB** | Human login and backend authorization | |

Alternatives considered: OPA/Rego (Go, general-purpose, less analyzable), UCAN, macaroons.

## Open points

- ~~Cedar for policies + Biscuit for tokens?~~ Direction (see 07, minimal trusted code): **not both**.
  Start with **one delegation mechanism for everything** (config, rollouts, commands, topic rights):
  COSE-signed delegation documents with a small, fixed constraint vocabulary (tenant, site, labels,
  components, actions, config fields, validity), verified in `fleetloom-verifier`. This mechanism is needed
  for desired state anyway, so it adds no second system. If richer conditions are ever needed, Biscuit is
  the fallback (one library, Datalog). Biscuit caveats: tokens are Protobuf-encoded (a second encoding
  in the trusted path) and only support Ed25519/P-256 today (no post-quantum yet).
- Group encryption for confidential topics across untrusted relays (beyond signing).
- How to present effective permissions to operators ("why can X do Y?").
