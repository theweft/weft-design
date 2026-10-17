# App deployment (draft design)

Principle: **secure by default, unrestricted by explicit grant.** Developers can do anything Linux
(or the MCU) can do; anything beyond the default sandbox is explicit, authorized by the deployer's
rights, and visible in reported state.

## Pipeline

Same path as OS updates: an **app package** (OCI artifact: weft app manifest + images by digest) →
signed **deployment document** for component `app/<name>` (artifact = package digest, plus granted
permissions) → `weft-verifier` emits `Install` → **workload plugin** runs it.

```yaml
name: pump-monitor
services:
  monitor:
    image: registry.example/pump-monitor@sha256:3f1c…   # by digest only
    health: { http: ":8080/health", timeout_s: 30 }
    resources: { memory_mb: 128, cpu_millis: 250 }
    restart: on-failure
permissions:                        # requested; effective = requested ∩ granted
  pubsub: { publish: ["tel/pump/**"], subscribe: ["cmd/pump/**"] }
  capabilities: [led.status, adc.0] # named hardware capabilities (see below)
  network: none                     # none | local | internet
config: { schema: config.schema.json, mount: /config }
```

## Permission tiers (nothing is impossible, everything is explicit)

| Tier | What the app gets | Typical use |
|---|---|---|
| **0 sandbox** (default) | Rootless, own user, read-only rootfs, seccomp, cgroup limits, own pub/sub namespace, no devices | Most apps |
| **1 named capabilities** | Hardware from the board's hardware profile (`led.status`, `adc.0`), granted pub/sub topics, network | Apps using I/O |
| **2 raw grants** | Arbitrary device nodes and sysfs paths (rw), extra Linux capabilities (e.g. `CAP_NET_ADMIN`), host mounts, host network | Drivers, network tools, legacy software |
| **3 unrestricted** | Privileged container, host PID/network/IPC, all devices; or a **native host service** (systemd unit, no container) | Bring-up, low-level system software |

- The deployer's rights carry a **maximum tier** (new delegation constraint). Example: platform team
  up to 3, product teams up to 2, site technicians up to 1. Narrowing applies as for all rights.
- Tiers ≥ 2 are flagged in reported state and dashboards (audit, IEC 62443 least-privilege evidence).
- **Development mode**: devices labelled `mode=dev` accept a local developer key for fast iteration
  (any tier, no fleet signing); production devices never trust dev keys. The device reports dev mode.

## Runtimes and formats: developers choose

Workload plugins behind one seam: **Podman/Quadlet** (default), Docker, Eclipse Ankaios, k3s/Helm,
systemd units / native binaries, WebAssembly. The weft manifest stays small (it is the privileged
executor's input); **importers outside the trusted path** translate Docker Compose and Margo
packages. A Compose feature without a weft equivalent maps to tier 2/3 grants instead of being
refused.

## Hardware access

- Prefer modern character devices: `/dev/gpiochipN` (libgpiod, lines by name), `/dev/i2c-*`,
  `/dev/spidev*`, `/dev/iio:deviceN`, serial. `/sys` stays read-only except granted files.
- Device numbers (`iio:deviceN`, `hwmonN`, `gpiochipN`, `i2c-N`, `ttyUSBN`) are assigned at probe
  time and are **not stable**. The board's **hardware profile** therefore holds *match rules* on
  stable properties (sysfs `name`, parent/physical path, device-tree node, GPIO line names, USB ids):

  ```yaml
  adc.0:      { match: { subsystem: iio,   attrs: { name: ads1015 }, parent: "platform/soc/40005400.i2c" } }
  fan.pwm:    { match: { subsystem: hwmon, attrs: { name: pwmfan } }, file: pwm1, access: rw }
  led.status: { gpio_line: STATUS_LED }
  modem:      { match: { subsystem: tty,   attrs: { idVendor: "2c7c", idProduct: "0125" } } }
  ```

- The workload plugin resolves rules at app start: **zero or several matches → fail closed**, app does
  not start, problem reported. It generates udev rules for stable links (`/dev/weft/adc0`) and
  ownership by the app's user; on hotplug it re-resolves and restarts or stops per manifest policy.
- Optional **hardware service**: a privileged daemon owns sensitive or shared hardware; apps request
  actions over local pub/sub (safety interlocks, shared buses).

## Network access: named conduits

Default **deny**. Networks work exactly like hardware: apps request **names**, profiles hold details.
The site's network owner publishes a signed **network profile** (config fragment for
`os/network-profile`) defining **conduits** (IEC 62443 term):

