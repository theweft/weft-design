# Minimal trusted code (draft design)

Guidance: less code means fewer implementation bugs. What matters most is the **trusted computing
base (TCB)**: the code that must be correct for the system to be secure. Glue, adapters, UIs and
plugins can be large as long as a bug there cannot break the security guarantees.

References: Saltzer & Schroeder, *The Protection of Information in Computer Systems* (1975),
"economy of mechanism"; D. J. Bernstein, *Some thoughts on security after ten years of qmail 1.0* (2007):
eliminate bugs, eliminate code, eliminate *trusted* code.

## 1. A tiny, pure verification core

All security decisions live in one small Rust crate, `fleetloom-core`:

- Verify signatures and delegation chains.
- Evaluate policy (capabilities, ACL decisions for commands and config writes).
- Resolve desired state and config fragments deterministically (05 §2.3).
- Decide what to do: "install artifact sha256:…", "apply config hash …", "reject, reason …".

Rules for this crate:
- **Pure functions, no I/O**: input = documents + device facts, output = decisions. No networking,
  no filesystem, no async runtime, no threads ("functional core, imperative shell").
- `#![forbid(unsafe_code)]`, `no_std` + `alloc`, so the **same code runs on Linux and MCUs**:
  one implementation, tested once.
- One input encoding only (deterministic CBOR), strict decoding: size limits, unknown fields rejected,
  "parse, don't validate" into typed structures.
- Minimal, vetted dependencies (COSE, CBOR, crypto); `default-features = false` everywhere.
- **Code budget** tracked in CI (lines of code and transitive dependency count).

Verification effort concentrates here: fuzzing (cargo-fuzz), property tests (proptest), Miri, and
model checking / proofs of key properties with **Kani** (e.g. resolution is deterministic and
order-independent; a narrowed delegation never grants more than its parent).

## 1b. Kernel scope (the `fleetloom-kernel` crate)

"Kernel" = the smallest trusted piece, not an OS kernel.

### Shape: one pure state-transition function

```rust
// no I/O, no clock, no randomness, no global state
fn step(state: &KernelState, anchors: &TrustAnchors, facts: &DeviceFacts, inputs: &[&[u8]])
    -> (KernelState, Decisions);

fn authorize(state: &KernelState, anchors: &TrustAnchors, facts: &DeviceFacts, request: &[u8])
    -> Verdict;   // for commands and local API calls
```

- **State** is passed in and out explicitly (highest versions seen, last-known-good hashes,
  revocations, time lower bound). The shell persists it. This makes every decision replayable and testable.
- **Inputs** are raw bytes; the kernel decodes them itself. The shell never pre-parses.

### Inside the kernel

| # | Function | Notes |
|---|---|---|
| K1 | Strict decoding | Deterministic CBOR only, size and depth limits, unknown fields rejected, into typed structs |
| K2 | Signature verification | COSE_Sign1 / COSE_Sign; small algorithm allowlist: Ed25519, ECDSA P-256 (TPMs and secure elements), ML-DSA; hybrid and threshold rules ("needs Ed25519 AND ML-DSA", "2 of 3 keys") |
| K3 | Delegation chains | Each step only narrows: tenant, site, labels, components, actions, config fields, validity window; revocation facts |
| K4 | Freshness and anti-rollback | Monotonic versions per (scope, issuer, component); expiry checked against a trusted time *lower bound*, never trusting the wall clock blindly |
| K5 | Authorization | Fixed constraint vocabulary, deny by default; used for commands, config writes, deployments, topic rights |
| K6 | Resolution | Snapshot + fragments → effective desired state: artifact hashes per component, effective config (merge patch + locks), deterministic |
| K7 | Plan | Effective desired state vs. reported facts → a small list of typed, authorized actions: `Install{component, hash}`, `ApplyConfig{component, hash}`, `Start/Stop{app}`, `RotateKey{...}` |
| K8 | Record building | Bytes for audit and reported-state records; *signing* is done by the key holder (TPM) outside |

### Outside the kernel (on purpose)

Networking and transports · storage · key storage and signing (TPM / secure element) · time source ·
downloading artifacts · JSON Schema validation of config (an authorized but invalid config is a
correctness problem, handled by the plugin with last-known-good rollback) · YAML / authoring ·
telemetry · UI · all plugins · executing actions.

The **executor** executes exactly the kernel's actions and re-checks artifact hashes before use.

### Dependencies (target)

`minicbor` (no_std CBOR), `coset` (COSE), `sha2`, `ed25519-dalek`, `p256`, `ml-dsa` (RustCrypto,
still pre-1.0: review before relying on it). Nothing else.

### Properties to prove or test exhaustively

- **Deny by default**: any decode, signature or delegation failure → no effect, state unchanged.
- **Narrowing**: a delegated right is never larger than its parent's.
- **No rollback**: an accepted version is never lower than the stored one.
- **Determinism**: same inputs in any order → same effective state and plan.
- **Locks hold**: no fragment changes a field locked by another scope without explicit override rights.
- **Totality**: no panics (`unwrap`/`panic` denied by lints), bounded memory and time for bounded input.

Tools: Kani (bounded model checking), proptest, cargo-fuzz on K1/K2, Miri.

### Budget

Target ≤ ~5,000 lines of own code; same crate builds for Linux and MCUs (`no_std` + `alloc`).

## 2. Privilege separation in the agent

```
 untrusted network ──► [ transport process ]  unprivileged, sandboxed: Zenoh/MQTT/USB/registry I/O
                               │ raw bytes (documents, artifacts by hash)
                               ▼
                       [ core / decider ]     no network, no privileges; runs fleetloom-core
                               │ narrow, typed commands: install(hash) · apply_config(hash) · start(app)
                               ▼
                       [ executor ]           privileged, tiny: re-checks artifact hash, calls SWUpdate /
                                              Podman / systemd; accepts nothing else
                               ▲
 plugins (OTA, workloads, overlay, adapters) run as separate, sandboxed processes
```

- A bug in the network-facing parts (the biggest attack surface) gives an attacker an
  unprivileged, sandboxed process that cannot forge decisions.
- The privileged executor has no parsers for untrusted formats and a fixed, tiny command set.
- Sandboxing with what the OS already has: separate users, systemd hardening options, seccomp,
  Landlock (Rust crate), read-only file systems.

## 3. Write less code by design

- **Don't write crypto, TLS or protocol stacks**: rustls, RustCrypto / aws-lc-rs, Zenoh, coset (COSE),
  minicbor, Cedar/Biscuit.
- **One of each in the trusted path**: one encoding (CBOR), one signature format (COSE), one artifact
  format (OCI), one merge semantics (JSON Merge Patch). YAML/JSON exist only in authoring tools, never
  on the device.
- **No scripting in the trusted path**: no shell or Lua hooks executed from documents. Prefer declarative
  handlers (applies to SWUpdate: no embedded Lua in update packages by default).
- **Delegate to the OS**: systemd for supervision, Podman for containers, dm-verity for integrity,
  the kernel for isolation. fleetloom orchestrates, it does not reimplement.
- **Fewer options**: every option is a code path and a misconfiguration risk. Secure defaults, few knobs.
- **Build only what the device needs**: Cargo features per device profile; a device image contains
  only its plugins.
- **Mandatory core stays small**: everything not in the six contracts is a plugin outside the TCB.

## 4. Dependencies are code too

- Prefer crates with few transitive dependencies; review new dependencies (cargo-vet), block licences
  and advisories (cargo-deny, cargo-audit), watch `unsafe` usage (cargo-geiger).
- Pin versions; reproducible builds; SBOM per build.
