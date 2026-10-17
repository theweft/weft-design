# Licensing policy and permissive stack

Goal: a stack that anyone can ship in a product, lock down with secure boot, and run as a
service, without copyleft obligations beyond simple source offers.

> Not legal advice. Licenses change (several did in 2025–2026); check the licence of the exact
> version adopted, and automate the check in CI.

## Why it matters where the code runs

- **On the device, code is distributed.** GPLv3 and LGPLv3 require "installation information": users
  must be able to install modified versions. That clashes with locked-down, secure-boot devices
  (anti-tivoization). GPLv2 has no such clause; it only requires providing source.
- **On the backend, code is usually not distributed.** GPL does not trigger for a hosted service, but
  **AGPL does**: offering a modified version over a network requires publishing the source.
- **Source-available licences are not open source:** BSL, SSPL, Elastic License, Timescale License,
  custom vendor licences. They typically forbid offering the software as a competing service or limit
  production use (e.g. single-node only).

## Licence tiers

| Tier | Licences | Rule |
|---|---|---|
| 🟢 Green | Apache-2.0, MIT, BSD-2/3, ISC, 0BSD, PostgreSQL, Zlib | Use freely |
| 🟢 Green (weak, file-level copyleft) | MPL-2.0, EPL-2.0, EDL-1.0 | Use freely; publish changes to *those files* only |
| 🟡 Yellow | LGPL-2.1, GPL-2.0 for separate programs (Linux kernel, U-Boot, BusyBox, SWUpdate) | OK with source-offer compliance; keep LGPL libraries dynamically linked |
| 🔴 Red on device | GPL-3.0, LGPL-3.0, AGPL-3.0 | Avoid in device images |
| 🔴 Red everywhere | AGPL-3.0 (backend), BSL, SSPL, ELv2, Timescale License, custom/"free up to N devices" | Avoid in core; allowed only as an optional, user-chosen plugin behind a seam |

## Status of surveyed tools

| Area | 🟢 Use | 🟡 OK with care | 🔴 Avoid in core (alternative) |
|---|---|---|---|
| OS / build | Yocto (with `INCOMPATIBLE_LICENSE`), Zephyr, FreeRTOS, NuttX, MCUboot, musl, toybox | Linux kernel, U-Boot, BusyBox, glibc, systemd (LGPL-2.1) | GRUB, bash, coreutils (GPLv3) → U-Boot / systemd-boot, BusyBox/toybox |
| OTA | Mender client, bootc, TUF libs, aktualizr (MPL-2.0), hawkBit (EPL-2.0) | RAUC (LGPL-2.1+), SWUpdate (GPL-2.0), OSTree (LGPL-2.0+) | – |
| Workloads | Podman, Moby/Docker, containerd, runc, k3s, Ankaios, Kanto, WAMR, Ocre, WasmEdge | crun (GPL-2.0+) | – |
| Messaging | Zenoh + zenoh-pico (EPL-2.0 / Apache-2.0), NATS, Mosquitto (EPL/EDL), NanoMQ, HiveMQ CE, VerneMQ, Cyclone DDS, Fast DDS, Leshan, Wakaama | – | EMQX ≥ 5.9 (BSL 1.1, single node free) → Mosquitto/NanoMQ/NATS; Anjay ≥ 3.10 (changed terms) → Wakaama/Leshan or zenoh-pico |
| Identity | step-ca, OpenBao (MPL-2.0), SPIRE, Keylime, tpm2-tss, Parsec, go-fdo, Keycloak, Ory, OpenFGA, SpiceDB | EJBCA CE (LGPL-2.1) | HashiCorp Vault (BSL) → OpenBao; Zitadel ≥ v3 (AGPL) → Keycloak/Ory |
| Artifacts | Zot, distribution, Harbor, Spegel, Dragonfly, cosign, Notation, ORAS, SeaweedFS | Ceph (LGPL mix) | MinIO (AGPL, archived), Garage (AGPL) → SeaweedFS |
| Ingestion | Kafka, Pulsar, NATS JetStream, Bento (MIT fork of Benthos), Vector (MPL-2.0), Telegraf, NiFi, Node-RED, Flink, RisingWave, ClickHouse, QuestDB, InfluxDB 3 Core, Apache IoTDB, DuckDB, Iceberg, PostgreSQL | – | Redpanda (BSL) → Kafka; Materialize (BSL); TimescaleDB extra features (Timescale License) → plain PostgreSQL / QuestDB / ClickHouse; TDengine (AGPL) |
| Observability | OpenTelemetry, Fluent Bit, Grafana Alloy, Prometheus, VictoriaMetrics/VictoriaLogs single-node, Perses (dashboards) | – | Grafana, Loki, Tempo, Mimir (AGPL) → Perses + VictoriaMetrics/VictoriaLogs + Jaeger |
| Remote access / overlay | Headscale, Tailscale client (BSD-3), wireguard-go, boringtun (BSD-3), Nebula (MIT), innernet, OpenZiti, frp, rathole | WireGuard kernel module (GPL-2.0) | NetBird server ≥ 0.53 (AGPL; client stays BSD-3); Teleport (non-OSS community licence); ZeroTier (source-available core, verify) |
| Platforms | ThingsBoard CE, Project EVE, Flight Control (verify), Mender server OSS (verify) | – | openBalena (AGPL, verify), OpenRemote (AGPL) |
| Config / CRDT | Automerge, Yjs, CUE, Jsonnet, Salt | – | Ansible (GPLv3; fine as an external control-side tool, not embedded) |

## Enforcement

- **Yocto:** `INCOMPATIBLE_LICENSE = "GPL-3.0* LGPL-3.0* AGPL-3.0*"` fails the build if such a package lands in the image; image licence manifests and SPDX SBOMs come for free.
- **Buildroot:** `make legal-info` produces the licence report.
- **CI for our own code:** `cargo-deny` (Rust), `go-licenses` (Go), plus ScanCode or the OSS Review Toolkit (ORT) for full scans.
- **Our own licence:** **MIT** in every weft repository (decided 2026-10-17). Short, universally understood, compatible with everything green. Trade-off: no explicit patent grant (Apache-2.0 has one); accepted for consistency and simplicity.
- **Blue Oak Model License 1.0.0** counts as green (permissive, OSI approved, with patent grant; used by minicbor).
- **cargo-deny uses an allow list**: every licence not listed (GPL, LGPL, AGPL, SSPL, BUSL, custom) fails CI. The trusted core (`weft-verifier`) additionally excludes weak copyleft.
