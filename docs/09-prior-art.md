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
| MCUboot | Adopted (MCU bootloader) | Image signature + security counter checked end to end; swap, direct-XIP, serial recovery |
| mcumgr / SMP | Adopted (proxy → MCU updates) | Zephyr standard for upload/test/confirm over UART, USB, BLE |
| HSS/LMS (RFC 8554) | Adopted (planned, firmware signing) | Post-quantum, verification needs only SHA-256; stateful signing in the HSM |
| LoRaWAN FUOTA | Integrate | Multicast firmware updates for LoRaWAN devices behind an LNS |
| libcsuit / nRF Connect SDK SUIT | Evaluate | SUIT manifest implementations for the directive format |
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

## Hardware simulation and test

Direction (2026-10-18): existing tools where they do the job; weft builds only the chip register
models and `weft-boardchecker`. See the hardware test row in [04-decisions.md](04-decisions.md).

| Project | Decision | Reason |
|---|---|---|
| QEMU (virt machine) | Adopted (weft-sim; virt for arm64 guests, pc for x86-64) | Best-maintained machine, same arm64 architecture as the Pi; runs the real drivers against simulated chips. The raspi4b machine is rejected: no usable I2C/SPI controllers, adding them means C inside QEMU |
| Firecracker, libkrun (microVMs) | Rejected | Tiny and fast because they drop almost all devices: no virtio-i2c/spi/gpio, no generic vhost-user devices, KVM only (no arm64 guest on x86 CI runners) |
| Cloud Hypervisor, crosvm | Rejected (for now) | rust-vmm based with some vhost-user device types, but no I2C/GPIO support known; the weft backend is a plain vhost-user daemon, so it can move if they add generic vhost-user devices |
| QEMU microvm machine | Rejected | x86 only. Small footprint comes from the guest instead: QEMU virt with only the needed devices, direct kernel boot, minimal kernel config and a busybox initramfs, about 128 MB RAM |
| rust-vmm vhost-device (i2c, spi, gpio) | Deferred (possible Linux-only transport later) | Replaced by weft-sim's in-guest bus module, see Board simulation in [04-decisions.md](04-decisions.md). Was: Rust virtio backends for QEMU's virt machine; I2cDevice and SpiDevice traits accept a simulated device instead of host hardware, so weft's register models plug in without patching QEMU. Check that the target kernel has the virtio SPI driver. Host support (checked 2026-10-18): the rust-vmm vhost crates are Linux-only (epoll, eventfd; a POSIX port is in progress). QEMU supports vhost-user on macOS/BSD since 9.1 (memory-backend-shm), but only when built with --enable-vhost-user, which Homebrew's QEMU is not; QEMU refuses vhost-user on Windows |
| swtpm / ms-tpm-20-ref | Adopted (planned, CI) | Real TPM 2.0 logic in software; tests measured boot and key handling without the SLB9672. The SPI transport is only tested on hardware |
| Zephyr emulators (native_sim + emul) | Adopted (planned, if the co-MCU runs Zephyr) | Register-level I2C/SPI device emulators linked into host tests of real drivers; matches the MCUboot/mcumgr choice |
| Linux dsa_loop | Adopted (planned, network config tests) | Mock DSA switch for weft's switch configuration; no faithful KSZ9477 register model exists, so switch registers are tested on hardware |
| Linux gpio-sim | Adopted (planned, weft-sim GPIO and interrupts) | Upstream simulated GPIO chips configured through configfs; covers reset, ALERT and interrupt lines without weft kernel code |
| Rust for Linux | Rejected (for now, weft-sim module) | No I2C adapter or SPI controller abstractions yet, and many board kernels (Raspberry Pi OS) are built without Rust; revisit when both change |
| Buildroot | Evaluate (weft-sim guest image) | Standard embedded tool for a kernel plus busybox initramfs per architecture; alternative to building the minimal guest with Nix |
| Linux i2c-stub, i2c-slave-eeprom, netdevsim | Rejected | i2c-stub is SMBus only (cannot take the MCP4728's raw writes); the register models cover these cases |
| Renode (Antmicro) | Rejected | Strong whole-board simulator, but a second emulator next to QEMU; no BCM2711 model that boots Linux; native_sim covers the co-MCU |
| Wokwi (+ custom chips API) | Rejected | MCU-only simulation that overlaps native_sim; CI runner needs a paid token |
| roadtest (Axis, RFC 2022) | Ideas (adopted pattern) | Kernel driver tests under UML with Python device models; not merged upstream, but the pattern behind weft's register models; weft-sim applies the same idea under QEMU with Rust models |
| umockdev | Rejected | Records and replays sysfs and i2c-dev/spidev ioctls; redundant once real drivers run against simulated chips |
| KiCad + ngspice | Adopted (planned, analog front ends) | SPICE simulation of ADC inputs and DAC outputs inside the existing design tool; digital buses are out of scope |
| SimulIDE, Proteus VSM | Rejected | Schematic plus firmware co-simulation, but limited MCU support (Proteus is commercial); no Linux host |
| atopile | Rejected (for now) | Hardware as code with typed I2C/SPI interfaces; would replace schematic capture to get checks weft-boardchecker provides |
| SKiDL | Rejected (for now) | Python-defined netlists; same trade-off as atopile |
| KiBot | Adopted (planned, board CI) | Runs KiCad ERC/DRC and the IPC-2581 export in CI, then weft-boardchecker |
| dt-schema (dtbs_check) | Adopted (planned, overlay CI) | Validates overlays against kernel bindings; complements weft-boardchecker, which checks them against the PCB |
| IPC-2581 (rev B/C) | Adopted (weft-boardchecker input) | Open IPC XML standard with netlist, BOM and component placement in one file; exported by KiCad 8+, Altium, Cadence Allegro/OrCAD and Siemens Xpedition. Part numbers only appear if the export maps them |
| IPC-D-356A | Adopted (fallback input) | Bare-board test netlist that nearly every PCB tool exports; nets and pads only, combined with a CSV BOM |
| ODB++ | Rejected | Widely supported, but the spec is controlled by Siemens; IPC-2581 covers the same data openly |
| EDIF | Rejected | Old schematic interchange standard; inconsistent dialects across tools |
| Gerber X2/X3 attributes | Rejected (as checker input) | Carries net and component attributes, but rebuilding connectivity from copper is the wrong layer for this check |
| roxmltree, csv, toml, clap | Adopted (weft-boardchecker) | Permissive, widely used; XML, BOM, config and CLI parsing |
| fdt (crate) | Rejected | MPL-2.0, outside the permissive allow list |
| fdt-rs | Rejected | MIT, but an unsafe zero-copy API and no release since 2024; reading a DTB is about 150 lines in weft-boardchecker. `dtc` stays the tool for device tree source |
| SystemRDL (Accellera) | Evaluate (later, chip register maps) | Standard register description language with open tooling (PeakRDL). Not used yet: chip behaviour (command protocols, conversions) needs code anyway, and no Rust tooling exists; revisit when register-heavy chips (switches) get models |
| CMSIS-SVD | Rejected (for external chips) | Register descriptions for MCU peripherals only; no I2C/SPI peripheral chips |
| KiCad symbol libraries | Evaluate (pinout source for weft-parts) | Symbols name every pin; a generator could fill weft-parts pinouts instead of typing them from datasheets. Address rules are not in them. IPC-2581 exports carry pin numbers only, so the chip data must supply pin roles |
| Linux dt-bindings (YAML) | Integrate (source of truth for compatibles and properties) | Defines compatible strings and property names such as reset-gpios; weft-parts references the binding path and adds only pin knowledge |
| embedded-hal 1.0 | Adopted (weft-parts) | Standard Rust I2C trait; the simulated bus implements it, so Rust MCU drivers run against the chip models on the host |
| ads1x1x, mcp4728, eeprom24x (crates) | Adopted (test oracles in weft-parts) | Independent drivers for the modelled chips; the models must satisfy them, not only weft's own reading of the datasheets |

## Project infrastructure

Direction (2026-10-18): the project's own infrastructure (DNS, mail authentication, website
settings) is code in the private repository weft-terraform.

