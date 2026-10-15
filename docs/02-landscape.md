# Landscape: what already exists

A rough survey of tools, standards and papers for each problem in
[01-problem-map.md](01-problem-map.md). The aim is to find the building blocks to weave together
and the seams other projects have already standardised.

> **Status:** first pass (Oct 2026), written mostly from prior knowledge and spot-checked
> on the web. Check licences, project health and maintenance status before adopting anything.
> Items marked ⚠ have a known status change.

Legend for **Fit**: ★ strong candidate building block · ○ worth evaluating · ✕ reference only
(closed, too heavy, or overlapping scope).

---

## 1. Full platforms (prior art and competition)

These already try to do "everything". We should study their data models and APIs, and decide which
ones we integrate with and which we only learn from.

| Project | What it covers | Licence / model | Fit |
|---|---|---|---|
| **balena** (balenaOS, balenaEngine, openBalena) | OS + container deployment + OTA + remote access | OSS server (openBalena) + SaaS | ○ closest to our vision for Linux devices; openBalena is single-tenant and limited |
| **Mender** (Northern.tech) | OTA (OS + app), deployments, remote terminal, inventory | Apache-2.0 client and open-source server + enterprise tier | ★ OTA layer |
| **Foundries.io FoundriesFactory** (Qualcomm since 2024) | Yocto-based LmP OS, TUF/Uptane OTA, Docker Compose apps | Commercial, OSS components | ✕/○ good reference for TUF + Compose |
| **Torizon** (Toradex) | OS, OTA (Uptane-based), containers | Commercial + OSS parts | ✕ reference |
| **Red Hat Edge Manager / Flight Control** (`flightctl`) | Fleet templates, bootc image-mode OS, apps, config, TPM identity | OSS upstream (Flight Control) + Red Hat product | ★ very close to our model for Linux; evaluate deeply |
| **Project EVE** (LF Edge) | Edge OS + hypervisor, VMs/containers, controller API | Apache-2.0 | ○ strong security model, heavy for small devices |
| **Pantacor / Pantavisor** | Container-based embedded Linux system and updates | OSS + SaaS | ○ |
| **Eclipse Kanto** | Modular edge agent: container mgmt, update, file upload, local MQTT | EPL-2.0 | ★ device-agent building blocks |
| **AWS IoT Core + Greengrass v2** | Broker, shadows, jobs, edge components | Proprietary cloud, Greengrass nucleus OSS | ✕ integration target (users "bring their own AWS") |
| **Azure IoT Hub + IoT Edge** ⚠ | Hub, twins, edge modules | IoT Edge 1.5 LTS end of servicing Nov 2026; IoT Central retirement reported | ✕ migration opportunity |
| **Google Cloud IoT Core** ⚠ | Retired Aug 2023 | – | ✕ cautionary tale for vendor lock-in |
| **ThingsBoard** (+ Edge) | Device mgmt, rule engine, dashboards, MQTT/CoAP/LwM2M | Apache-2.0 CE + PE | ○ ingestion/visualisation reference |
| **OpenRemote** | IoT platform, assets, rules, UI | AGPL-3.0 | ○ |
| **Golioth**, **Memfault**, **Particle** | MCU-centric cloud: OTA, telemetry, observability | SaaS, OSS SDKs | ✕ reference for MCU class |
| **NervesHub** | OTA for Elixir Nerves devices | Apache-2.0 | ✕ reference (nice UX) |
| **Siemens Industrial Edge**, **Losant**, **Kaa** | Industrial/commercial platforms | Commercial | ✕ |

**Takeaway:** No open project covers the whole chain with swappable layers. balena and Flight Control
come closest for Linux devices, Mender is the most mature open OTA stack, and the MCU world is mostly SaaS.

---

## 2. Base OS / firmware build (P1)

| Tool | Notes | Fit |
|---|---|---|
| **Yocto Project / OpenEmbedded** | De-facto standard for embedded Linux; meta-layers exist for Mender, RAUC, SWUpdate, OSTree | ★ |
| **Buildroot** | Simpler alternative; supports RAUC/SWUpdate | ★ |
| **bootc / image-mode Linux** (Fedora IoT, CentOS/RHEL image mode) | The OS *is* an OCI image; updates are pulled from a registry | ★ aligns with "everything is OCI" |
| **OSTree / libostree**, **Ubuntu Core (snaps)**, **Flatcar** | Atomic, image-based Linux variants | ○ |
| **Zephyr RTOS**, **FreeRTOS**, **NuttX** | MCU class | ★ (Zephyr) |
| **MCUboot** | Secure bootloader for MCUs with A/B slots; used by Zephyr | ★ |
| **Isar**, **Debos**, **mkosi** | Debian/systemd-based image builders | ○ |

