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
