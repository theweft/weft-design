# Test lab and funding (draft, 2026-10-18)

Prices are rough euro estimates (single units, incl. VAT, October 2026) and vary widely by distributor: the FRDM-IMX93 alone ranges from about $74 to $108. Memory prices have pushed some single-board computers up since 2025.

## 1. What needs real hardware, and what doesn't

**Simulate** (free, in CI):

- fleet scale (thousands of nodes as containers);
- topologies T1–T7 (network namespaces);
- bad links (`tc netem`: loss, delay, bandwidth caps, satellite RTT);
- architectures (QEMU user/system for arm64, riscv64, Cortex-M).

**Real hardware** is needed for:

- code size, RAM and timing on real MCUs and SoCs;
- **power-cut tests** (atomic persistence, docs/11);
- flash and eMMC wear;
- boot and A/B / MCUboot slot switching;
- TPM and secure-element key storage;
- real radios (Wi-Fi roaming, BLE, cellular modems, NAT timeouts);
- energy use;
- a real Modbus device and a real industrial router as reference points.

## 2. Tier A: essential lab (≈ €<!--f:lab.tier_a_cost-->850–1,000<!--/f-->)

| Item | Covers | ≈ € |
|---|---|---|
| Raspberry Pi 5 (4 GB) + PSU + NVMe/SD | arm64 Linux, Yocto/SWUpdate and bootc tests | 120 |
| NXP FRDM-i.MX93 | Industrial-class arm64 (Cortex-A55 + M33), the same SoC family as chargers and gateways | 80–100 |
| RISC-V Linux board (Banana Pi BPI-F3 or StarFive VisionFive 2) | riscv64 Linux | 80–120 |
| Mini PC (Intel N100 class, new or used) | x86-64 gateway / IPC stand-in, backend node | 150–200 |
| ESP32-C6 DevKit, STM32 Nucleo-U575, nRF54L15 DK, Raspberry Pi Pico 2 W | MCU targets (RISC-V, Cortex-M33, BLE/Thread, cheap Wi-Fi) | 90 |
| TPM 2.0 module for the Pi + NXP SE050 eval board | Hardware keys, monotonic counters | 55 |
| Managed switch with VLANs | Network conduits, zones | 40 |
| 2× USB-RS485 + cheap Modbus energy meter | Real Modbus RTU device for the adapter | 60 |
| USB- or GPIO-controlled power relay | Automated power-cut tests | 30 |
| LTE modem (USB or HAT) + IoT SIM | Real cellular: NAT timeouts, handovers, keepalive cost | 75 |
| Cables, USB-serial adapters, SD cards, debug probe (CMSIS-DAP) | – | 80 |

## 3. Tier B: reasonable for credible benchmarks (+ ≈ €<!--f:lab.tier_b_cost-->1,000–1,800<!--/f-->)

| Item | Why | ≈ € |
|---|---|---|
| Nordic Power Profiler Kit II | MCU current and energy per operation | 100 |
| USB-C power meter / INA219 logger | Linux board energy | 30 |
| Second units: 2× Pi, 1× i.MX93, 3× ESP32 | Multi-node tests: peer-to-peer, uplinks, mesh on real radios | 300 |
| Industrial cellular router (e.g., Teltonika RUT241/RUT956) | The typical box in the field (docs/12); OpenWrt-based | 120–250 |
| Raspberry Pi CM5 + IO board (eMMC) | Closer to a real product than an SD card | 130 |
| Used industrial gateway (e.g., Siemens IOT2050, Moxa UC) | Real target, real reference | 250–400 |
| i.MX8M Plus board | Common gateway SoC, NPU | 200–400 |
| Used small PLC (or OpenPLC on a Pi for free) | Real PLC as a Modbus/OPC UA counterpart, gate signals | 0–300 |

**Skip as a single developer** (simulate instead):

- vendor EVKs at €500–1,000;
- Joulescope-class meters;
- HIL racks;
- satellite terminals (Starlink: ~€300+ hardware plus a monthly plan; `netem` models the links well enough).

## 4. Running costs (≈ €5–20 per month)

- **Backend:** a small VPS for the relay, registry and backend tests, e.g., a Hetzner arm64 instance: ~€5–10/month.
- **IoT SIM data:** a few euro per month or less, with prepaid multi-year SIMs.
- **CI:**
  - GitHub Actions is free for public repositories, including Linux arm64 runners (GA since August 2025);
  - RISC-V and MCUs run in QEMU in CI;
  - lab boards can run as self-hosted runners, but **only for trusted branches**, never for pull requests from forks on a public repository.

## 5. Funding

| Programme | For | Amount | Status (2026-10-18) | Fit |
|---|---|---|---|---|
| **NLnet: Restack** (EU, Horizon Europe, Open Internet Stack) | Open internet infrastructure, from hardware to applications; mentions secure, privacy-friendly devices, deployment and service management, and supply chain trust | €5k–50k first grant, scale-up possible | **Open, deadline <!--f:funding.nlnet_restack_deadline-->3 Nov 2026<!--/f-->, 12:00 CET** | **Good.** Frame weft as open infrastructure for securely managing devices: signed updates, supply chain (SBOM, provenance), no lock-in |
| NLnet: CodeSupply | Open software metadata (origin, licences, vulnerabilities) | €5k–50k (total €400k) | Open, same deadline | Partial: only the SBOM/VEX/firmware-provenance part (G5/G6) |
| **netidee** (Internet Privatstiftung Austria) | Internet projects with open results, OSI licences; individuals allowed | up to €60k | Call 21 closed 7 July 2026; next call expected spring 2027 (assumption from the yearly cycle) | Good, if you live in Austria |
| Prototype Fund (Germany) | Open-source prototypes by individuals | – | Continues (four more years from 2024); current round dates not found | Only for residents of Germany |
| Sovereign Tech Agency Fellowship / Fund (Germany) | Maintenance of widely used existing infrastructure | – | 2026 fellowship closed 6 April 2026 | Not yet: weft is new, not widely used |
| National research and innovation agencies (e.g., FFG and aws in Austria) | Usually companies or research partners | varies | – | Later, e.g., with a company or university partner |

### Notes for NLnet

- Funding is primarily for technical development and paid by milestones. Hardware can be included as a budget item if it is justified by the plan (check the FAQ).
- Proposals must be in English. Results must be under a free or open licence (MIT is fine).
- **Generative AI** ([NLnet GenAI policy v1.1](https://nlnet.nl/foundation/policies/generativeAI/), valid since 26 Jan 2026): allowed, but with conditions.
  - **Proposal:** disclose any AI use. Keep a *prompt provenance log* with the model, the date and time, the prompts, and the unedited output.
  - **Funded work:**
    - describe the AI use in the README;
    - mark generated contributions, e.g., as separate commits with the model, its version and the prompts in the commit message;
    - keep the logs publicly reachable without a login.

    For tests and docs, a README note is enough.
  - **Payment:** purely AI-generated deliverables are **not payable**. Human work (design, review, verification, integration) must be substantial, and you must be able to explain every design and code decision.
  - You are responsible for checking that AI output doesn't reproduce incompatibly licensed code.
  - Non-compliance can lead to rejection or termination of the grant.
- Keep the first grant focused: €5k–50k for one clear deliverable. For example:
  1. `weft-verifier` 1.0 (TUF roles, fuzzing, Kani, ML-DSA);
  2. the agent and executor for the embedded Linux profile with SWUpdate;
  3. the Modbus adapter and the pub/sub contract with conformance tests;
  4. the test lab and published benchmarks.

  A small, convincing first grant leads to scale-ups.
