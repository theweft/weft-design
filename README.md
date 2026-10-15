# fleetloom

**Weave existing open tools into one end-to-end embedded fleet platform.**

fleetloom is an integration project, not a rewrite. It aims to cover the full path of an
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

## Name

*Fleet* says what it manages. A *loom* weaves existing threads into one fabric, which is exactly
what this project does with existing tools.

## Docs

- [docs/01-problem-map.md](docs/01-problem-map.md): the problems we have to solve, and where the seams go.
- [docs/02-landscape.md](docs/02-landscape.md): rough literature/tool survey per problem.
- [docs/03-licensing.md](docs/03-licensing.md): licence policy and a permissive stack (no GPLv3/AGPL/BSL in the core).
- [docs/04-decisions.md](docs/04-decisions.md): current design decisions, open questions and next steps.
- [docs/05-envelope-and-config.md](docs/05-envelope-and-config.md): draft design of the message envelope, topic namespace and configuration model.

## Status

Exploration / research phase.
