# Design decisions (draft)

Status: working draft from the first research session (2026-10-15). "Decided" means current
direction, not frozen. Each item can be revisited with a short rationale.

## Core principles

| # | Principle | Status |
|---|---|---|
| D1 | **Every layer is swappable** behind a seam (contract), preferably an existing open standard | Decided |
| D2 | **Offline by design, cloud optional**: everything works without cloud or internet | Decided |
| D3 | **Decentralized by design**: authority comes from signing keys, not from a server location. Desired state is signed documents that any path can carry (cloud, peer, gateway, USB). Each device reconciles locally. Servers are optional caches, aggregators and UIs | Decided |
| D4 | **Topology-flexible pub/sub**: peer↔peer, peer→peer→cloud, direct to cloud, mesh; topology is configuration, not code | Decided |
| D5 | **Permissive licences**: no GPLv3/LGPLv3/AGPL/BSL/SSPL in the core; GPLv2/LGPL-2.1 for separate programs is OK. fleetloom itself: Apache-2.0 | Decided (see 03-licensing.md) |
| D7 | **Certifiable by design**: make adopters' certification cheap (EU CRA, IEC 62443-4-1/4-2, EN 18031, automotive). Ship a compliance kit: threat models, requirement-to-feature-to-test mappings, hardening guide, security-level profiles. Documented secure development lifecycle, signed reproducible releases with SBOM/VEX/SLSA provenance, vulnerability handling policy, LTS releases with defined support periods. Keep fleetloom out of safety functions | Decided |
| D8 | **Rust everywhere**: device agent, backend, CLI and tooling in Rust (matches Zenoh, bootc, Ferrocene for safety-adjacent work). MCUs: Rust (Embassy, or Rust on Zephyr) where possible, C only where an existing C stack is required (zenoh-pico, Zephyr subsystems). Plugins talk over process boundaries (local IPC), so third-party plugins can use any language. Pinned toolchains, minimal dependencies, cargo-deny/cargo-vet/cargo-audit | Decided |
| D9 | **Minimal trusted code**: security decisions live in the **verifier** (`fleetloom-verifier`), one small, pure, `no_std`, unsafe-free Rust crate (verify, evaluate policy, resolve state) shared by Linux and MCUs; privilege-separated agent (sandboxed transport, unprivileged decider, tiny privileged executor); one encoding/signature/artifact format in the trusted path; no scripting in the trusted path; code and dependency budgets in CI. See [07-minimal-tcb.md](07-minimal-tcb.md) | Decided |
| D6 | **Crypto agility and post-quantum readiness**: algorithm IDs in all signed data, multiple keys per role, hybrid key exchange where available, PQ/hybrid signatures for long-lived roots first | Decided |

## Mandatory core (the "narrow waist")

Contracts are mandatory, implementations are not:

1. Trust model: device identity, signing/verification, scoped key delegation (TUF-style), algorithm agility.
2. Desired/reported state document format: versioned, signed, scoped, with deterministic conflict rules.
3. Artifact references: content-addressed, OCI format, signed.
4. Plugin interfaces + capability advertisement per device.
5. Message envelope (CloudEvents-compatible + fleetloom extensions: signature, sequence, correlation)
   and topic namespace (device-identity based, `fl/{tenant}/{device}/{domain}/...`, location via aliases).
6. Status/health vocabulary + OpenTelemetry conventions.

Each contract gets conformance tests. Margo is an adapter, not part of the core.

## Conflicts and consistency

- Desired state from several authorities: no merging; deterministic precedence (scope priority, then
  monotonic version per scope), field locks, delegations limit what a scope may change.
- Reported state, telemetry, audit: append-only.
- Shared editable data: CRDTs.
- Fleet-wide invariants (e.g. max % updating): coordination or pre-allocated quotas.
- CALM as the design test: monotonic decisions (abort rollout, revoke) are coordination-free;
  non-monotonic ones (proceed rollout, "key still valid") need a seal, a timeout or a coordinator.
- Conflicts are never silent: devices report overrides.
- Logical/hybrid clocks for ordering, not wall clock.

