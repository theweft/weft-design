# Prior art: what we adopted and why

Living list. Every project we evaluate gets a row: **Adopted** (we use it), **Ideas** (we take its
design, not its code), **Integrate** (supported through an adapter or plugin), **Rejected** (with reason).

## Full platforms

| Project | Decision | Reason |
|---|---|---|
| balena | Ideas | Closest to the vision for Linux (OS as container image, apps, OTA). Production assumes balenaCloud; openBalena is limited and AGPL; not decentralized |
| Mender | Integrate | Most mature open OTA stack; can be an OTA plugin. Central server model, no swappable layers |
| Red Hat Flight Control / Edge Manager | Ideas, Integrate | Fleet templates by labels, bootc devices. Central control plane, Linux only |
| Foundries.io FoundriesFactory | Ideas | TUF/Uptane + Compose apps on Yocto proves the model works. Commercial |
| Torizon (Toradex) | Ideas | Offline update bundles ("Lockboxes") as reference for our bundle format. Commercial |
| Project EVE (LF Edge) | Rejected | Strong security model, but a hypervisor OS, too heavy for small devices |
| Pantavisor | Rejected | Interesting container-based embedded system, overlaps our OS profile without swappable seams |
| Eclipse Kanto | Ideas | Modular edge agent building blocks; our agent stays smaller and plugin-based |
| thin-edge.io | Evaluate | Rust, Apache-2.0, plugin-based edge framework; closest device-side overlap. Pending deep evaluation |
| Edge Core | Evaluate | Self-hosted Linux fleet control plane with WireGuard mesh; licence to verify |
| FLECS | Integrate (via Margo) | App runtime + marketplace; open core only (console and marketplace closed); covers workloads only |
| Viam | Ideas | Topology-flexible (WebRTC peer-to-peer) robot fleets; proprietary cloud, robotics focus |
| AWS IoT + Greengrass, Azure IoT Edge | Integrate | Users "bring their own cloud"; vendor lock-in; Azure IoT Edge 1.5 LTS end of servicing Nov 2026 |
| Google Cloud IoT Core | Rejected | Retired 2023: cautionary tale for lock-in |
| ThingsBoard, OpenRemote | Rejected | Centralized IoT platforms; OpenRemote is AGPL |
| Golioth, Memfault, Particle | Ideas | MCU-class reference (OTA, observability). SaaS |
| NervesHub | Ideas | Good OTA UX reference; Elixir-specific |
| Eclipse IoT Packages (Cloud2Edge) | Ideas | "Glue existing tools" precedent (Hono, Ditto, hawkBit); appears stale since about 2020 |
| FIWARE, Eclipse Arrowhead | Ideas | Swappable-component architectures; not focused on OTA/fleets |
| United Manufacturing Hub | Evaluate | Open-source Unified Namespace stack (MQTT, Kafka, Benthos); licence and architecture to check |
| Siemens Industrial Edge | Ideas | Machine builder + plant operator app model on industrial PCs; proprietary, Siemens ecosystem (see docs/12) |
| Rancher Fleet, SUSE Edge, Azure Arc | Rejected (retail/IT edge) | GitOps for k3s/Kubernetes clusters per site; solves x86 store edge well, too heavy for embedded, central control plane |
| Formant, InOrbit | Ideas | Robot fleet operations (teleop, observability); SaaS, robotics only |
| Niagara Framework (Tridium) | Integrate | De-facto building automation platform; adapter target, proprietary |
| Vendor energy clouds (SMA, SOLARMAN, Solectria portal) | Ideas | Staged transfer vs. installation, per-device update order; closed, single-vendor |
| OCPP CSMS backends | Integrate | OCPP owns charging logic and its own firmware-update command; weft manages what's underneath |
| ChirpStack | Integrate | MIT LoRaWAN network server; runs as a weft app on gateways for offline farms and sites |
| Davra | Rejected | Commercial IoT platform (Azure IoT Central migration target); centralized |

## Standards and specifications

| Spec | Decision | Reason |
|---|---|---|
| Margo | Integrate | Interoperability for workload deployment; covers apps only, still in preview. Adapter, not core |
| TUF | Ideas (planned) | Roles, thresholds, root rotation, snapshot (mix-and-match) and timestamp (freeze attacks). Code (tough, go-tuf) needs std, JSON |
| Uptane | Ideas (planned) | Director = per-device instructions; partial verification for MCUs |
| SUIT (RFC 9019, 9124) | Adopt (MCU artifacts, planned) | CBOR + COSE update manifests for constrained devices |
| COSE (RFC 9052, 9053) | Adopted | Signature format; compact, algorithm-agile, MCU-friendly |
| CBOR (RFC 8949) + CDDL (RFC 8610) | Adopted | One deterministic encoding in the trusted path |
| Protobuf | Rejected (trusted path) | No canonical encoding, not self-describing; fine at the edges (OTLP, Sparkplug) |
| CloudEvents | Adopted (envelope attributes) | Neutral, transport bindings; we add signatures and correlation |
| Sparkplug B | Integrate | Industry standard, but MQTT/broker-bound, fixed namespace, no signatures |
| OPC UA (PubSub, open62541) | Integrate | Must-have industrial adapter |
| ISA-95 / Unified Namespace | Integrate | Location paths as projections, not primary names |
| JSON Merge Patch (RFC 7396) | Adopted | Config merge semantics |
| REUSE, SPDX | Adopted | Licence compliance per file and repository |