---

## 3. Device identity, provisioning, onboarding (P2)

| Tool / standard | Notes | Fit |
|---|---|---|
| **FIDO Device Onboard (FDO)** (LF Edge, `go-fdo`, Intel reference) | Late-binding, zero-touch onboarding to an owner | ★ onboarding seam |
| **EST (RFC 7030)**, **ACME (RFC 8555)**, SCEP | Certificate enrollment | ★ EST for devices |
| **TPM 2.0** (`tpm2-tss`), **Parsec** (CNCF), secure elements (ATECC, SE050) | Hardware-bound keys | ★ |
| **DICE** (TCG) | Identity for MCUs without a TPM | ○ |
| **smallstep step-ca**, **OpenBao** (Vault fork) PKI, **EJBCA** | Private CA | ★ step-ca / OpenBao |
| **SPIFFE / SPIRE** | Workload identity; has TPM node attestation | ○ for workloads on the device |
| **Keylime** | Remote attestation (TPM/IMA) | ○ |
| **Matter** device attestation | Consumer IoT reference | ✕ |

---

## 4. Artifact storage & distribution (P3)

| Tool / standard | Notes | Fit |
|---|---|---|
| **OCI distribution spec + OCI artifacts** (ORAS) | One registry for containers, OS images, firmware and config bundles | ★ core seam |
| **Harbor**, **Zot**, **distribution/registry** | Registries; Zot is small and OCI-native, good for edge caching | ★ Zot, ○ Harbor |
| **Spegel**, **Dragonfly**, **Kraken** | P2P / local mirroring of images | ○ fits the peer topology idea |
| **S3-compatible storage**: SeaweedFS, Garage, Ceph RGW | Large blobs | ○ ⚠ MinIO community edition is archived (2026); avoid |
| **casync**, **zchunk**, **bsdiff**, Mender delta, RAUC adaptive updates, SWUpdate delta | Delta and chunked updates | ○ |

---

## 5. Device ↔ cloud communication with flexible topology (P4, P4a)

This is the layer where the topology requirement matters most. It breaks down into four questions:

1. Does the protocol support **peer-to-peer** with no infrastructure?
2. Can it **route over multiple hops** (device → peer/gateway → cloud)?
3. Can it connect **directly to the cloud**?
4. Does it keep working when **partitioned**?

| Technology | Peer↔peer | Multi-hop / relay | Direct to cloud | Partition tolerance | Notes | Fit |
|---|---|---|---|---|---|---|
| **Eclipse Zenoh** | ✅ peer mode, multicast/gossip scouting | ✅ routers form a routed mesh | ✅ client → router | ✅ local mesh continues; storages/queries | Unifies pub/sub, queries and storage; runs on MCUs (zenoh-pico); plugins for MQTT, DDS, ROS 2 (`rmw_zenoh`), REST | ★ **best match for P4a** |
| **NATS** (+ JetStream) | ✗ (needs a server; server can run on the device) | ✅ leaf nodes, clusters, superclusters | ✅ | ✅ leaf nodes buffer with JetStream | Very good ops story and multi-tenancy (accounts); has an MQTT gateway | ★ strong alternative / backend bus |
| **MQTT 5** (Mosquitto, EMQX, HiveMQ, NanoMQ, VerneMQ) | ✗ broker-centric | ◐ broker-to-broker bridges, configured per hop | ✅ | ◐ local broker + bridge queue | Universal interop; Sparkplug B adds state/birth/death semantics | ★ as interop bridge, not as core |
| **DDS** (Cyclone DDS, Fast DDS) / ROS 2 | ✅ native, LAN discovery | ◐ via routing services / Zenoh bridge | ✗ poor over WAN | ✅ on LAN | Real-time, robotics, automotive | ○ via Zenoh bridge |
| **MQTT-SN**, **CoAP** + **LwM2M** (Leshan, Wakaama, Anjay) | ◐ | ◐ via gateways | ✅ | ◐ | Constrained/MCU devices; LwM2M also has a device-management object model | ○ MCU class |
| **libp2p** (gossipsub) | ✅ | ✅ | ◐ | ✅ | Strong NAT traversal, but heavy and blockchain-centric ecosystem | ✕/○ |
| **Eclipse uProtocol** (Eclipse SDV) | – | – | – | – | Transport-agnostic abstraction (Zenoh, MQTT, SOME/IP transports) | ○ reference for our API seam |
| **Kafka**, **Pulsar**, **RabbitMQ**, **AMQP** | ✗ | ✗ | ◐ | ✗ | Backend-side, not for devices | backend only (§9) |
| **Thread / BLE mesh / Wi-Fi mesh / LoRaWAN** (ChirpStack) | link layer | link layer | via border router | – | Transport underneath, not pub/sub | ○ |