## Layer choices (defaults, all replaceable)

| Layer | Default | Notes |
|---|---|---|
| Envelope & config | See [05-envelope-and-config.md](05-envelope-and-config.md) | Draft |
| Access control | See [06-access-control.md](06-access-control.md) | Draft |
| Pub/sub core | **Zenoh** | Peer, routed and client modes; zenoh-pico for MCUs |
| MQTT compatibility | **Zenoh MQTT plugin / bridge** | Existing brokers attach via standard broker bridging. To test: QoS, retained messages, topic edge cases |
| Push vs pull | Both | Signed documents can be pushed over pub/sub and pulled on reconnect; trust is in the signature |
| OTA, embedded profile | **Yocto + A/B with SWUpdate** (preferred over RAUC), minimal base OS, apps in containers | SWUpdate is GPL-2.0: fine; keep custom handlers generic/open or out of process |
| OTA, image-mode profile | OSTree/bootc delivered as OCI | Gateways and edge PCs; Fedora IoT/CentOS bootc or Yocto+OSTree (prototype needed) |
| OTA, MCU profile | MCUboot + SUIT | Zephyr |
| Artifacts | OCI registry for everything (SWUpdate bundles via ORAS) | Same signing, mirroring, offline copy for all profiles |
| Workloads | Podman (Quadlet) / Eclipse Ankaios | Margo app package as optional format |
| Overlay network | WireGuard managed by fleetloom, or **Nebula** | OpenZiti as optional zero-trust plugin; Rosenpass for PQ WireGuard |
| Identity | step-ca / OpenBao, TPM 2.0, FDO onboarding | |
| Ingestion | Kafka protocol, Bento, ClickHouse/QuestDB | |
| Observability | OpenTelemetry, Fluent Bit, VictoriaMetrics, Perses | |
| Twin / read model | Optional central read model (Ditto-like) built from signed documents | Not the source of truth |

## Industry standards: hub and spoke

- Small canonical model (entities, signals with value/type/unit/time/quality, events, commands with
  replies, desired/reported state, descriptions); one adapter per standard, never standard-to-standard.
- Declarative mappings (Bento), semantic IDs as references (OPC UA types, AAS semantic IDs, IPSO, ECLASS),
  units normalized (UCUM), self-description + schema registry, conformance tests per adapter.
- First adapters: **Sparkplug B** and **OPC UA (PubSub + client via open62541)**. Later: Unified Namespace
  (ISA-95 paths), VDA 5050, LwM2M, AAS. Field protocols via Apache PLC4X / Telegraf.

## Positioning vs. existing projects

Nobody combines full stack + swappable seams + topology-flexible pub/sub + decentralized offline
operation + permissive licences. Closest: balena, Red Hat Flight Control, Mender (lifecycle);
Margo and FLECS (workload layer only; FLECS is open-core, implements Margo); Viam (topology, robotics).

## Open questions

1. Single fleetloom agent with plugins vs. orchestrating existing agents.
2. Revocation strategy for long-offline devices (short-lived delegations vs. clock trust).
3. Rollout "proceed" decisions: sealed waves vs. deterministic cohorts with local health gates.
4. Exact document format (TUF metadata extension? own format?) and bundle format.
5. End-to-end payload signing across untrusted hops: cost on MCUs and constrained links.
6. bootc on Yocto: feasibility prototype.

## Next steps

1. Prototype: Mosquitto client on device A ↔ native Zenoh app on device B (peer-to-peer) ↔ existing
   Mosquitto broker in "cloud" via the Zenoh MQTT plugin.
2. Deep evaluation: SWUpdate, Flight Control, Mender, Eclipse Kanto/Ankaios, Zenoh security model, hawkBit, Margo PR spec.
3. Draft the desired-state document format and trust model (first mandatory contract).
4. Set up project hygiene for certifiability from day one: SECURITY.md + disclosure policy, OpenSSF Scorecard and Best Practices badge, SBOM/provenance in CI; evaluate hosting under a foundation (Eclipse ORC working group for CRA).
