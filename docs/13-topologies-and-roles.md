# Topologies and node roles (draft)

Extends P4a (docs/01) and D4 (docs/04): the same apps, topics and documents work in every topology. The topology is configuration, not code.

## 1. Terms

"Edge" is ambiguous in the industry:

- **edge device**: any device near the physical process, including sensors;
- **edge gateway**: the device that connects a local network to a higher one;
- **edge server**: a site computer for heavier workloads.

weft therefore does not use "edge" as a role name. It describes what a node *does*:

| Term | Meaning |
|---|---|
| **Node** | Anything running the weft agent, or an MCU running weft directly |
| **Local network** (zone) | Nodes that can reach each other without going through an uplink: a machine, a line, a ship, a building |
| **Upstream** | Whatever sits above a local network: a site server, a plant network, another weft deployment, or a cloud. It is *not necessarily* a cloud |
| **Endpoint** role | Produces and consumes data, runs apps. Does not forward for others |
| **Relay** role | Forwards traffic for other nodes inside a local network (mesh, multi-hop) |
| **Uplink** role | Connects a local network to an upstream: forwards permitted traffic, stores and forwards while disconnected, caches artifacts and documents |
| **Proxy** role | Speaks for devices that don't run weft (Modbus devices, PLCs, small MCUs over BLE/CAN/serial) through adapters |
| **Site services** role | Optional local services: OCI registry mirror, document store, local UI, conflict view, ingestion buffer |

Rules:

- Roles combine. A single device can be endpoint, uplink, proxy and site services at once.
- Roles are assigned by signed config (the network profile), not by different builds.
- Several nodes may have the same role (several uplinks, several relays).

## 2. Supported topologies

| # | Topology | Roles | Example |
|---|---|---|---|
| T1 | **Standalone**, no upstream | One node: endpoint + site services. Updates by USB or technician laptop | Isolated machine, lab, air-gapped OT cell |
| T2 | **Single device, direct upstream** | Endpoint + uplink on the same node | EV charger or inverter with LTE |
| T3 | **Local network, one uplink** | Many endpoints/proxies, one uplink node (the classic "gateway") | Machine with a gateway, farm with a LoRaWAN gateway |
| T4 | **Local network, every device its own uplink** | Every node endpoint + uplink, peers talk locally | Charging park where each charger has cellular, robots on Wi-Fi |
| T5 | **Hierarchy** | Uplinks at each level, each level runs on its own | Machine → line → plant → cloud (ISA-95 / Purdue), ship → shore office |
| T6 | **Mesh** | Relays forward. Any node with an uplink role carries traffic upstream | Building sensors, mine tunnels |
| T7 | **Intermittent / mobile** | Uplinks come and go, with several link classes | Ship, train, robot, vehicle |

The topologies can be mixed: for example T3 inside a building that is one level of T5.

## 3. Design rules that make this work

1. **Trust comes from signatures, not from the path.** Documents and messages are signed end to end (COSE), so a relay or uplink cannot forge or change them. It can only delay or drop them. Rollback and equivocation are caught by the verifier (marks), and freshness by the timestamp role (planned). This is why any path is acceptable.
2. **Everything is idempotent and content-addressed.** Documents are identified by digest, and artifacts by OCI digest. Messages carry an id, so duplicates arriving through several uplinks (T4, T6) are dropped. No leader election is needed.
3. **Apps never see the topology.** They use the same topics (`weft/{tenant}/{device}/...`) everywhere. Whether a message goes peer to peer, through a relay or upstream is decided by routing config.
4. **Each local network keeps working without upstream.** Local pub/sub, local decisions (verifier on each node) and site services keep running. Uplinks store and forward, and replay when the link is back (P15).
5. **Crossing an uplink is policy.** What may leave a local network, at what rate, and what may come in, is a network conduit (docs/08) enforced at the uplink. This covers data sovereignty, downsampling and the IEC 62443 zone boundary. Default: nothing leaves.
6. **Transfers depend on the link.** Uplinks know their link class (`local`, `unmetered`, `metered`, `satellite`). Large artifacts move only over permitted classes (docs/12 §5).
7. **Discovery is local and automatic; upstream is configured.** Nodes inside a local network find each other with Zenoh scouting (multicast or gossip). Upstream endpoints are explicit config, reached over the overlay (Nebula/WireGuard).
8. **Proxies are a trust boundary.** A device that can't verify documents itself trusts its proxy. This must be visible in the inventory ("verified by proxy X") and in the device class (docs/10). MCUs that run the verifier themselves don't need to trust the proxy.

