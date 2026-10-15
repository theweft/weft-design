# Problem map

A rough sketch of everything an end-to-end embedded fleet platform has to solve. The
problems are grouped by layer. Each layer lists the core problems, the hard parts,
and the **seam**: the contract at which a user can swap in their own solution.

```
 ┌──────────────────────────────── CLOUD / BACKEND ────────────────────────────────┐
 │  P10 Fleet UI / API / inventory / RBAC / multi-tenancy                          │
 │  P9  Observability backend      P8 Ingestion, stream processing, storage        │
 │  P5  Rollout orchestration (OTA + workloads + config)    P3 Artifact registry   │
 │  P2  Identity / PKI / provisioning service                                      │
 └──────────────▲──────────────────────────▲─────────────────────────▲──────────────┘
                │ P4 messaging (pub/sub)   │ P6 artifact download    │ P11 remote access
 ┌──────────────┴──────────────────────────┴─────────────────────────┴──────────────┐
 │                                   EDGE DEVICE                                    │
 │  P7  Device agent: desired/reported state, commands, local broker/buffer         │
 │  P5a Workload runtime (containers / wasm)    P5b Config apply                    │
 │  P6a Update client (A/B, atomic, rollback)   P2a Device identity (TPM/SE)        │
 │  P1  Base OS / firmware build (Yocto, Buildroot, Zephyr, ...)                    │
 └──────────────────────────────────────────────────────────────────────────────────┘
 Cross-cutting: P12 security & supply chain · P13 device classes · P14 compliance
```

## P1. Base OS / firmware build

- Reproducible OS images for Linux-class devices; firmware for MCU-class devices.
- Image layout that supports atomic updates (A/B partitions, OSTree, image-based OS).
- Board support for many hardware targets.
- **Seam:** the output artifact format (disk image, update bundle, OCI image) plus a manifest.

## P2. Device identity, provisioning, onboarding

- Unique, unclonable identity per device (TPM 2.0, secure element, DICE).
- Zero-touch onboarding: device boots, proves identity, gets bound to a tenant/fleet.
- Certificate issuance, rotation, revocation. Factory vs. field provisioning.
- **Seam:** X.509 certs + an enrollment protocol (EST, ACME, FDO). Bring-your-own-CA.

## P3. Artifact storage & distribution

- Store OS images, firmware, containers, config bundles, with versions and metadata.
- Efficient delivery: deltas, resumable downloads, CDN, peer/local caching, low bandwidth links.
- **Seam:** an OCI registry for everything (OCI artifacts), with signed metadata.

## P4. Device ↔ cloud communication (pub/sub)

- Telemetry upstream, commands/actions downstream, request/reply, events.
- Unreliable links: store-and-forward, offline buffering, QoS, backpressure.
- Topic/namespace design, schema & payload encoding (JSON, Protobuf, CBOR, Sparkplug).
- Edge-local messaging between processes and between nearby devices.
- Authorization per device/topic, multi-tenancy.
- **Seam:** a pub/sub API + a message envelope (CloudEvents) + a contract (AsyncAPI), independent of the wire protocol and topology.

### P4a. Topology-flexible pub/sub (key requirement)

The same application code (same topics, same API) must work across any network topology,
chosen per deployment and mixed within one fleet:

```
 (a) peer ↔ peer              device ◄──────► device                     (LAN, no infra)
 (b) peer → peer → cloud      device ──► gateway/peer ──► cloud          (multi-hop, relay)
 (c) peer → cloud             device ─────────────────► cloud            (direct)
 (d) mesh / mixed             devices form a local mesh; any node with uplink routes to cloud
```

- **Location transparency:** a subscriber should not care whether the publisher is on the same
  device, the same LAN, or behind the cloud.
- **Discovery:** automatic local discovery (multicast/gossip) plus configured routers for WAN.
- **Routing & relays:** multi-hop forwarding, gateways that aggregate MCU-class devices, NAT traversal.
- **Partition tolerance:** a local cluster keeps working when the uplink drops, and syncs when it returns.
- **Security across hops:** end-to-end authenticity/authorization, not just per-link TLS, because
  intermediate peers may not be trusted.
- **Policy:** what may leave the local network (data sovereignty, bandwidth), downsampling at the edge.
- **Seam:** the topology is configuration, not code. A user with an existing broker (e.g. their MQTT
  infrastructure) attaches it as one more node/bridge in the topology.

## P5. Rollout orchestration (the "what goes where, when")

- Target groups / tags / cohorts; staged and canary rollouts; pause, abort, auto-rollback on health signals.
- One model for three kinds of change: **OS/firmware (P6)**, **workloads (P5a)**, **configuration (P5b)**.
- Maintenance windows, dependencies between components, compatibility matrices.
- **Seam:** a declarative "desired state" document per device/group (GitOps-friendly).