**Takeaway:** Zenoh is the only mature option that natively covers peer, routed multi-hop and
direct-to-cloud with the same API, and it has bridges to MQTT, DDS and ROS 2. That makes it a strong
candidate for the core bus. NATS is a good fit for the backend, and MQTT stays as the
interoperability seam for users who already run a broker.

**Contracts and payloads on top of the transport:**

| Standard | Use |
|---|---|
| **CloudEvents** | Common message envelope (id, source, type, time) across transports |
| **AsyncAPI** | Machine-readable topic/channel contracts, code generation |
| **Sparkplug B** (Eclipse Tahu) | State semantics on MQTT (birth/death certificates) |
| **Protobuf / CBOR / FlatBuffers** + a schema registry | Compact, typed payloads |
| **W3C Web of Things Thing Description**, **COVESA VSS** (Eclipse Kuksa), **LwM2M objects**, **OPC UA** information models | Describing device capabilities (sensors/actions) |

**End-to-end security across untrusted hops:** Zenoh has ACLs and TLS/QUIC per link. End-to-end
signing of payloads would need COSE/OSCORE-style object security or our own envelope signatures.
This is an open gap to research.

---

## 6. Workloads on the edge (P5a)

| Tool | Notes | Fit |
|---|---|---|
| **Podman + Quadlet** (systemd units) | Daemonless, rootless; declarative via systemd; no extra agent | ★ Linux default |
| **Docker / Moby + Compose**, **balenaEngine** | Familiar; Compose is the de-facto app manifest for edge (balena, Foundries) | ★ Compose as manifest format |
| **containerd / nerdctl**, **crun**, **youki** | Lower-level runtimes | ○ |
| **Eclipse Ankaios** | Lightweight workload orchestrator for automotive/embedded, Podman-based, declarative manifests | ★ evaluate |
| **Eclipse BlueChi** | Multi-node systemd unit controller | ○ |
| **Eclipse Kanto container management** | Containerd-based edge agent | ○ |
| **k3s**, **KubeEdge**, **OpenYurt**, **MicroShift**, **k0s** | Kubernetes on edge; KubeEdge handles offline nodes | ○ for edge servers only |
| **Eclipse Symphony** | Orchestrator across heterogeneous toolchains | ○ reference |
| **WasmEdge**, **wasmCloud**, **WAMR** | WebAssembly workloads, also on MCUs | ○ future option |

---

## 7. Configuration management and rollouts (P5, P5b)

| Tool / pattern | Notes | Fit |
|---|---|---|
| **Device twin / shadow** pattern (AWS shadows, Azure twins, Eclipse Ditto) | Desired vs. reported state | ★ pattern; **Eclipse Ditto** as an open twin service |
| **GitOps** (Flux, Argo CD, Rancher Fleet) | Desired state in Git; Rancher Fleet targets many clusters | ★ pattern |
| **Flight Control fleet templates** | Label-based templating of device specs | ★ reference model |
| **CUE**, **Jsonnet**, **KCL**, JSON Schema | Typed, layered config | ○ |
| **Ansible (pull)**, **Salt**, **Puppet**, **CFEngine** | Classic config management; heavy for small devices | ✕/○ |
| **hawkBit rollouts**, **Mender deployments**, **Argo Rollouts / Flagger** | Staged, canary and health-gated rollouts | ★ hawkBit model, ○ others as reference |
| **OpenFeature** | Feature flags as a lightweight form of config | ○ |

---

## 8. OTA updates on the device (P6)