## 4. Mapping to existing tools

| weft role | Zenoh | MQTT | Overlay |
|---|---|---|---|
| Endpoint | `peer` (Linux) or `client` (zenoh-pico on MCUs) | Client | Overlay member |
| Relay | `peer` with gossip / `router` | n/a (broker only) | Nebula relay, or WireGuard routing |
| Uplink | `router` with an upstream connection, plus storage | Broker with a bridge to upstream (zenoh-bridge-mqtt or a broker bridge) | Lighthouse / gateway |
| Proxy | Adapter app publishing on behalf of devices | Same | n/a |
| Site services | Zenoh storage (persistent queryables) | Retained messages (limited) | n/a |

## 5. Tunnels and the overlay

The overlay (P11, docs/04: Nebula or weft-managed WireGuard, with OpenZiti optional and Rosenpass for post-quantum keys) is a **separate layer below pub/sub**. Every topology works with or without it.

1. **Pub/sub can run over any path:**
   - directly on the local network;
   - over the overlay;
   - over a tunnel the customer already has (IPsec, OpenVPN, their own WireGuard).

   Zenoh and MQTT only need IP reachability. Integrity never depends on the tunnel (rule 1), so a tunnel adds confidentiality and reachability, not trust.
2. **Overlay config is signed weft config.** A network component (`net/overlay`) carries the following, distributed and conflict-resolved like any config:
   - the peers' public keys, endpoints and AllowedIPs;
   - the Nebula CA bundle and groups;
   - the firewall rules generated from the policy.

   The private keys never leave the device, ideally staying in the TPM or secure element.
3. **Works offline.** Nebula certificates are signed offline by the tenant CA, so no live controller is needed. WireGuard peer lists come in signed documents, so a site can run its own overlay with no upstream (T1, T5).
4. **Uplink as tunnel gateway.**
   - An uplink node can route a subnet into the overlay. This gives remote access to devices that cannot join the overlay themselves (PLCs, Modbus TCP devices, HMIs), which is the "remote PLC programming" use case of classic industrial gateways (docs/12 §6).
   - The route is scoped by a conduit and time-limited by a grant, and every session is logged.
5. **MCUs** usually reach the overlay through their proxy or uplink. Running WireGuard on an MCU is possible (ESP32 ports exist), but it is not a weft requirement.
6. **Apps that need their own tunnel** (a customer VPN, a vendor service link) ask for it in the manifest. It is granted like any other network right (a `net:` grant at a higher permission tier, docs/08), so developers are not blocked, but the device owner sees and approves it.
7. **Nested tunnels** are fine: the overlay can run inside a corporate VPN or over satellite SD-WAN. Mind the MTU (docs/10: 1,280–1,420 bytes).

## 6. Open points

- Choosing an uplink when several are available (cost, link class, health) without central coordination.
- Group encryption for confidential traffic through untrusted relays and uplinks (docs/06).
- How a T5 level enforces that its subtree only receives documents meant for it, if bandwidth is limited: filter by selector at the uplink.
- Overlay key rotation and revocation timing across offline sites (a revoked device stays in a site overlay until that site gets the revocation).
- Test topologies T1–T7 in the dev environment (containers with network namespaces, WireGuard/Nebula between them, simulated link loss and metered links).
