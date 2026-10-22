# Design decisions (draft)

Status: working draft from the first research session (2026-10-15). "Decided" means current
direction, not frozen. Each item can be revisited with a short rationale.

## Core principles

| # | Principle | Status |
|---|---|---|
| D1 | **Every layer is swappable** behind a seam (contract), preferably an existing open standard | <!--f:principle.d1-->Decided<!--/f--> |
| D2 | **Offline by design, cloud optional**: everything works without cloud or internet | <!--f:principle.d2-->Decided<!--/f--> |
| D3 | **Decentralized by design**: authority comes from signing keys, not from a server location. Desired state is signed documents that any path can carry (cloud, peer, gateway, USB). Each device reconciles locally. Servers are optional caches, aggregators and UIs | <!--f:principle.d3-->Decided<!--/f--> |
| D4 | **Topology-flexible pub/sub**: peer↔peer, peer→peer→cloud, direct to cloud, mesh; topology is configuration, not code | <!--f:principle.d4-->Decided<!--/f--> |
| D5 | **Permissive licences**: no GPLv3/LGPLv3/AGPL/BSL/SSPL in the core; GPLv2/LGPL-2.1 for separate programs is OK. weft itself: **<!--f:license.weft-->MIT<!--/f-->** for all repositories (consistent; contributions under the Developer Certificate of Origin) | <!--f:principle.d5-->Decided<!--/f--> (see 03-licensing.md) |
| D7 | **Certifiable by design**: make adopters' certification cheap (EU CRA, IEC 62443-4-1/4-2, EN 18031, automotive). Ship a compliance kit: threat models, requirement-to-feature-to-test mappings, hardening guide, security-level profiles. Documented secure development lifecycle, signed reproducible releases with SBOM/VEX/SLSA provenance, vulnerability handling policy, LTS releases with defined support periods. Keep weft out of safety functions | <!--f:principle.d7-->Decided<!--/f--> |
| D8 | **Rust everywhere**: device agent, backend, CLI and tooling in Rust (matches Zenoh, bootc, Ferrocene for safety-adjacent work). MCUs: Rust (Embassy, or Rust on Zephyr) where possible, C only where an existing C stack is required (zenoh-pico, Zephyr subsystems). Plugins talk over process boundaries (local IPC), so third-party plugins can use any language. Pinned toolchains, minimal dependencies, cargo-deny/cargo-vet/cargo-audit | <!--f:principle.d8-->Decided<!--/f--> |
| D9 | **Minimal trusted code**: security decisions live in the **verifier** (`weft-verifier`), one small, pure, `no_std`, unsafe-free Rust crate (verify, evaluate policy, resolve state) shared by Linux and MCUs; privilege-separated agent (sandboxed transport, unprivileged decider, tiny privileged executor); one encoding/signature/artifact format in the trusted path; no scripting in the trusted path; code and dependency budgets in CI. See [07-minimal-tcb.md](07-minimal-tcb.md) | <!--f:principle.d9-->Decided<!--/f--> |
| D6 | **Crypto agility and post-quantum readiness**: algorithm IDs in all signed data, multiple keys per role, hybrid key exchange where available, PQ/hybrid signatures for long-lived roots first | <!--f:principle.d6-->Decided<!--/f--> |

## Requirements and facts

- **Values** (limits, requirements, defaults, dates, estimates): `facts.toml` in this repository, checked by `tools/factcheck`.
- **Requirements**: StrictDoc (`.sdoc`) in each repository, traced to code and tests with relation markers on functions. A coverage gate runs in each repository's tests, and ReqIF is exported for assessors.
- **Pilot:** `weft-verifier/requirements/verifier.sdoc`, with 28 requirements, a draft IEC 62443-4-2 / CRA mapping, and the gate in `tests/requirements.rs`.
- **System requirements:** `requirements/system.sdoc` in this repository, SYS-001 to SYS-043:
  - adoption contract;
  - decisions and conflicts;
  - offline and topology;
  - pub/sub transport contract T1–T6;
  - platform.

  Component requirements name their parents in `SATISFIES`.
- **Cross-repository check:** `just check` (factcheck) verifies across all sibling repositories:
  - requirement ids are unique;
  - `SATISFIES` and `FACTS` references resolve;
  - every Active system requirement verified by Test is satisfied by a component requirement.

  Draft system requirements (executor, identity, pub/sub, topology) mark what is designed but not built yet.
- Requirements refer to principles D1–D9 and to fact ids. Values stay in `facts.toml`.

## Naming

- All crates and repositories are `weft-*`.
- **Components that act** get role nouns: `weft-verifier`, `weft-executor`, `weft-agent`.
- **Formats** get artifact nouns, with their check and encode functions inside: `weft-envelope`, `weft-directive`, `weft-parts` (chip data).
- Names never suggest more security than a component provides. That is why it is `weft-directive`, not `weft-verifier-lite` (docs/18).
- **Project name "weft" kept for now** (2026-10-18). Name check: no registered WEFT mark for software
  found, but the name is crowded: WeaveMindAI's weft (Rust AI orchestration language, about 2,000
  stars, since 2026-04), Weft Learning (weftlearning.com, US mark WEFT LEARNING for education SaaS),
  the `weft` crate on crates.io and many other weft projects. Compound "weft fleet" rejected: "fleet"
  is descriptive and "WeftFleet" is close to Webfleet (Bridgestone, fleet management). Revisit
  (attorney check, EU filing in classes 9 and 42, or rename) before the first release, funding, a
  foundation move, or if another party files WEFT for software; the EU is first-to-file.

