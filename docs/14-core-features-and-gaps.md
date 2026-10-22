# Core features, adoption contract, and gaps (draft, 2026-10-18)

An overview of the other docs: what weft does, what a system must accept to use it, and what is still missing.

## 1. Core features

| # | Feature | What it gives | Status | Docs |
|---|---|---|---|---|
| F1 | **Signed, decentralized desired state** | Config, deployments, delegations and revocations as signed documents (CBOR + COSE). Any path can carry them, and every device decides locally | Verifier implemented | 05, 07 |
| F2 | **Delegated authority with priorities** | Keys with narrowing rights (tenant, actions, priority range, components, labels, validity). Numeric priorities with a tenant scope profile. Locks with override right | Implemented | 05 §2.3, 06 |
| F3 | **Deterministic conflict handling** | Ties, locks, deployment ties and equivocation become stable conflict ids, never silently. Reported everywhere, and checked before rollout by a dry run | Implemented in the verifier, surfacing designed | 05 §2.3b |
| F4 | **Rollback and freshness protection** | Version marks per slot, a time floor, revocation, and withholding a component when something is missing | Implemented (TUF timestamp role still missing) | 07, 11 |
| F5 | **OTA for three device profiles** | Yocto A/B (SWUpdate), image mode (bootc/OSTree via OCI), MCU (MCUboot + SUIT). One OCI registry for all artifacts. Staging separate from activation | Designed | 04, 08 |
| F6 | **Activation gates** | Device state, app veto, approval, time window, and emergency override as a right | Designed | 08, 11 |
| F7 | **Apps without restrictions on developers** | Manifest, Podman/Quadlet, permission tiers 0–3, opaque grants (`hw:`, `net:`, `pub:`, `secret:`, ...), hardware profiles with stable matching. Anything is possible, but visible and approved | Designed | 08 |
| F8 | **Topology-flexible pub/sub** | One envelope and topic namespace across peer, relay, uplink, mesh and hierarchy. Zenoh by default, MQTT and NATS by bridge | Designed | 01 P4a, 05, 13 |
| F9 | **Node roles** | Endpoint, relay, uplink, proxy and site services, assigned by config. The same build fits all topologies T1–T7 | Designed | 13 |
| F10 | **Overlay networking and remote access** | Nebula or weft-managed WireGuard (Rosenpass for post-quantum), subnet routing through uplinks, time-bounded audited access | Designed | 01 P11, 13 §5 |
| F11 | **One access policy, many enforcers** | Compiled to Zenoh ACLs, MQTT ACLs, overlay rules and app firewalls. Network conduits per IEC 62443 | Designed | 06, 08 |
| F12 | **Offline operation everywhere** | USB or laptop bundles, local UI, site services, store and forward. No cloud required at any point | Principle, partly designed | 04 D2, 11, 13 |
| F13 | **Ingestion and observability** | Store and forward, Kafka protocol, Bento, ClickHouse/QuestDB, OpenTelemetry | Layer choice only | 04 |
| F14 | **Industry adapters (hub and spoke)** | Modbus first, then OPC UA and Sparkplug, onto a small canonical signal model | Layer choice only | 04, 12 |
| F15 | **Certifiability kit** | CRA, IEC 62443, SBOM, reproducible releases, LTS, minimal trusted code | Principle, project hygiene started | 04 D7, D9 |

## 2. Adoption contract: what a system must accept to use weft

This is the narrow waist (docs/04, "Mandatory core"), made concrete. **Only these points are mandatory.** Every product mentioned elsewhere (Zenoh, SWUpdate, Podman, Nebula, ClickHouse, any cloud) can be replaced or left out.

1. **Authority is keys, not servers.**
   - Desired state reaches a device only as signed weft documents, under a tenant trust anchor.
   - No server can directly command a device into a new state.
2. **The device decides.**
   - Every managed device runs `weft-verifier`, or is managed by a proxy that runs it on the device's behalf and is shown as such.
   - The device applies *only* the verifier's instructions, through a small, separate executor.
3. **Each device has its own key pair, hardware-backed where possible.** Device identity is the key, not a name or an address.
4. **Software is named components with content-addressed artifacts.**
   - Every deployable thing is a component (`os`, `apps/x`, `fw/mcu1`) referring to an artifact by digest.
   - Versions only move forward per slot.
5. **State is persisted atomically before acting.**
   - The verifier state (marks, time floor, revocations) survives power loss (docs/11).
   - Anti-rollback depends on it.
6. **Order comes from versions, not clocks.** Wall time is used only as a lower bound for expiry.
7. **The device reports back in weft terms.**
   - Reported state, conflicts and withheld components are reported in the weft vocabulary, so any backend or local UI can show them.
   - How they travel is free.
8. **Messages that use weft pub/sub use the envelope and topic namespace.** The transport is free. A system can also keep its own messaging and only use 1–7.

### Adoption levels

A product can adopt weft gradually:

| Level | Uses | Typical adopter |
|---|---|---|
| **L1 Trust** | Contract 1–7: signed config and deployments, the verifier and the executor. The adopter keeps their own OTA tool and messaging | Device maker with an existing update stack |
| **L2 Lifecycle** | L1 plus weft OTA profiles, apps and activation gates | Gateway or embedded Linux product |
| **L3 Connected** | L2 plus weft pub/sub, roles, overlay and access policy | Multi-device sites, mixed topologies |
| **L4 Full** | L3 plus ingestion, adapters and fleet read model | End-to-end fleet operator |

## 3. Gaps