| Tool | Notes | Fit |
|---|---|---|
| **RAUC** | A/B and adaptive (delta) updates, signed bundles, strong Yocto support | ★ |
| **SWUpdate** | Very flexible; hawkBit integration; delta support | ★ |
| **Mender client** | Full client including state scripts and app updates | ★ |
| **OSTree / bootc** | File-system-level atomic updates from a repo or OCI registry | ★ |
| **Eclipse hawkBit** | Update *server* with DDI (device) and DMF (AMQP) APIs; tool-agnostic | ★ server seam |
| **Uptane** (`aktualizr`, `aktualizr-lite`) | Automotive-grade compromise-resilient update framework built on TUF | ○ |
| **TUF** (`python-tuf`, `go-tuf`, `tough`) | Secure update metadata | ★ for repository metadata |
| **MCUboot + mcumgr / SMP** (Zephyr) | MCU firmware updates, also over BLE | ★ MCU class |
| **SUIT manifests** (IETF, RFC 9019 / RFC 9124) | Standard update manifest for constrained devices | ○ |

---

## 9. Ingestion infrastructure (P8)

| Area | Options | Fit |
|---|---|---|
| **Stream / log** | Apache Kafka, **Redpanda**, NATS JetStream, Apache Pulsar | ★ Kafka protocol as the seam |
| **Pipelines / connectors** | Redpanda Connect (ex-Benthos) or its OSS fork **Bento**, **Vector**, **Telegraf**, Apache NiFi, Node-RED, Kafka Connect | ★ Bento/Vector |
| **Stream processing** | Apache Flink, **RisingWave**, Arroyo, Materialize, Bytewax | ○ |
| **Time series** | **TimescaleDB**, **QuestDB**, InfluxDB 3, VictoriaMetrics, Apache IoTDB, TDengine | ★ Timescale or QuestDB |
| **Analytics / lake** | **ClickHouse**, Apache Parquet + **Apache Iceberg**, DuckDB | ★ |
| **Rules / alerting** | EMQX rule engine, ThingsBoard rule chains, Grafana alerting | ○ |

---

## 10. Fleet observability (P9)

| Tool | Notes | Fit |
|---|---|---|
| **OpenTelemetry** (Collector, OTLP, OTel Arrow) | Standard for metrics, logs, traces | ★ seam |
| **Fluent Bit**, **Vector**, **Grafana Alloy** | Small agents for devices | ★ |
| **Prometheus** (remote write), **VictoriaMetrics**, **Loki**, **Grafana** | Backend and dashboards | ★ |
| **Memfault**, **Sentry**, **Crashpad / Breakpad** | Crash and coredump collection | ○ |

---

## 11. Remote access (P11)

| Tool | Notes | Fit |
|---|---|---|
| **WireGuard** + **Headscale** (OSS Tailscale control plane), **NetBird** | Overlay networks with NAT traversal | ★ |
| **OpenZiti** | Zero-trust overlay, embeddable SDKs | ○ |
| **Teleport**, **Mender remote terminal**, balena tunnels | Audited sessions | ○ |
| **frp**, **rathole**, **Zenoh-based tunnel** | Tunnelling through existing links | ○ |

---

## 12. Fleet backend / UI (P10)

Most of this will be our own glue: an API-first service over inventory, groups, desired state and
rollouts. References worth reading: the hawkBit management API, the Mender API, the Flight Control API
(Kubernetes-style resources), the Eclipse Ditto API, and the LwM2M object model. Identity/RBAC options
are **Keycloak**, **Zitadel** and **Ory**, with **OpenFGA** or **SpiceDB** for fine-grained authorization.

---

## 13. Security, supply chain and compliance (P12, P14)

| Item | Notes |
|---|---|
| **Sigstore / cosign**, **Notation** (Notary v2) | Signing OCI artifacts |
| **in-toto**, **SLSA** | Build provenance |
| **SPDX**, **CycloneDX**, Yocto's built-in SPDX output | SBOMs per image |
| **TUF**, **Uptane** (IEEE-ISTO 6100) | Compromise-resilient update metadata |
| **OSV**, **Dependency-Track** | Map SBOMs to vulnerabilities across the fleet |
| **EU Cyber Resilience Act** | Reporting duties start Sep 2026, main obligations Dec 2027; pushes demand for update + SBOM + vulnerability handling |
| **UNECE R155/R156**, **ISO/SAE 21434**, **ISO 24089** | Automotive cybersecurity and software update management |
| **EN 18031** (Radio Equipment Directive) | Cybersecurity for connected radio devices in the EU |
| **IEC 62443** | Industrial security |

