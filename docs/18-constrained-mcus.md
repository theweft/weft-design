# Constrained MCUs: crypto and OTA (draft)

How MCUs that are too small for `weft-verifier` (measured: ~<!--f:measure.verifier.code_size_m4-->88 KB<!--/f--> code on Cortex-M4, ~<!--f:measure.verifier.peak_heap-->30 KB<!--/f--> peak heap for 10 documents, docs/10) still take part in weft safely.

## 1. Main idea: split authenticity from authorization

A firmware update answers two questions:

1. **Is this image genuine?** (authenticity)
2. **Should this device run it, now?** (authorization: version, priority, conflicts, activation gates)

Question 2 needs the whole verifier: delegations, priorities, conflict rules. Question 1 needs only **one signature check with a key baked into the bootloader**. MCUboot already does exactly that.

So on small MCUs:

- the **release key** that signs the firmware image is checked by the MCU's bootloader, *end to end*;
- the **proxy** (gateway/uplink, docs/13) runs the verifier on the MCU's behalf and decides *which* genuine image to install and *when*.

A compromised proxy can then, at worst:

- withhold updates (visible: the reported version falls behind);
- install a *different but genuine and not older* image (anti-rollback counter in the bootloader).

It can **never** install foreign or modified firmware. This is the same idea as Uptane's "partial verification" ECUs, which check only one signature.

## 2. MCU classes

| Class | Typical hardware | Runs | Image authenticity | Authorization | Config |
|---|---|---|---|---|---|
| **M1 Full** | ≥ <!--f:device.m1.flash_min-->1 MB<!--/f--> flash, ≥ <!--f:device.m1.ram_min-->256 KB<!--/f--> RAM, Cortex-M33/M4F/M7, ESP32-C6, nRF54 | `weft-verifier`, MCUboot, zenoh-pico | MCUboot + release key | On device (verifier) | Signed weft documents, verified on device |
| **M2 Constrained** | <!--f:device.m2.flash_min-->128 KB<!--/f-->–1 MB flash, <!--f:device.m2.ram_min-->16 KB<!--/f-->–256 KB RAM, Cortex-M0+/M4, nRF52, small STM32 | MCUboot (or a similar signing bootloader), a small update agent | MCUboot + release key (end to end) | **Proxy** runs the verifier, sends a compact signed *directive* | Compact config signed by the proxy |
| **M3 Peripheral** | < 128 KB flash, < 16 KB RAM, 8/16-bit, or no signing bootloader (UART, CAN or I²C ROM bootloaders) | Its application, maybe a vendor bootloader | **None on the device**: only a CRC or nothing. The proxy checks the image digest before flashing | Proxy | Written by the proxy (Modbus registers, CAN parameters) |
| **M0 Fixed** | Cannot be updated | – | – | – | Read-only; the version is reported by the proxy |

Rules:

- The class is part of the device description, and the inventory shows "verified by proxy X" for M2/M3 (docs/13 rule 8).
- For M3 the proxy is fully trusted. Security there comes from the local bus being isolated (conduits). That must be documented for IEC 62443.

## 3. Crypto on small MCUs

### 3.1 Verification only, if possible

- An M2/M3 device verifies but **needs no private key** for updates. That avoids secure key storage, RNG and side-channel protection.
- Public keys only need integrity, not secrecy. They can sit in the bootloader, in OTP, or as a hash in OTP.
- A private key is needed only if the device must **prove its identity** or sign its own reports. Options, from strongest to cheapest:
  1. a secure element (ATECC608, SE050, OPTIGA Trust M; ~€1–3), which does key storage and P-256 in hardware;
  2. the MCU's own key storage (TrustZone-M / PSA, a protected flash area) with a true RNG;
  3. a **per-device symmetric key** shared with its proxy (HMAC-SHA-256 / AES-CMAC), provisioned at manufacture. Very cheap, but the proxy can then impersonate the device. Acceptable for M2/M3 behind a trusted proxy.

### 3.2 Algorithm choice

| Algorithm | Why on MCUs | Notes |
|---|---|---|
| **ECDSA P-256 (ES256)** | **Hardware acceleration is common**: STM32 PKA, Nordic CryptoCell, ESP32-C6 ECC accelerator, secure elements | Preferred for M2. This is the reason the verifier has the `p256` feature |
| Ed25519 | Simple, constant time, small code; rarely hardware-accelerated | Fine on M4F and above in software |
| SHA-256 | Hardware-accelerated on most MCUs; hashing an image streams from flash, with no RAM buffer | Used for digests everywhere |
| **HSS/LMS** (RFC 8554, COSE alg −46) | **Post-quantum, and verification needs only SHA-256**: cheap on any MCU, with hardware SHA even cheaper | Signatures ~2–5 KB, public key ~60 bytes. Signing is *stateful* (each one-time key used once), so it is done only by the release HSM. Ideal for firmware signing, recommended by CNSA 2.0 |
| ML-DSA-44 | Post-quantum, general purpose | Public key 1,312 B, signature 2,420 B. Verification needs a few KB RAM with streaming implementations. Realistic on M4 and above; heavy for M0+. Measure |
| RSA-2048/3072 | Supported by MCUboot | Large keys and signatures, slow without acceleration; legacy only |

**Proposal:**

- Firmware images are signed with **P-256 now, with HSS/LMS added** as the post-quantum path. An image can carry both signatures during the transition, and the bootloader checks what it supports.
- Directives and config (§4.2) from the proxy use **P-256 or Ed25519**. They are short-lived, so a quantum attacker gains little, and they move to ML-DSA once that fits.

### 3.3 Anti-rollback and time

