# Minimum device requirements (draft)

Measured values (2026-10-17, weft-verifier 0.1, Rust 1.99.1): see the bottom of this file.

## Device classes

| | MCU, verifier on device | MCU behind a gateway | Embedded Linux | Gateway / edge PC |
|---|---|---|---|---|
| CPU | 32-bit Cortex-M4/M33 class or better | Any MCU MCUboot supports | ARMv7-A / ARMv8-A | arm64 / x86-64 |
| Flash / storage | ≥ 1 MB (2 firmware slots + bootloader + settings) | ≥ 256 KB | ≥ 1 GB eMMC (A/B base OS + data); ≥ 4 GB with containers | ≥ 16 GB |
| RAM | ≥ 256 KB (≥ 128 KB without TLS / with small document sets) | ≥ 32 KB | ≥ 256 MB (≥ 512 MB with containers) | ≥ 2 GB |
| Boot | MCUboot, A/B slots, signature check | MCUboot | U-Boot or UEFI, A/B (SWUpdate) or OSTree/bootc | UEFI, bootc or A/B |
| Keys | Secure element, TrustZone-M (PSA) or protected flash | Same | TPM 2.0 (discrete or firmware) recommended | TPM 2.0 |
| Entropy | True RNG (keys, TLS) | True RNG | Kernel RNG | Kernel RNG |
| Time | RTC recommended; time floor persisted in flash | – | RTC | RTC |
| Anti-rollback | Monotonic counter (OTP / secure element) recommended | Same | TPM counter or eFuses recommended | TPM |
| Network | IP stack (Zephyr) for zenoh-pico/MQTT-SN, or none | BLE, CAN, serial, Thread to the gateway | Ethernet / Wi-Fi / cellular | Any |

## Software architecture per class

- **MCU, on device**: Zephyr (or Embassy) + MCUboot + `weft-verifier` (no_std) + zenoh-pico + settings
  storage. No processes: privilege separation via TrustZone-M / MPU where available (verifier and
  keys in the secure partition, e.g. a TF-M partition).
- **MCU behind a gateway**: MCUboot verifies the firmware signature (SUIT manifest planned); the
  gateway runs the verifier and forwards approved firmware and config.
- **Embedded Linux**: Yocto, systemd, SWUpdate, Podman (Quadlet), Zenoh, weft agent as separate
  processes (sandboxed transport, unprivileged decider, tiny privileged executor). Kernel: namespaces,
  cgroup v2, seccomp, nftables, overlayfs; dm-verity recommended.
- **Gateway / edge PC**: same as embedded Linux, optionally bootc image mode, optionally k3s.
- **Backend**: optional; any Linux host (single binary or Kubernetes).

## Environment and infrastructure

Principle: weft **requires no infrastructure** (offline-first) but uses it when present, and states
what degrades without it.

### Time

| Topic | Requirement | Without it |
|---|---|---|
| Hardware clock | Battery-backed RTC strongly recommended (supercap: days, battery: years) | Boots at a reset date; the verifier's time floor (persisted, advanced by signed `issued_at`) never goes backwards |
| Network time | **NTS** (RFC 8915) or **Roughtime**; plain NTP only as fallback, may only move time forward within limits, large jumps reported | Spoofed NTP could push time forward and "expire" certificates (denial of service) |
| Industrial time | PTP (IEEE 1588) and GNSS as sources | – |
| Durations | Monotonic clock for timers, retries, health checks | Time jumps would break them |
| Signed time | Planned TUF-style timestamp document doubles as authenticated coarse time for offline sites | – |
| Precision | RTC drift (~2 s/day) is irrelevant; only expiry checks depend on wall time | – |

UTC only; no time zones in the platform.

### Network

- Addressing: DHCP, static, IPv6 SLAAC, **link-local** (required for infrastructure-free peer-to-peer).
- Discovery: multicast scouting where allowed; **configured peers/routers always work** (industrial
  networks often block multicast); mDNS optional.
- DNS optional: everything works with IP addresses; only hostname conduits need DNS.
- Restrictive networks: outbound-only (NAT, CGNAT), HTTP proxy support, **fallback over 443**
  (Zenoh over QUIC/WebSocket), published list of required ports.
- MTU 1,280–1,420 on cellular and tunnels: no dependence on large packets.
- Metered links: bandwidth budgets, deltas, telemetry batching.
- Sleeping devices: liveliness distinguishes "asleep" from "gone".

### Storage, power, reliability

- **Atomic persistence (hard requirement for the shell)**: verifier state and document store survive
  power loss at any moment (write temp, fsync, rename; journaling file system).
- Flash wear: write budgets for logs/telemetry, rotation, RAM buffers, eMMC health monitoring; no SD
  cards in production.
- Read-only root file system, separate data partition.
- **Hardware watchdog (required)**: recovers hung agents, drives boot counting (U-Boot bootcount,
  MCUboot test mode) for A/B rollback.
- Brownout detection; optional supercap for clean shutdown.

### Security hardware and boot chain

- Unbroken secure boot: ROM → bootloader → kernel → dm-verity root file system.
- JTAG/SWD locked in production.
- Unique per-device keys (generated on device or in a secure element).
- Hardware RNG for early-boot key generation.

### Lifecycle and operations

- Factory provisioning: image, keys, onboarding voucher (FDO), hardware profile id, test.
- Recovery independent of the fleet: recovery partition or USB boot, factory reset.
- On-site service: local UI/CLI via Ethernet, USB network gadget or BLE; USB bundles (FAT32/exFAT,
  signed content only).
- Decommissioning and resale: revoke keys, secure erase, ownership transfer.
- Regulatory: radio domain (Wi-Fi country code), EN 18031.

## Measurements

| Item | Value | How |
|---|---|---|
| Verifier code size, Cortex-M4 (`thumbv7em-none-eabihf`) | ≈ 88 KB | Static library, `opt-level = "z"`, LTO, panic = abort; includes CBOR, SHA-256, Ed25519 (P-256 enabled made no measurable difference, inlining makes per-crate attribution unreliable; budget extra to be safe). Allocator not included (+ a few KB) |
| Peak heap for one step | ≈ 30 KB on 64-bit (≈ 20 KB expected on 32-bit) | 10 documents, 3.1 KB input: 1 delegation, 3 components × 2 config priorities (20 fields each), 3 deployments |
| Persisted state | 659 bytes | Same scenario, CBOR |

Optimization potential: borrow instead of copy in the CBOR layer, fewer generic instantiations,
processing documents in batches. Re-measure on real hardware (e.g. nRF5340, STM32U5) before committing.