---

## 14. Papers and reading list

Citations are from memory and need checking before we rely on them.

**OTA updates**
- Kuppusamy et al., *Uptane: Securing Software Updates for Automobiles*, ESCAR Europe 2016.
- Samuel, Mathewson, Cappos, Dingledine, *Survivable Key Compromise in Software Update Systems*, ACM CCS 2010 (the TUF paper).
- Zandberg, Schleiser, Acosta, Tschofenig, Baccelli, *Secure Firmware Updates for Constrained IoT Devices Using Open Standards: A Reality Check*, IEEE Access 2019.
- Bauwens, Ruckebusch, Giannoulis, Moerman, De Poorter, *Over-the-Air Software Updates in the Internet of Things: An Overview of Key Principles*, IEEE Communications Magazine 2020.
- Arakadakis, Charalampidis, Makrogiannakis, Fragkiadakis, *Firmware Over-the-Air Programming Techniques for IoT Networks: A Survey*, ACM Computing Surveys 2021.
- IETF RFC 9019 (SUIT architecture) and RFC 9124 (SUIT information model).

**Messaging and topology**
- Liang, Yuan, Lin, *A Performance Study on the Throughput and Latency of Zenoh, MQTT, Kafka, and DDS*, arXiv:2303.09419, 2023. Zenoh came out ahead; critics note the DDS configuration was unfavourable.
- Zhang et al., ROS 2 middleware comparison over Ethernet, Wi-Fi and 4G, arXiv:2309.07496. Cyclone DDS did better on Ethernet, Zenoh on Wi-Fi and 4G.
- Corsaro et al., Zenoh design papers and the zenoh.io blog on the protocol and routing.
- OASIS MQTT 5.0 specification; Eclipse Sparkplug 3.0 specification.

**Edge orchestration**
- Xiong, Sun, Xing, Huang, *Extend Cloud to Edge with KubeEdge*, IEEE/ACM SEC 2018.
- Surveys on container orchestration at the edge (e.g. in ACM Computing Surveys and IEEE Access) are worth a targeted search.

---

## 15. First impressions for fleetloom

1. **Messaging core:** Zenoh as the topology-flexible bus, with an MQTT 5 bridge for interop and NATS or Kafka at the backend.
2. **Artifacts:** an OCI registry for everything (containers, bootc OS images, RAUC bundles, config), signed with cosign, with TUF metadata on top.
3. **Device side:** our own small agent that drives existing tools (RAUC/SWUpdate/Mender for OS, Podman/Quadlet or Ankaios for workloads) through plugins.
4. **Backend:** a desired-state API modelled on the twin pattern and on Flight Control's fleet templates, with hawkBit-style rollout control and health gating from OpenTelemetry data.
5. **Ingestion:** a Kafka-protocol bus, Bento or Vector pipelines, and TimescaleDB/ClickHouse plus Iceberg storage.
6. **Evaluate in depth next:** Flight Control, Mender, Eclipse Kanto, Eclipse Ankaios, Zenoh (including zenoh-pico and its security model), hawkBit.

### Sources spot-checked for this pass
- [Microsoft lifecycle: products ending support in 2026](https://learn.microsoft.com/en-gb/lifecycle/end-of-support/end-of-support-2026) (Azure IoT Edge 1.5 LTS)
- [The Register on Azure IoT Central](https://www.theregister.com/2024/02/15/microsoft_retires_azure_iot_central/)
- [Red Hat Edge Manager: managing device fleets](https://docs.redhat.com/en/documentation/red_hat_edge_manager/1.0/html/managing_device_fleets/device-fleets)
- [Eclipse Symphony project page](https://projects.eclipse.org/projects/iot.symphony/governance)
- [InfoQ: MinIO S3 API alternatives](https://infoq.com/news/2025/12/minio-s3-api-alternatives/)
- [arXiv 2303.09419: Zenoh vs MQTT, Kafka and DDS](https://arxiv.org/abs/2303.09419)
- [arXiv 2309.07496: ROS 2 middleware comparison](https://arxiv.org/html/2309.07496v4)
