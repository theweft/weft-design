# weft-agent: the Linux device and gateway agent (draft)

Everything that runs on a Linux device or gateway around `weft-verifier`. It builds on docs/07 §2 (privilege separation), docs/11 (time and persistence), docs/13 (roles), docs/15 (transport contract) and docs/16 (transport protocols).

## 1. Three processes

The network-facing code may prove who it is, but it never decides anything:

```
 network / USB ──► [ transport ]   unprivileged, sandboxed: Zenoh/MQTT/USB/registry I/O, document sync
                        │ raw bytes (documents, artifacts by digest)
                        ▼
                   [ decider ]     no network, no privileges: weft-verifier + persisted state
                        │ typed instructions: install(hash) · apply_config(hash) · start(app)
                        ▼
                   [ executor ]    privileged, tiny: re-checks hashes, activation gates,
                                   calls SWUpdate / Podman / systemd; nothing else
```

- A bug in the transport, which has the largest attack surface, gives an attacker an unprivileged, sandboxed process that cannot forge decisions.
- The decider has no network and no privileges. Its only input is bytes, and its only output is typed instructions.
- The executor parses no untrusted formats and accepts a fixed, small command set.
- Sandboxing uses what the OS already has: separate users, systemd hardening options, seccomp, Landlock, read-only file systems.
- The IPC between the processes is gap G10. Candidate: CBOR messages over Unix sockets, with peer credentials checked.

## 2. Repository layout (Cargo workspace)

| Crate | Contents | Process |
|---|---|---|
| `weft-transport` | The `Transport` trait (docs/15 §5) and adapters: sim (turmoil), file/USB, later Zenoh and MQTT | transport |
| `weft-sync` | Content-addressed document store; push plus anti-entropy pull; outbox per traffic class; dedupe and sequence numbers (docs/15 §3) | transport |
| `weft-decider` | Feeds documents to `weft-verifier::step`; atomic state persistence (docs/11); instructions to the executor; reported state and conflicts | decider |
| `weft-executor` | Fixed command set; hash re-check; activation gates (docs/08); calls to SWUpdate, bootc, Podman/Quadlet, systemd, config files | executor |
| `weft-agent` (binary) | Starts and sandboxes the three processes; node roles from config (endpoint, relay, uplink, proxy, docs/13) | supervisor |
| `tests/sim` | Deterministic turmoil simulations across all crates (docs/04, test layers) | – |

Outside the agent:

- `weft-verifier`: the trusted core, a dependency.
- `weft-cli`: signing and authoring tools. Private keys of operators never live on devices.
- The backend, industry adapters (Modbus, OPC UA) and plugins: separate apps and processes.
- MCUs: no processes. M1 runs the verifier directly; M2 uses `weft-directive` (docs/18).

## 3. Authentication

| What is authenticated | Where | How |
|---|---|---|
| **Documents**: who may change what | Decider, via `weft-verifier` | COSE signatures and the delegation chain to the tenant trust anchor. The only kind that grants authority |
| **Network peers**: who may connect | Transport, and the overlay | mTLS (TLS 1.3, raw public keys), Zenoh/MQTT ACLs generated from the policy, Nebula/WireGuard certificates (docs/06, docs/16) |
| **The device**: identity for reports and onboarding | Identity key in the TPM or secure element, used by the decider | Signs reported state, conflicts and audit records |
| **Operators**: who issues documents | Operator side: `weft-cli`, HSM, later passkeys | Their keys sign documents; devices only verify |
| **On-site technicians** | Signed documents at local priority (a key on a laptop or hardware token) | Logging in to a local UI is the app's concern; only signed documents change anything |
| **Apps on the device** | Local broker or router, and the agent | Identity from Unix socket peer credentials or a per-app key; pub/sub rights from manifest grants (docs/08) |
| **Remote access** | Overlay | Time-bounded, audited grants (docs/13 §5) |

**Two device keys, both in the TPM:**

- the **identity key**, used only by the decider, signs reports;
- the **transport key**, used only for TLS handshakes, is certified by the identity key.

A compromised transport process can hold connections open but cannot forge signed reports.

**Not designed yet:** onboarding (G1): enrolling the device key and installing the trust anchors. Candidate: FDO.

## 4. OTA flow

1. **Decide (decider):** the verifier turns deployment documents into `Install{component, digest}`. Version, priority, conflicts, rollback and withholding are all checked there.
2. **Fetch (transport):** the artifact is fetched **by digest** into a staging store, from the OCI registry, a peer, the gateway's cache or a USB stick. The digest is checked, so the source needs no trust.
3. **Install (executor):**
   - check the digest again;
   - wait for the activation gate (battery, machine idle, time window);
   - call the backend:

     | Profile | Backend |
     |---|---|
     | Yocto A/B | SWUpdate |
     | Image mode | bootc / OSTree |
     | Apps | Podman / Quadlet |
     | Config files | Atomic write (docs/11) |
     | MCUs behind the device (proxy role) | mcumgr plus a `weft-directive` (docs/18) |
4. **Confirm or roll back:**
   - a health check runs after the reboot or restart, then the update is confirmed (e.g. the A/B "boot OK" flag);
   - otherwise it rolls back automatically;
   - failed installs become an input to the verifier (device facts), so the same artifact is not retried forever.
5. **Report:** the new version and any conflicts, signed with the identity key.

The executor stays small because existing tools do the heavy lifting.

**Open:** whether the backends are compiled into the executor behind features, or run as separate small privileged helpers. To decide with the first backend.

## 5. First milestone: the turmoil simulation

Built:

- `weft-transport` with only the sim adapter;
- `weft-sync`;
- `weft-decider` with an in-memory state store.

The executor is a stub that records instructions. Process separation, Zenoh, MQTT and the real executor come later.

**Simulation:**

- **Nodes:** a cloud issuer, a gateway (uplink), three devices and a malicious relay.
- **Documents enter:** from the cloud, on a device's "USB stick", and from a technician on site.
- **Faults:** random partitions, delay, reordering and duplicates. The malicious relay tampers with, replays and drops documents.
- **Seeds:** each one reproducible; CI runs a few hundred.

**Checks:**

1. **Convergence:** after healing, all nodes hold the same documents (SYS-032).
2. **Agreement:** devices with the same labels reach identical decisions (SYS-017).
3. **Safety:** no rejected document is applied; versions and the time floor never go backwards (SYS-005, SYS-012).
4. **Untrusted path:** the malicious relay can only delay decisions, never change them (SYS-021).

**Sync summary**, the first version: a hash of the sorted digest list, followed by the full list only when the hashes differ. The simulation shows whether something smarter is needed (docs/15 §8).

## 6. Open points

- G10: the IPC between processes and to plugins.
- G1: onboarding and key enrollment.
- How the executor packages its backends (§4).
- Health check definition per profile, and how long to wait before confirming.
- Secrets delivery to apps (G4).