- Rollback protection uses a **monotonic security counter** in the image header. MCUboot's hardware security counter is backed by OTP, a secure element, or protected flash.
- M2/M3 devices have **no trusted time**. Expiry and freshness checks are done by the proxy's verifier.
- Directives carry a **sequence number** that the device persists. Each new directive must have a higher number, so an old directive can't be replayed.

## 4. OTA on small MCUs

### 4.1 Flow

```
release HSM ──signs image──► OCI registry ──► proxy (gateway/uplink)
                                          │  1. verifier: is a deployment for fw/<mcu> authorized? which version?
                                          │  2. checks image digest + signature (same check as the bootloader)
                                          │  3. activation gates (docs/08): battery, machine idle, window
                                          ▼
                                   MCU (M2): receives directive + image in chunks into the secondary slot
                                          │  4. bootloader checks the release signature + security counter
                                          │  5. test boot, self-test, confirm (or automatic revert)
                                          ▼
                                   reported version → proxy → weft reported state
```

### 4.2 The directive (M2)

- A small CBOR/COSE message from the proxy to one MCU:
  - device id;
  - component;
  - image digest;
  - security counter;
  - a sequence number;
  - optional config values.
- It is signed with the proxy's key, which the MCU learned when it was provisioned.
- About <!--f:directive.size-->150–250 bytes<!--/f-->, verified with one signature check.
- Long term this is a **SUIT manifest** (RFC 9019/9124) for interoperability. Nordic's nRF Connect SDK already uses SUIT on newer parts, and libcsuit exists.

### 4.3 Flash layout options

| Layout | Flash need | Rollback | When |
|---|---|---|---|
| MCUboot swap (move or scratch) | ~2× image + scratch/trailer | Yes (test → confirm, revert otherwise) | Default for M1/M2 |
| MCUboot direct-XIP (A/B in place) | 2× image | Yes, no swap time, less flash wear | When both slots can execute in place |
| **Secondary slot in external SPI flash** | Internal 1× image + a cheap external NOR (~€0.20–0.50) | Yes | Small internal flash: the most common fix |
| Overwrite-only + serial recovery | ~1× image | **No** (a bad image needs recovery) | Very small flash. The proxy must be able to re-flash over the bus (M3-like) |
| Compressed or delta images | Smaller transfer, sometimes a smaller slot | Depends | Slow links (LoRa, NB-IoT). Support varies by MCUboot version and SDK. Evaluate |

### 4.4 Transport from proxy to MCU

| Link | Standard to use |
|---|---|
| UART / USB, BLE | **mcumgr / SMP** (Zephyr + MCUboot standard): image upload, test, confirm, list |
| CAN | ISO-TP + UDS (automotive style), or CANopen SDO for CANopen devices |
| Thread / 6LoWPAN / NB-IoT | CoAP block-wise, LwM2M firmware object; OSCORE for message security (docs/16) |
| LoRaWAN | FUOTA (multicast + fragmentation). Very slow, so use small or delta images |
| Vendor ROM bootloader (M3) | Through an adapter plugin per vendor protocol (STM32 system bootloader, ESP ROM, etc.) |

Requirements for every link:

- **chunked and resumable** after power loss or disconnect;
- the proxy keeps the image until the MCU confirms;
- confirmation comes only after the MCU's own self-test (watchdog revert otherwise).

### 4.5 Products with several processors

Many products contain one Linux SoC and several MCUs: motor control, safety, radio.

- **Compatibility:** a product-level deployment lists compatible versions together, as a *bundle* of components.
- **Order and activation:** all images are staged first and activated together. If one MCU fails its self-test, all of them revert where possible. This is the component ordering gap from docs/12 §5 and docs/14.
- **Safety MCUs:** weft may deliver the image, but activation goes through an explicit gate (G13) and the MCU's own certified update logic.

### 4.6 Energy (battery devices)

- Activation gate: battery level and an energy budget for the flash write.
- Sleepy devices pull on wake from the proxy's mailbox (G12). The proxy never pushes into a sleeping radio.

## 5. What this means for weft-verifier and the agent

- **Verifier:** no change for M1. For M2, the proxy needs the verifier's `Install` instructions *per proxied device*. The proxy then turns them into directives and SUIT manifests outside the trusted code. The verifier's facts must include the proxied device's reported version and class.
- **`weft-directive`** (name decided 2026-10-18), a tiny crate for M2:
  - `no_std`, no alloc;
  - one signature check plus a sequence check;
  - target < <!--f:directive.code_budget-->10 KB<!--/f--> code without the crypto library, a few hundred bytes of RAM.

  This is a second, much smaller trusted piece of code for M2 devices.
- Layout: `weft_directive::check(...)` is `no_std`, has no `alloc`, and is the only part an MCU runs. `weft_directive::encode(...)` sits behind a feature and is used by the proxy, outside the trusted code.
- Deliberately *not* named `weft-verifier-lite`: M2 trusts its proxy for authorization, which is a weaker model than the verifier's, and the name keeps that visible.
- **Signing tooling:** HSS/LMS needs stateful key management in the release HSM. Plan it before the first post-quantum firmware release.
- **CI:** add an M2 reference board (nRF52840 or STM32L4) and an M0+ board (RP2040, STM32G0) to the test lab (docs/17).

## 6. Open points

- Measure on real MCUs: Ed25519 vs. P-256 (hardware) vs. LMS vs. ML-DSA-44 verification time, RAM and energy.
- MCUboot support status for LMS and ML-DSA (and in which version), plus compressed images.
- Rotating the release key on devices whose bootloader keys are in OTP: keep several key slots, or a key hash with a signed key-update path.
- Proxy key provisioning for M2 (at manufacture vs. at onboarding, G1).