## Building blocks

| Project | Decision | Reason |
|---|---|---|
| Zenoh (+ zenoh-pico, MQTT plugin) | Adopted (default pub/sub) | Only mature option for peer-to-peer, routed and direct-to-cloud with one API; MQTT bridge |
| NATS | Integrate | Strong backend bus; needs a server, no true peer-to-peer |
| rustls (+ aws-lc-rs) | Adopted (planned, transport adapters) | Memory-safe TLS 1.3, raw public keys, hybrid ML-KEM key exchange; permissive |
| quinn | Adopted (planned, QUIC) | Rust QUIC, permissive; QUIC preferred for moving nodes and multi-link uplinks |
| MQTT over QUIC (EMQX) | Rejected (for now) | Not standardized; Zenoh QUIC covers the need |
| WebTransport | Evaluate (later) | Browser streams over HTTP/3; UDP caveat as QUIC |
| CoAP + OSCORE/EDHOC | Evaluate | Message-level (COSE) security that survives proxies and store and forward; candidate for MCU commands |
| wstunnel | Evaluate | Tunnels UDP overlays (WireGuard/Nebula) through TCP/WebSocket on 443 |
| MQTT brokers (Mosquitto, NanoMQ, HiveMQ CE) | Integrate | Interop via the Zenoh MQTT plugin and broker bridging |
| EMQX ≥ 5.9 | Rejected | Business Source License |
| DDS (Cyclone, Fast DDS) | Integrate | LAN and robotics; via Zenoh bridge |
| SWUpdate | Adopted (embedded OTA, preferred) | Flexible handlers, local web UI, GPL-2.0 is fine as a separate program |
| RAUC | Integrate | Clean A/B model; alternative plugin |
| OSTree / bootc | Adopted (image-mode profile) | OS as OCI image for larger devices; Yocto + OSTree is proven (Torizon, Foundries) |
| MCUboot | Adopted (MCU profile) | Standard secure bootloader with A/B slots |
| Podman + Quadlet | Adopted (default workloads) | Rootless, daemonless, systemd-native |
| Eclipse Ankaios | Integrate | Alternative workload orchestrator |
| Docker Compose | Integrate (importer) | Too large as the privileged executor's input; translated outside the trusted path |
| coset | Rejected | Pulls a second CBOR library (ciborium) and ~11k lines into the trusted path; COSE_Sign1 implemented directly (~150 lines) |
| minicbor | Adopted | no_std CBOR; Blue Oak licence (permissive, patent grant) |
| ed25519-dalek, p256, sha2 (RustCrypto) | Adopted | no_std, audited, permissive |
| ml-dsa (RustCrypto) | Planned | Post-quantum signatures; pre-1.0, needs review |
| Biscuit | Rejected (for now) | Second delegation system next to ours, Protobuf encoding, no post-quantum yet; fallback if conditions get richer |
| Cedar | Rejected (for now) | Second policy language; formal analysis is attractive for later |
| OPA / Rego, UCAN, macaroons | Rejected | General-purpose or less analyzable; our fixed constraint vocabulary suffices |
| Eclipse Ditto | Ideas | Twin model; optional central read model, not the source of truth |
| Kubernetes Server-Side Apply | Ideas | Field ownership, conflicts surface instead of silently resolving |
| Windows Group Policy, Puppet Hiera, CSS cascade, XACML | Ideas | Priority layers, enforced settings (locks), user-defined hierarchies, "only one applicable" |
| CRDTs (Automerge, Yjs) | Ideas | Multi-value register = our tie rule; CRDTs for shared editable data |
| Nebula | Adopted (overlay default, with WireGuard) | Certificate-based, no live controller, offline-friendly |
| OpenZiti | Integrate | Zero-trust per-service access; needs a controller |
| Headscale + Tailscale client | Integrate | Mature NAT traversal, self-hosted |
| NetBird server ≥ 0.53 | Rejected | AGPL |
| Rosenpass | Adopted (planned) | Post-quantum pre-shared keys for WireGuard |
| step-ca, OpenBao | Adopted | Local CA; OpenBao instead of BSL-licensed Vault |
| SeaweedFS | Adopted | S3-compatible storage; MinIO (AGPL, archived) and Garage (AGPL) rejected |
| Kafka, Bento, ClickHouse, QuestDB | Adopted (ingestion defaults) | Permissive; Redpanda (BSL) rejected |
| OpenTelemetry, Fluent Bit, VictoriaMetrics, Perses | Adopted | Permissive; Grafana, Loki, Tempo (AGPL) rejected |
| Apache PLC4X, Telegraf, EdgeX device services | Integrate | Field-protocol access for adapters |
| WAMR, LF Edge Ocre, Zephyr LLEXT | Integrate (MCU apps) | Optional dynamic apps on larger MCUs |
