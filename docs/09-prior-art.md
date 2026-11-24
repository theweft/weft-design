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
| StrictDoc | Adopted (requirements as code) | Apache-2.0; text requirements with source traceability (Rust parser), HTML and ReqIF export; coverage gate is our own test |
| Sphinx-needs | Rejected | Flexible, but needs the Sphinx/reStructuredText stack; our docs are Markdown |
| OpenFastTrace | Rejected | Simple tag syntax, but Java and GPL-3.0 |
| Doorstop | Rejected | LGPL-3.0, older; StrictDoc grew out of it |
| ReqIF | Adopted (export format) | Exchange format that customers' and assessors' requirements tools import |

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
| Renode (Antmicro) | Rejected for weft-sim; Evaluate (later, MCU firmware in a CM5 twin) | MIT. Strong whole-board simulator, but a second emulator next to QEMU and no BCM2711/BCM2712 model that boots Linux, so not for weft-sim. native_sim covers the co-MCU only when a host build is enough; running the exact MCU firmware binary needs an instruction-set emulator, and Renode has the widest peripheral coverage plus built-in multi-machine time sync. Peripheral models are C#, so weft-parts models would be reached over a socket |
| renode-stm32u5 (60north) | Evaluate (with Renode) | Community STM32U575 platform, peripheral models and tests for Renode 1.17 (Nucleo-U575ZI-Q, which is in the lab plan); upstreaming started (renode#1041, 2026-10) |
| renode-rp2040 | Evaluate (with Renode) | Community RP2040 for Renode, including PIO; several instances connect through GPIO. RP2350 support not known |
| QEMU RP2040 machine (RFC v3, 2026-09) | Watch | Pico/RP2040 machine posted to qemu-arm, not merged |
| QEMU TCG with `-icount` | Evaluate (later, CM5 twin) | Guest time from executed instructions (fixed time per instruction), fully virtual and deterministic with single-threaded TCG; skips idle time. Not cycle-accurate. Rules out KVM/HVF, so not for weft-sim |
| hatter6822/rpi5_machine | Evaluate (later, CM5 twin) | Out-of-tree QEMU `raspi5b` machine (BCM2712: 4× A76, GIC-400, UART10, PSCI, full memory map) on QEMU 11.1.1, written for upstream submission; a stock Raspberry Pi OS kernel boots to the root mount. No RP1, eMMC or firmware emulation yet. GPL-2.0-or-later like QEMU (checked 2026-11-24): acceptable for a test tool, not for the core |
| jordanhubbard/uConsole (RP1 model) | Ideas | QEMU fork with Pi 5 boot; PR #14 (2026-11) models RP1 as a PCI endpoint with 61 MSI-X vectors. Reference for an RP1 model |
| jlaportebot/pi5-emulator | Rejected | Pi 5 QEMU patches with custom MMIO devices for one bare-metal hobby OS; not a generic machine |
| schlae/cm5-reveng | Ideas (reference, measurements only) | Reverse-engineered CM5 (KiCad schematic and 10-layer layout), CC BY-SA 4.0, so no files are copied into weft (share-alike). Names the datasheet's "reserved" test points (internal rails, the DA9091 PMIC's I2C bus and interrupt) and puts every test point at the datasheet coordinates as a 1.0 mm pad; used for the CM5 lab carrier's pogo pins (docs/22) |
| QEMU vfio-user client | Evaluate (later, CM5 twin) | In QEMU since 10.1: a PCI device implemented in a separate process over a UNIX socket (libvfio-user). Would allow RP1 in Rust, but then Ethernet and USB must be written too; preferred: RP1 in QEMU, only low-speed controllers forwarded |
| Remote Port (AMD/Xilinx; upstream series v2, 2026-09) | Evaluate (later, hub time sync) | Socket protocol between QEMU and external simulators: memory transactions, interrupts and quantum-based time sync. Carries no UART/I2C/SPI, so a hub adds those |
| SystemC TLM-2.0, libsystemctlm-soc | Rejected (for now); pattern adopted | Apache-2.0 standard for loosely and approximately timed models, bridged to QEMU through Remote Port. C++ and a large framework next to Rust models; weft adopts the loosely timed pattern instead (time-aware models, the caller owns time) |
| NeXosim (formerly Asynchronix) | Evaluate (later, hub scheduler) | Rust discrete-event simulator: models connected by ports, deterministic, as fast as possible or locked to real time. Possible host for the hub's scheduler; MIT or Apache-2.0, version 1.0 (checked 2026-11-24); maintenance activity not yet checked |
| gem5 | Rejected | Cycle-level simulator for architecture research; Linux boots take hours and there is no BCM2712 or RP1 model |
| Wokwi (+ custom chips API) | Rejected | MCU-only simulation that overlaps native_sim; CI runner needs a paid token |
| roadtest (Axis, RFC 2022) | Ideas (adopted pattern) | Kernel driver tests under UML with Python device models; not merged upstream, but the pattern behind weft's register models; weft-sim applies the same idea under QEMU with Rust models |
| umockdev | Rejected | Records and replays sysfs and i2c-dev/spidev ioctls; redundant once real drivers run against simulated chips |
| KiCad + ngspice | Adopted (planned, analog front ends) | SPICE simulation of ADC inputs and DAC outputs inside the existing design tool; digital buses are out of scope |
| SimulIDE, Proteus VSM | Rejected | Schematic plus firmware co-simulation, but limited MCU support (Proteus is commercial); no Linux host |
| tscircuit | Evaluate (AI-first schematic as code) | MIT, TypeScript/React, active (checked 2026-11-24); exports KiCad projects and netlists, has an ngspice (WebAssembly) simulation, visual diffs in pull requests and a Claude Code skill. Its own autorouter is beta (clearance violations reported), so layout stays in KiCad. Candidate for designing boards with an AI agent, verified by weft's checks |
| atopile | Watch | Hardware as code (`.ato`), MIT core with constraint-based part selection and KiCad layout reuse. Checked 2026-11-24: the CLI is announced as ending with 0.15.8 in favour of a hosted app, newer modules (autolayout, agent gateway) ship in the wheel but not in the public repo, telemetry on by default, Python 3.14 only. Too uncertain to build on now |
| SKiDL | Evaluate (fallback for schematic as code) | MIT, Python; writes KiCad netlists and nothing else, so it is small and stable; no layout, schematic drawing experimental |
| Quilter, DeepPCB | Evaluate (later, layout of larger boards) | Commercial cloud layout on KiCad files using reinforcement learning against physics or design rules, not LLMs; useful once boards outgrow manual or Freerouting layout |
| Flux.ai | Rejected | Commercial browser EDA with an AI copilot; designs live in its cloud, not in git |
| Freerouting | Evaluate (routing simple boards) | GPL-3.0 Java autorouter, run on KiCad's DSN export as an external tool (not linked into weft) |
| KiCad MCP servers, KiCad IPC API | Evaluate (agent-driven placement) | Let an agent read and edit a KiCad board; KiCad 10 still has no schematic API, so schematics come from code instead |
| KiBot | Adopted (planned, board CI) | Runs KiCad ERC/DRC and the IPC-2581 export in CI, then weft-boardchecker |
| dt-schema (dtbs_check) | Adopted (planned, overlay CI) | Validates overlays against kernel bindings; complements weft-boardchecker, which checks them against the PCB |
| IPC-2581 (rev B/C) | Adopted (weft-boardchecker input) | Open IPC XML standard with netlist, BOM and component placement in one file; exported by KiCad 8+, Altium, Cadence Allegro/OrCAD and Siemens Xpedition. Part numbers only appear if the export maps them |
| IPC-D-356A | Adopted (fallback input) | Bare-board test netlist that nearly every PCB tool exports; nets and pads only, combined with a CSV BOM |
| KiCad netlist (`kicad-cli sch export netlist`) | Adopted (planned, schematic-level simulation input) | KiCad's own S-expression/XML netlist: every symbol field, pin names and pin electrical types, hierarchical net names; simulation before layout. KiCad only, so IPC-2581 stays the tool-neutral input. See [21-pins-and-nets.md](21-pins-and-nets.md) |
| KiCad `Sim.*` symbol fields | Ideas (adopted pattern) | KiCad's convention for attaching ngspice models to symbols; weft would use an optional `Weft.Model` field the same way, normally unneeded because models come from the part number |
| SPICE netlist export | Adopted (planned, analog subcircuits to ngspice) | Passives with values and device models; not a source for the digital board model |
| IBIS (ANSI/EIA-656) | Evaluate (pin electrical data for weft-parts) | Vendor I/O buffer models: VIH/VIL, drive strength, thresholds; could fill weft-parts pin domains instead of typing them from datasheets |
| VHDL-AMS (IEEE 1076.1), Verilog-AMS | Rejected | Standard mixed-signal behavioural model languages; heavy toolchains and no Linux-driver or emulator integration; weft-parts models stay in Rust |
| WireViz | Rejected | Documents cables and harnesses from YAML, but GPL-3.0 and no electrical semantics; weft would use a small system file that mates connectors instead |
| LXA TAC (Linux Automation) | Ideas (lab carrier supervisor) | Commercial test automation controller: DUT power switch with voltage and current measurement, UART, USB host/device, GPIO, labgrid exporter; software open (tacd GPL-2.0, Yocto layer MIT), hardware not. The CM5 lab carrier puts the same functions on the carrier (docs/22) |
| VIBE2051/RPI-CM5-Carrier-Board, ShawnHymel/rpi-cm4-carrier-template | Ideas (reference) | Open KiCad CM5 carriers with wide GPIO access, and a CM4/CM5 carrier template; none exposes every pin or adds a supervisor |
| Raspberry Pi CM5 IO Board design files | Adopted (planned, reference carrier for a CM5 twin) | Official KiCad project (rev 2, KiCad 8/9) on the Raspberry Pi product information portal; exports checked with kicad-cli 10.0.8 (docs/21 §6). Raspberry Pi's licensing page grants use, modification and distribution of hosted design files for any purpose; bundled 3D models stay under their makers' terms |
| USB/IP | Evaluate (USB in a CM5 twin) | Network transport for USB at transfer level; QEMU and Renode can attach devices over it, so no USB signals at pin level |
| QEMU and Renode CAN models | Evaluate (CAN in a CM5 twin) | Both emulate CAN controllers on a virtual bus; weft would add arbitration timing and transceivers at net level |
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