### P5a. Workload (container) deployment on the edge

- Run and supervise containers/apps on constrained devices with limited RAM/flash.
- Declarative manifests, health checks, restart policies, resource limits.
- Works offline; converges once connectivity returns.
- **Seam:** OCI images + a manifest format (Compose, Kubernetes pod spec, Quadlet, Ankaios manifest).

### P5b. Configuration management

- Desired vs. reported config (device twin / shadow), drift detection.
- Layering: global → fleet → group → device overrides; secrets handling.
- **Seam:** versioned config documents (YAML/JSON + schema) delivered via P4 or as OCI artifacts.

## P6. OTA updates (device side)

- Atomic install, A/B or snapshot-based; automatic rollback on failed boot/health check.
- Signature verification and anti-rollback; secure boot chain integration.
- Delta updates; bandwidth and power awareness; MCU updates via a gateway.
- **Seam:** update bundle format + update server API (e.g. hawkBit DDI, Mender API, SUIT manifests).

## P7. Device agent

- The one process on the device that talks to the backend: reports inventory/state, applies
  desired state, dispatches commands, exposes local APIs to apps.
- Must stay small, robust, and itself updatable.
- **Seam:** a plugin interface per capability (update, workloads, config, telemetry), so third-party
  agents/clients (Mender client, RAUC, Kanto, Greengrass, ...) can be driven by it or replace it.

## P8. Ingestion infrastructure

- Bridge from device messaging into backend streams; routing and fan-out.
- Validation, schema registry, enrichment with device metadata.
- Storage: time series, event logs, raw data lake (Parquet/Iceberg).
- Stream processing / rules / alerting; exporting to customer systems.
- **Seam:** a stream API (Kafka protocol is the de-facto standard) + table formats.

## P9. Observability of the fleet itself

- Device health metrics, logs, traces, crash dumps, boot/update outcomes.
- Feeds rollout decisions in P5 (health-gated rollouts).
- **Seam:** OpenTelemetry (OTLP) everywhere.

## P10. Fleet management backend & UI

- Device inventory, search, grouping, tags; tenants, users, RBAC, audit log.
- API-first (everything the UI does is in the API), CLI, Terraform/GitOps integration.
- **Seam:** a public REST/gRPC API + webhooks/events.

## P11. Remote access & debugging

- Secure shell/port forwarding to devices behind NAT/CGNAT, without inbound ports.
- Session audit, just-in-time access.
- **Seam:** WireGuard/overlay network or tunnel over the existing messaging link.

## P12. Security & supply chain (cross-cutting)

- Signed artifacts (Sigstore/cosign, Notation), SBOMs (SPDX/CycloneDX), provenance (SLSA, in-toto).
- Compromise-resilient update metadata (TUF / Uptane).
- Vulnerability tracking per deployed version across the fleet.

## P13. Device classes (cross-cutting)

The same platform has to handle very different devices:

| Class | Example | OS | Implications |
|---|---|---|---|
| MCU | Cortex-M, ESP32 | Zephyr, FreeRTOS, bare metal | No containers, tiny protocols (MQTT-SN, CoAP/LwM2M), often via gateway |
| Small Linux | Cortex-A, 256 MB–1 GB RAM | Yocto/Buildroot | A/B OTA, lightweight container runtime |
| Edge server | x86/ARM, multi-GB | Linux distro / image-based OS | Full container orchestration, maybe k3s |

## P14. Regulation & compliance (cross-cutting)

- EU Cyber Resilience Act (CRA): security updates and vulnerability handling for products sold in the EU.
- Automotive: UNECE R156 / ISO 24089 (software update management systems).
- Radio Equipment Directive delegated act (EN 18031) for connected radio devices in the EU.
- Audit trail of what software was on which device when.

## Open design questions

1. **Single agent or orchestrated agents?** Write one fleetloom agent with plugins, or coordinate existing agents?
2. **Which protocol is the baseline?** The topology requirement (P4a) favours a protocol with native peer, routed and brokered modes (Zenoh is the strongest candidate). MQTT 5 then becomes a bridge for interoperability rather than the core.
3. **Everything as OCI?** Using one OCI registry for OS images, containers and config would simplify P3/P6/P5 a lot.
4. **Kubernetes on the edge or not?** k3s/KubeEdge reuse a big ecosystem but are heavy for small devices.
5. **Where does the "desired state" live?** Git (GitOps), a database behind an API, or both.
6. **MCU support scope:** first-class from day one, or via a gateway pattern later.