### 3.1 Known and listed (verifier roadmap and earlier docs)

- Removal ("absent") deployments.
- Permission grants as opaque strings checked by the verifier.
- Failed installs in device facts.
- TUF-style timestamp and snapshot roles, signature thresholds and root rotation.
- A time-quality fact.
- `authorize()` for commands.
- SUIT manifests for MCU artifacts.
- Fuzzing and Kani proofs.
- ML-DSA / hybrid signatures.
- Ordering and dependencies between components (docs/12 §5).
- Transfers that depend on the link class (docs/12 §5).
- Revocation reaching long-offline sites (docs/04 open question 2, docs/13).

### 3.2 Not designed yet (only named in the problem map, or missing)

| # | Gap | Why it matters | First idea |
|---|---|---|---|
| G1 | **Device lifecycle**: manufacturing, onboarding, ownership transfer, resale, RMA, decommissioning, factory reset | Every fleet needs this. Ownership transfer is a key handover, which suits our model | FDO for onboarding. Ownership = a signed delegation from the previous owner's anchor. Decommissioning = revocation plus key wipe |
| G2 | **Rollout orchestration**: waves, canaries, health checks, automatic halt and rollback, fleet-wide limits | Open question 3. This is what operators look at daily | Deterministic cohorts (hash of device id) plus signed wave documents. Devices report health. Halting is monotonic (CALM), proceeding is signed |
| G3 | **Commands and request/response**: reboot, read a value, set a register | Not only state. Needs idempotency, timeouts, replies, authorization | Signed command documents with an expiry, `authorize()` in the verifier, replies as events |
| G4 | **Secret distribution**: `secret:` grants exist, but there is no delivery mechanism yet | Apps need credentials and certificates | Secrets encrypted to the device key (COSE_Encrypt / HPKE), sealed in TPM. Rotation as ordinary config versions |
| G5 | **Attestation and inventory**: what is *really* running (measured boot, TPM quotes), per-device SBOM | Utilities and IEC 62443 require it. CRA vulnerability handling needs to know which devices are affected | IETF RATS / EAT tokens as signed reported state. SBOM per artifact in the OCI registry (referrers) |
| G6 | **Fleet vulnerability response**: "which devices run the affected component?" within the CRA's 24 h | Legal obligation from 11 Sep 2026 | Read model query over the inventory (G5) plus VEX |
| G7 | **Key management operations**: offline root ceremony, HSMs, threshold signing, recovery from key loss or root compromise | The whole model rests on keys | Document ceremonies. Threshold for roots (with TUF). Recovery documents prepared in advance |
| G8 | **App data and migrations**: persistent volumes across updates, schema migrations, backup and restore, rollback with data | Rollback of an app whose data was migrated forward breaks it | Snapshot before migration (btrfs/LVM or app hook). Rollback restores the snapshot. A manifest field declares the data version |
| G9 | **Diagnostics**: logs, crash dumps, core dumps, remote shell, MCU coredumps (Memfault-like) | Support cost dominates fleets | OpenTelemetry logs, crash uploads as artifacts, remote shell as a time-bounded grant over the overlay |
| G10 | (decider ↔ executor decided 2026-10-22, docs/20 §1; plugins still open) **Plugin interface spec**: the IPC between agent, verifier host, executor and plugins | Mandatory core item 4 is still empty | Pick one local IPC (varlink, or Zenoh on a Unix socket) and define capability advertisement |
| G11 | **Canonical data model and schema registry** | Hub and spoke needs the hub defined | Signal = value, type, unit (UCUM), time, quality. Schemas in CDDL or JSON Schema, stored in the OCI registry |
| G12 | **Sleepy and battery devices** | Delivering documents to devices that wake once an hour | Uplink/proxy keeps a mailbox (like LwM2M queue mode). The device pulls on wake |
| G13 | **Safety interlocks** | weft stays out of safety functions, but activation gates need signals from safety PLCs (machine idle) | Gate inputs as read-only signals from adapters. Never the reverse direction |
| G14 | **Format and contract evolution** | Devices live 15–30 years: FORMAT_VERSION upgrades, mixed fleets | Rules: new versions are additive, devices state the versions they support, and backends serve several |
| G15 | **Privacy and data retention** (GDPR) | Telemetry can be personal data (vehicles, homes, charging) | Data classification label per topic. Retention policy in ingestion. Conduit rules per class |
| G16 | **Tools and UX**: CLI, authoring with scope profiles, web console, local UI, fleet dry-run in CI | Without them nobody adopts it | CLI first (Rust). The local UI is a weft app. The console is built on the read model |
| G17 | **Test infrastructure**: conformance suites per contract, device simulator, topology test bed, HIL | D1 says each contract gets conformance tests | Simulator of N devices in containers, topologies T1–T7 with network namespaces |
| G18 | **Governance and releases**: semantic versioning of contracts, LTS periods, foundation, DCO | Needed for adopters and for the CRA (support periods) | Decide before the first public release |

### 3.3 Suggested order

1. Must come before anyone can pilot:
   - **G10** (plugin interface), because everything else plugs into it;
   - **G2** (rollouts) and **G3** (commands), because operators expect them on day one;
   - **G1** (onboarding), because a device must first enter the fleet.
2. Required for certification and CRA: **G5/G6** (attestation, inventory, vulnerability response), **G7** (key operations), **G14** (format evolution).
3. Needed for real products: **G4**, **G8**, **G9**, **G11**, **G16**, **G17**.
4. Domain-specific later: **G12**, **G13**, **G15**, **G18** (G18 before the first public release).