## Dev environment and end-to-end tests

Decided 2026-10-20 where marked Adopted (see docs/04, Dev environment and tests). Fault-heavy end-to-end tests (partitions, power loss, clock jumps) need a harness; dev-loop tools only start services.

| Project | Decision | Reason |
|---|---|---|
| Tilt | Adopted (planned, dev loop) | Apache-2.0, maintained; watch/rebuild/restart many services with a UI on Linux, macOS and Windows; multi-repo Tiltfiles. Compose backend first, Kubernetes profile later. Starts services but cannot inject faults or assert behaviour, so tests live elsewhere |
| devenv + process-compose (+ services-flake) | Rejected (as main dev loop) | Apache-2.0 / MIT and Nix-native, but Nix runs on Windows only via WSL2; Nix stays optional |
| nix-community/cache-nix-action | Adopted (CI) | MIT; keeps the Nix store in GitHub's Actions cache, no external account (Cachix would need one) |
| DeterminateSystems/magic-nix-cache-action | Rejected (for now) | Same purpose; cache-nix-action uses GitHub's cache directly |
| systemd service sandboxing | Adopted (weft-agent decider) | NoNewPrivileges, PrivateNetwork, ProtectSystem=strict, RestrictAddressFamilies, SystemCallFilter, empty CapabilityBoundingSet: what the OS already has (docs/07 §2) |
| Landlock (landlock crate) | Evaluate (later) | In-process filesystem sandbox for the decider, independent of systemd; MIT/Apache-2.0 |
| SO_PEERCRED (rustix) | Evaluate (later) | Check the decider's uid on the executor socket; std's `peer_cred` is unstable, rustix would be a new dependency |
| Docker Compose | Adopted (planned, Tilt backend) | Simplest orchestration of the dev services; same OCI images as a later Helm chart |
| Skaffold, Garden, DevSpace | Rejected | Kubernetes-centred dev loops; weft's backend does not need Kubernetes |
| NixOS VM tests (runNixOSTest) | Adopted (weft-agent L1 end-to-end test) | Real systemd units, real disks and a real power cut (`crash()`); slim VM with `profiles/minimal.nix` boots in about 13 s; the driver type-checks the Python test script. Linux with KVM (tier 2) |
| Containerlab | Evaluate (topology e2e) | BSD-3; topologies of containers/VMs from YAML with netem impairments; good for T1–T7 |
| CORE (Common Open Research Emulator) | Rejected (for now) | BSD-2 network emulator; Containerlab is more active |
| Shadow | Evaluate (later, system scale) | BSD-style; runs unmodified binaries (zenohd, mosquitto, agent) on a simulated network with simulated time, deterministic; Linux only, some syscalls unsupported. For real-protocol tests with hundreds of nodes |
| turmoil | Adopted (weft-agent simulations) | MIT, tokio; simulated hosts, TCP/UDP, partitions and time in one process, seeded; runs in `cargo test` on all platforms. Only for weft's own code behind the Transport and clock abstractions; cannot run zenohd or other binaries. Its simulated filesystem (`unstable-fs`) also tests power loss |
| madsim | Rejected (for now) | Apache-2.0 deterministic simulator; less active than turmoil |
| LazyFS | Evaluate (later, real filesystems) | FUSE filesystem that loses unsynced writes on command. For now turmoil's simulated filesystem (`unstable-fs`) tests the persistence protocol deterministically, including power cuts in the middle of a save; LazyFS would add real filesystem behaviour. Licence to verify |
| dm-log-writes / dm-flakey | Evaluate (crash consistency) | Kernel device-mapper targets for replaying writes and simulating failing disks |
| Toxiproxy, Pumba | Evaluate (fault injection) | MIT / Apache-2.0; TCP faults and container netem/kill/pause |
| testcontainers-rs | Evaluate (integration tests) | Apache-2.0; brokers, registries and databases from Rust tests |
| Jepsen | Ideas | EPL-1.0, Clojure; the reference for partition and clock-skew testing of distributed systems; we adopt its fault model |
| labgrid | Evaluate (real hardware lab) | LGPL-2.1 (to verify), Python/pytest; controls power, serial and SD muxing of real boards (docs/17) |
| LAVA | Rejected | AGPL-3.0 board farm; heavier than labgrid for one developer |
| Renode | See above | Rejected for weft-sim, evaluated for MCU firmware in a CM5 twin; could also serve networks of MCUs (M1/M2 fleets) |
| kind, k3d, minikube | Evaluate (later, Kubernetes profile) | Apache-2.0 / MIT / Apache-2.0 local clusters; k3d runs k3s, the distribution edge sites use |
| Rancher Desktop | Adopted (recommended runtime) | Apache-2.0; containers plus optional k3s on Linux, macOS and Windows |
| Podman Desktop | Adopted (recommended runtime) | Apache-2.0; daemonless containers on all three platforms |
| Docker Desktop | Integrate | Works, but needs a paid subscription in larger companies |
| Lima, WSL2 | Adopted (tier 2 on macOS/Windows) | Linux VMs for the Linux-only tests |
| Chaos Mesh, LitmusChaos | Evaluate (later, with the Kubernetes profile) | Kubernetes-native partitions, delay, clock skew (TimeChaos), IO faults |
| KubeVirt | Rejected | VMs as Kubernetes pods; too heavy for simulated devices |

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