```yaml
conduits:
  plc-line3: { egress: [{ cidr: "192.168.10.20/32", ports: [502] }] }   # Modbus to one PLC
  cloud-api: { egress: [{ host: "api.example.com", ports: [443] }] }
  local-ui:  { ingress: [{ port: 8080, from: [local, overlay] }] }
```

The app manifest requests `conduits: [plc-line3, cloud-api]`. The workload plugin resolves names via
the profile: per-app network namespace (Podman default) + generated **nftables** rules; hostnames via a
local resolver that answers only allowed names and fills nftables sets (e.g. dnsmasq `nftset`). No L7
filtering; a proxy app can do that. Raw rules not in any profile (a specific CIDR) are a tier 2
grant; host networking is tier 2 as well. Device-level firewall rules not tied to an app: config for
`os/firewall`. MCUs: firmware owns the network stack; WebAssembly apps get network only as a granted
host function.

## Grants in the verifier: opaque strings

The verifier does **not** know what grants mean. Its only job: every grant in a deployment document
is covered by the signer's *grantable patterns*, and delegations intersect those patterns (narrowing).
It then copies the signed grants into the `Install` instruction.

- One pattern syntax for everything: segments separated by `/` or `.`, `*` = one segment,
  `**` = everything below (the same matcher as component patterns).
- Prefixes are a **naming convention** for people and plugins, not types the verifier interprets:

  | Prefix | Meaning | Defined by | Enforced by |
  |---|---|---|---|
  | `hw:` | hardware capability (`hw:led.status`) | board hardware profile | workload plugin (devices, sysfs, udev) |
  | `net:` | conduit (`net:plc-line3`) | site network profile | workload plugin (nftables, DNS) |
  | `pub:` / `sub:` | pub/sub namespace by direction (`pub:tel/pump/**`) | device namespace | local Zenoh router ACL |
  | `secret:` | secret the app may receive (`secret:db-password`) | secret store | workload plugin |
  | `vol:` | storage shared between apps (`vol:recordings`) | workload plugin | workload plugin |
  | `raw:` | tier 2 items outside profiles (`raw:device=/dev/mem`, `raw:cap=NET_ADMIN`) | the grant itself | workload plugin |
  | `tier:` | privilege tier (`tier:2`) | tier table | workload plugin (sandbox) |

- "Maximum tier 2" is expressed as grantable names `tier:0`, `tier:1`, `tier:2` (authoring tools expand it).
- **Unknown grants are handled by the plugin, fail closed**: a grant it does not understand stops the
  app from starting and is reported. New grant types never require a verifier change.
- Effective permissions = what the app manifest requests ∩ what the deployment grants (computed by the
  plugin; the verifier never sees the manifest).

## Microcontrollers

- **Default: firmware + config.** The application is compiled into the firmware (component `fw`,
  MCUboot A/B). Day-to-day changes are config fragments stored in Zephyr settings.
- **Optional: dynamic apps** on MCUs with roughly ≥ 256 KB RAM: WebAssembly (WAMR, LF Edge Ocre;
  sandboxed without an MMU, permissions as host functions) or Zephyr LLEXT (native, trusted code only).
- Same deployment documents and the same `weft-verifier` (no_std) on the MCU or on a gateway.

## Lifecycle

Install: pull by digest (registry, mirror, peer, USB/OCI layout) → start new → health check →
remove old; on failure switch back (previous images kept). Config: `ApplyConfig` → validate against
the app schema → mount read-only → reload or restart.

## Activation gates (when an update may take effect)

Downloading and staging happen immediately; **activation** (reboot, slot switch, app restart) waits
for a gate. Reasons: safety and production (no reboot mid-run), operations (shifts, staff on site),
regulation (UNECE R156: safe state, consent), cost and power (night tariffs, charging), spreading risk.

| Gate | Example | Wall time? |
|---|---|---|
| Device state | machine idle, vehicle parked, no job running, battery > 50 % | no |
| App veto | app says "not now" (like systemd inhibitor locks) | no |
| Approval | technician on site or user consent | no |
| Time window | 02:00–04:00 UTC | yes |

- The deployment document carries the signed activation policy (e.g. "idle OR night window OR
  technician confirms"). Evaluation needs live device state, so it happens in the executor, not the
  verifier (the verifier still only says `Install`).
- Emergency override ("activate as soon as possible") for critical patches is an explicit right.
- Config changes that apply without restart need no gate.
- Most gates need no wall time; time windows are UTC ranges.

## Gaps in weft-verifier

1. Removal ("absent" deployments).
2. Permission grants (opaque strings) in deployment documents, covered by the deployer's grantable
   patterns (intersected along delegations).
3. Failed installs reported in device facts, so the verifier stops retrying.
