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

## Name

*Fleet* says what it manages. A *loom* weaves existing threads into one fabric, which is exactly
what this project does with existing tools.

## Docs

- [docs/01-problem-map.md](docs/01-problem-map.md): the problems we have to solve, and where the seams go.
- [docs/02-landscape.md](docs/02-landscape.md): rough literature/tool survey per problem.

## Status

Exploration / research phase.