| Project | Decision | Reason |
|---|---|---|
| OpenTofu | Adopted (weft-terraform) | Open-source fork of Terraform under the Linux Foundation (MPL-2.0); same language and providers. Files use the `.tofu` extension |
| HashiCorp Terraform | Rejected | Business Source License since 2023, outside weft's licence policy |
| Cloudflare provider (cloudflare/cloudflare) | Adopted | Official provider for DNS, Email Routing and Workers settings |
| Cloudflare R2 (s3 backend) | Evaluate (state storage) | Keeps state off laptops and out of git; free tier covers it |

## Project signing and security contact

Direction under discussion (2026-10-18): one key per purpose, no single long-lived project key.

| Project | Decision | Reason |
|---|---|---|
| crates.io Trusted Publishing | Evaluate (crate releases) | Publishes from CI via short-lived OIDC tokens; no long-lived token to steal. Cargo does not verify signatures on crates, so a separate crate-signing key would protect nobody |
| Sigstore (cosign, GitHub artifact attestations) | Evaluate (release artifacts, SLSA provenance) | Keyless signing tied to the CI identity, logged in a public transparency log; matches D7 (signed releases with provenance) |
| gitsign | Ideas | Sigstore keyless commit signing; SSH signing is simpler for a small team |
| SSH commit and tag signing | Evaluate (per developer) | Supported by git and shown as Verified by GitHub; key can live on a hardware token |
| OpenPGP (security@ key) + Web Key Directory | Evaluate (security reports) | Expected by some reporters and referenced from security.txt; WKD publishes the key on theweft.org. Role key, not a personal key; Gmail cannot decrypt it, so it needs Thunderbird or similar |
| GitHub private vulnerability reporting | Evaluate (primary security channel) | Built-in private reports per repository, no keys for reporters to handle |
| security.txt (RFC 9116) | Evaluate | Standard place on theweft.org for the security contact, policy and encryption key |
| minisign / signify | Rejected (for now) | Simple and sound, but Sigstore covers releases without a key to guard |
| weft-verifier document keys | Separate | Device-facing signing (COSE, HSS/LMS in an HSM) is product trust, unrelated to release and email keys |