## Brand

Logo (2×2 weave), colours (indigo and amber), wordmark (Inter) and lockups: decided, with rejected
alternatives and rationale, in [19-brand.md](19-brand.md).

## Dev environment and tests

Decided 2026-10-20:

- **Platforms in two tiers.**
  - **Tier 1, native on Linux, macOS and Windows:** rustup (reads `rust-toolchain.toml`), `just`, `cargo test` including deterministic simulations, the CLI, and the agent's logic. CI runs tier 1 on all three operating systems.
  - **Tier 2, Linux only:** VM, network and filesystem fault tests. On macOS they run in a Linux VM (Lima, OrbStack, UTM); on Windows in WSL2.
  - **Nix is optional.** It is a convenience and the CI environment, not a requirement.
  - **Justfile rule:** no bash constructs in recipes, so that tier 1 works on Windows. Use parameterized dependencies instead of shell loops.
- **Dev loop: Tilt** with Docker Compose or local processes as the backend.
  - Container runtime: Podman Desktop or Rancher Desktop recommended; Docker Desktop needs a paid subscription in larger companies.
  - Kubernetes is not required. A second Tilt profile (`--backend=k8s`, k3d or Rancher Desktop's k3s) comes once there is a multi-replica cloud backend or adopters ask for Helm charts.
  - Every service is an OCI image with health checks, configured through files and environment variables, so Compose and Kubernetes can use the same images.
- **Test layers:**
  1. unit and integration tests per repository;
  2. **deterministic simulation (turmoil)** of weft's own async code: agent, document sync, store and forward. Seeded, all platforms, many runs per CI job;
  3. crash consistency with LazyFS (persistence protocol, docs/11);
  4. system end-to-end tests with NixOS VM tests (power loss, reboots, USB flow), and Containerlab for topologies T1–T7;
  5. weft-sim for drivers and hardware access;
  6. later: Shadow (real binaries such as zenohd at scale, simulated network and time) and labgrid (real hardware lab).
- **Repository:** a new `weft-lab` repository will hold the Tiltfile and the end-to-end scenarios across repositories. It is created once there is a second running component (agent or CLI).

## Board simulation (weft-sim)

Decided 2026-10-18, prototype working (one chip, x86-64 guest):

- **Models run inside the guest.** A small kernel module (`weft_sim.ko`) provides I2C (later SPI)
  adapters that forward each transfer to `/dev/weft-sim`; a static Rust runner answers from the
  weft-parts models. Bus traffic never leaves the VM, so the host needs only a stock QEMU, and the
  same module and runner also work on a real Raspberry Pi without the HAT.
- **The guest's CPU follows the host's** (x86-64 or arm64), so KVM, HVF or WHPX always accelerate
  it; emulating another CPU is not acceptable as the default. Drivers and overlays are
  architecture-independent: an x86-64 guest applies Raspberry Pi overlays to its device tree.
- **Not vhost-user** (the earlier plan): rust-vmm's vhost crates are Linux-only, QEMU supports
  vhost-user on macOS only when built with an extra option (Homebrew's QEMU is not), and never on
  Windows. vhost-user stays possible as a later Linux-only transport.
- **The module is C**, dual MIT OR GPL-2.0-only: Rust-for-Linux has no I2C adapter or SPI
  controller abstractions yet, and a C module loads into any kernel, including Raspberry Pi OS
  kernels built without Rust. It only copies bytes; all logic stays in Rust. Rewrite in Rust when
  the abstractions exist.
- **Host tools are portable; weft-sim needs QEMU.** Board checks and embedded-hal model tests run
  natively on Linux, macOS and Windows. weft-sim runs wherever QEMU has a hypervisor (KVM, HVF on
  macOS 15+ via Homebrew, WHPX).
- **Guest kernel:** the prototype uses the prebuilt nixpkgs kernel; the target is weft's own minimal
  configuration on an upstream LTS kernel, built in CI and published as a ready-made image, so
  users need only QEMU.
- **Name:** weft-sim, not "weft-twin": in fleet platforms "device twin" means the cloud-side
  mirror of device state (AWS shadows, Azure twins), which weft also has.

## AI-built, human-decided

weft is developed heavily with an AI coding assistant (Claude Code): it drafts most docs, code,
tests and artwork; the maintainer directs, decides and reviews. This is stated up front in the
[README](../README.md#how-weft-is-made) and marked per commit with a `Co-Authored-By` trailer.
Consequence for the design: prefer claims that can be checked mechanically (facts with sources,
tests against independent implementations, CI) over prose that has to be trusted.

## Mandatory core (the "narrow waist")

Contracts are mandatory, implementations are not:

1. Trust model: device identity, signing/verification, scoped key delegation (TUF-style), algorithm agility.
2. Desired/reported state document format: versioned, signed, scoped, with deterministic conflict rules.
3. Artifact references: content-addressed, OCI format, signed.
4. Plugin interfaces + capability advertisement per device.
5. Message envelope (CloudEvents-compatible + weft extensions: signature, sequence, correlation)
   and topic namespace (device-identity based, `weft/{tenant}/{device}/{domain}/...`, location via aliases).
6. Status/health vocabulary + OpenTelemetry conventions.

Each contract gets conformance tests. Margo is an adapter, not part of the core.

The concrete adoption contract (what a system must accept to use weft) and adoption levels L1–L4 are in [14-core-features-and-gaps.md](14-core-features-and-gaps.md).

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
| Overlay network | WireGuard managed by weft, or **Nebula** | OpenZiti as optional zero-trust plugin; Rosenpass for PQ WireGuard |
| Identity | step-ca / OpenBao, TPM 2.0, FDO onboarding | |
| Ingestion | Kafka protocol, Bento, ClickHouse/QuestDB | |
| Observability | OpenTelemetry, Fluent Bit, VictoriaMetrics, Perses | |
| Twin / read model | Optional central read model (Ditto-like) built from signed documents | Not the source of truth |
| Hardware test | **`weft-parts`** holds one data file per chip (pinouts, address straps, device tree facts) plus behavioural I2C/SPI models; **`weft-boardchecker`** checks the PCB (IPC-2581, fallback IPC-D-356A + BOM) against the compiled overlay, run by KiBot; **`weft-sim`** runs the real Linux drivers in a small QEMU guest against the models (see Board simulation below); **embedded-hal** runs Rust MCU drivers against them on the host; swtpm, Zephyr native_sim, KiCad + ngspice | Existing tools where they do the job; weft writes only chip data, models, the board checker and weft-sim's bus bridge. Pi firmware, boot chain, switch fabric and analogue behaviour need real hardware. See [09-prior-art.md](09-prior-art.md) |

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

1. Single weft agent with plugins vs. orchestrating existing agents.
2. Revocation strategy for long-offline devices (short-lived delegations vs. clock trust).
3. Rollout "proceed" decisions: sealed waves vs. deterministic cohorts with local health gates.
4. Exact document format (TUF metadata extension? own format?) and bundle format.
5. End-to-end payload signing across untrusted hops: cost on MCUs and constrained links.
6. bootc on Yocto: feasibility prototype.

## Next steps

Updated 2026-10-22. **L1 (trust) first**: signed documents, the verifier and an executor, usable with an adopter's own messaging and OTA. weft's own communication is L3 and comes after.

Done:

- desired-state document format and trust model (`weft-verifier` 0.0.3);
- document sync and decider, tested in a deterministic simulation (`weft-agent` 0.0.1, docs/20);
- **operator tool `weft-cli`** 0.0.1: keys, delegations, config, deployments, revocations, inspect, and a dry run that fails on conflicts;
- project hygiene: SECURITY.md, private vulnerability reporting, requirements as code.

L1, in order:

1. ~~**Atomic file state store**~~ done 2026-10-22: `weft_decider::FileStore` (temp file, fsync, rename, fsync the directory; checksummed), SYS-006 Active. Power cuts at every point of a save on turmoil's simulated filesystem: no acknowledged state lost in 300 runs; without fsync, 300 of 300 lost one.
2. ~~**Inbox and outbox directories**~~ done 2026-10-22: the `weft-agent` binary reads documents from an inbox (read-only, e.g. a USB stick), stores them before deciding (`<state>/documents/`), and writes `outbox/report.json` after every round. SYS-008 and SYS-020 Active.
3. ~~**Executor, first backends**~~ done 2026-10-22: config as `<config-dir>/<component>.json` (atomic, path-safe) plus `--on-config`; `Install` through `--on-install` with the digest in an environment variable; executed results persisted (at least once). End-to-end test: USB stick, restart without the stick, tie, forged document, failing hook.
4. ~~**Process separation**~~ done 2026-10-22: `weft-agent decider` (unprivileged, no network, read-only system via systemd) and `weft-agent executor` (privileged) over a Unix socket with typed, bounded CBOR requests; the executor re-checks hashes and paths and runs only its own hooks. SYS-003 Active.
5. **End-to-end test** as a NixOS VM test: signed config on a "USB stick" applied offline, conflicts reported, power cut during apply.

After L1:

- L2: SWUpdate, bootc and Podman backends; activation gates.
- L3: Zenoh and MQTT transport adapters (the Mosquitto ↔ Zenoh ↔ broker prototype); Shadow for scale tests.
- Deep evaluations: thin-edge.io, SWUpdate, Flight Control, Mender, Eclipse Kanto/Ankaios, hawkBit, Margo.
- Hygiene: OpenSSF Scorecard and Best Practices badge, SBOM and provenance in CI, actions pinned to commit SHAs; foundation hosting (Eclipse ORC working group for CRA).
