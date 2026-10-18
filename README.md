# weft

**Weave existing open tools into one end-to-end embedded fleet platform.**

weft is an integration project, not a rewrite. It aims to cover the full path of an
embedded device fleet:

- **Lifecycle**: provisioning, identity, OTA updates (OS, firmware, apps), rollbacks, decommissioning.
- **Workloads & config**: container-based app deployment and declarative configuration (desired vs. reported state).
- **Communication**: publish/subscribe messaging between devices and cloud (telemetry up, commands/actions down).
- **Ingestion**: routing, storing and processing what the fleet produces.
- **Operations**: observability, remote access, inventory, rollout control.

## Core principle: every layer is swappable

Each layer sits behind a *seam*: a well-defined contract, preferably an existing open standard
(OCI, MQTT/Sparkplug, CloudEvents, OpenTelemetry, TUF/Uptane, ...). If a user already has a
solution for a layer (e.g. their own MQTT broker, Mender, or a Kafka-based pipeline), they plug
it in at the seam instead of adopting ours.

## Core principle: offline by design, cloud optional

Everything (provisioning, updates, config, workloads, messaging, data collection) must work with
no cloud and no internet. The cloud is just one optional place to run the control plane:

- The **same backend** can run in the cloud, on-prem, on a laptop, on a gateway, or on the device itself.
- Updates and config can travel as **signed offline bundles** (USB stick, local file share, service laptop).
- Devices keep working **autonomously** when disconnected, and reconcile when a control plane is reachable again.

## Core principle: topology-flexible communication

Pub/sub works peer-to-peer, via other peers or gateways, or directly to the cloud, with the same
application API. The topology is configuration, not code.

## Adoption levels

weft can be adopted step by step. Only the adoption contract is mandatory: signed documents, the verifier on (or for) each device, and a small executor. See [docs/14-core-features-and-gaps.md](docs/14-core-features-and-gaps.md) §2.

| Level | Adds | Typical adopter |
|---|---|---|
| **L1 Trust** | Signed config and deployments, `weft-verifier`, executor. Your own OTA tool and messaging stay | Device maker with an existing update stack |
| **L2 Lifecycle** | weft OTA profiles, apps, activation gates | Gateway or embedded Linux product |
| **L3 Connected** | weft pub/sub, node roles, overlay, access policy | Multi-device sites, mixed topologies |
| **L4 Full** | Ingestion, industry adapters, fleet read model | End-to-end fleet operator |

## Name

*weft* is the thread that runs across a loom and ties all the fixed threads into one fabric.
weft does the same with existing tools: it weaves them into one platform without replacing them.

- Website: [theweft.org](https://theweft.org) (also theweft.dev, weftfleet.org, weftfleet.dev, weftfleet.com)
- Crates: `weft-*` (e.g. `weft-verifier`)

## Docs

- [docs/01-problem-map.md](docs/01-problem-map.md): the problems we have to solve, and where the seams go.
- [docs/02-landscape.md](docs/02-landscape.md): rough literature/tool survey per problem.
- [docs/03-licensing.md](docs/03-licensing.md): licence policy and a permissive stack (no GPLv3/AGPL/BSL in the core).
- [docs/04-decisions.md](docs/04-decisions.md): current design decisions, open questions and next steps.
- [docs/05-envelope-and-config.md](docs/05-envelope-and-config.md): draft design of the message envelope, topic namespace and configuration model.
- [docs/06-access-control.md](docs/06-access-control.md): draft design of access control (policy model, enforcement points, delegation).
- [docs/07-minimal-tcb.md](docs/07-minimal-tcb.md): keeping the trusted code small (pure core, privilege separation, fewer formats).
- [docs/08-apps.md](docs/08-apps.md): app deployment, permission tiers, hardware access, microcontrollers.
- [docs/09-prior-art.md](docs/09-prior-art.md): similar projects and standards, what we adopted and why.
- [docs/10-device-requirements.md](docs/10-device-requirements.md): minimum hardware and software per device class, with measurements.
- [docs/11-time-and-persistence.md](docs/11-time-and-persistence.md): which clock for what, and power-loss-safe state.
- [docs/12-case-study.md](docs/12-case-study.md): target industries, their typical hardware/network/backend setups, and what follows for weft.
- [docs/13-topologies-and-roles.md](docs/13-topologies-and-roles.md): node roles (endpoint, relay, uplink, proxy, site services) and the supported topologies.
- [docs/14-core-features-and-gaps.md](docs/14-core-features-and-gaps.md): feature overview, the adoption contract (what is mandatory), adoption levels, and open gaps.
- [docs/15-pubsub-contract.md](docs/15-pubsub-contract.md): what weft requires from a pub/sub transport, what it builds itself, and how transports are swapped.
- [docs/16-transport-protocols.md](docs/16-transport-protocols.md): TLS, QUIC, HTTP versions, WebSockets, firewall traversal, constrained links.

## Status

Exploration / research phase.
