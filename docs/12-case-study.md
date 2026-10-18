# Case study: target industries and their typical setups (draft)

Researched 2026-10-17. This is a rough survey, not market research. Most public material on edge setups is written by vendors, and there are no reliable public figures for platform market share. Each claim is therefore marked with how solid it is:

- **[S]**: survey or standard (independent data);
- **[V]**: vendor or case-study material (shows a pattern, not how common it is);
- **[J]**: our own judgment.

## 1. What makes an industry a good fit for weft

weft adds the most value where several of these hold:

1. **Offline or poor links.** The site must keep working, and accept changes, without a cloud (P15).
2. **Mixed devices.** MCUs, embedded Linux and gateways in one fleet, which one tool should manage.
3. **Several parties with authority.** OEM, integrator, operator, owner and technician all change config or software, so conflicts need rules (the priority model in docs/05 §2.3).
4. **Long life and regulation.** A 10–20 year service life, CRA, IEC 62443 and sector rules, so signed, auditable and certifiable updates matter.
5. **Mixed protocols.** Fieldbus and domain protocols have to be bridged to pub/sub and ingestion.
6. **No platform lock-in.** The customer, or the regulator, wants to be able to replace the cloud, or run without one.

Context **[S]**: in the Eclipse 2024 IoT & Embedded Developer Survey (747 respondents), the leading target industries were industrial automation (34%), automotive (29%), energy management (29%, up from 24%), environmental monitoring (23%) and healthcare (18%). Connectivity was the top concern (48%). OTA updates (32%) and communication security (40%) were among the top security measures.

## 2. Overview

| Industry | Fit | Why | Main competitor / incumbent |
|---|---|---|---|
| Distributed energy (PV, batteries, heat pumps) | ★★★ | Several parties, cellular, long life, grid regulation, large fleets | Vendor clouds (SMA, Huawei, SOLARMAN), AWS IoT |
| Industrial automation / machine builders | ★★★ | Offline OT, IEC 62443, mixed devices, machine builder and plant operator both have authority | Siemens Industrial Edge, Margo, FLECS, Azure IoT Edge |
| Water, wastewater, remote utilities | ★★★ | Remote unattended sites, cellular, very conservative, IEC 62443 | SCADA vendors, RTU vendors |
| Maritime and offshore | ★★★ | Expensive, intermittent satellite links, ship as an offline site | Vendor-specific ship servers, Marlink and similar |
| Rail and public transport | ★★☆ | Depot vs. in-service links, EN 50155 hardware, 30-year life | Vendor-specific onboard platforms |
| Mobile robots / AMR | ★★☆ | Wi-Fi roaming, peer-to-peer, ROS 2 already moving to Zenoh | In-house tools, Formant, InOrbit |
| EV charging | ★★☆ | Large Linux fleets, cellular. OCPP owns charging and firmware, so weft sits underneath | CSMS backends via OCPP, AWS IoT |
| Building automation | ★★☆ | Several parties (owner, facility manager, integrator), BACnet, long life | Niagara, Honeywell, Siemens Desigo |
| Mining, oil and gas | ★★☆ | Offline, private LTE, safety. Small vendor community | SCADA and DCS vendors |
| Agriculture | ★☆☆ | Sensors are LoRaWAN (not weft endpoints). Fit is only at the gateway or machine | LNS (ChirpStack, TTN), OEM clouds |
| Retail, QSR, kiosks | ★☆☆ | Mostly x86 IT, k3s/Rancher and Azure Arc already fit | Rancher Fleet, Azure Arc, SUSE Edge |
| Medical devices | ★☆☆ (later) | Heavy regulation (IEC 62304, FDA). Worth it only after weft is mature | OEM-specific |
| Automotive (in-vehicle) | ☆☆☆ | Uptane and AUTOSAR are entrenched, OEM-controlled | Uptane implementations, OEM stacks |
| Consumer IoT | ☆☆☆ | Low margins, Matter, vendor clouds | Matter, vendor clouds |

Star ratings are **[J]**.

## 3. Industry profiles

For each industry: device hardware, field network (on site), backhaul (site to cloud), backend, and what it means for weft.

### 3.1 Distributed energy (PV inverters, batteries, heat pumps, energy managers)

- **Hardware:**
  - inverter and battery management on MCUs;
  - a data logger or energy manager on embedded Linux or an MCU **[V]**.
- **Field network:** Modbus RTU (RS-485) and Modbus TCP, with SunSpec models as the vendor-neutral register map **[V]**. Some Wi-Fi and Ethernet.
- **Backhaul:**
  - the home or site internet;
  - often 4G as well, so devices don't depend on the customer's network **[V]**.
- **Backend:**
  - vendor clouds push firmware to the fleet: SOLARMAN, Yaskawa Solectria, SMA **[V]**;
  - grid communication moving to IEEE 2030.5 **[V]**.
- **Typical practices:**
  - SMA separates transfer from installation, with a configurable order and timing per inverter **[V]**;
  - battery and inverter firmware are versioned separately, and the different reporting channels don't always agree (anecdotal forum report) **[V]**.
- **For weft:**
  - This matches the activation-gate model (staging is separate from activation) and the component model.
  - Several parties set config here: the OEM (limits), the installer (site), the grid operator (curtailment, grid code) and the owner (tariffs). That is exactly the priority/lock case: grid code values are *locked* at site priority **[J]**.
  - Modbus/SunSpec and IEEE 2030.5 belong in the ingestion and adapter layer, not in the core.

### 3.2 Industrial automation, machine builders

- **Hardware:**
  - ARM Linux gateways (e.g., Moxa UC series on Cortex-A, −40 to +85 °C) for protocol conversion and data aggregation;
  - x86 industrial PCs for vision and analytics **[V]**;
  - vendors quote 10–15 year use and ARM SoC availability of ≥ 15 years **[V]**;
  - PLCs stay; weft does not replace them **[J]**.
- **Field network:** PROFINET, EtherNet/IP, Modbus TCP, OPC UA, plus legacy serial.
- **Backhaul:** the plant network, segmented by the Purdue model (ISA-95/IEC 62443 zones and conduits). Direct internet from a cell is often prohibited **[J]**.
- **Backend:**
  - an on-premises historian/MES, with the cloud usually optional;
  - manufacturing leads in on-premises edge use tied to OT **[S, Edge IR 2025 survey, 100+ respondents]**;
  - in Europe, 70% of 800 surveyed companies feel too dependent on non-European technology **[S, HarfangLab, IT-focused]**.
- **For weft:**
  - Offline/on-prem must be the default deployment, not the exception.
  - Network conduits (docs/08) map one-to-one to IEC 62443.
  - The machine builder (OEM, lower priority) and the plant operator (site, higher priority, with lock override for safety) are a real authority split **[J]**.

### 3.3 Water, wastewater, remote utilities

- **Hardware:** RTUs and PLCs at pump stations, wells and reservoirs. Increasingly cellular RTUs **[V]**.
- **Field network:** Modbus and DNP3 to the SCADA host.
- **Backhaul:**
  - private radio and leased lines are being replaced by cellular;
  - one case study reports cellular connectivity above 99.5% over a year;
  - cellular is also used as a backup path **[V]**.
- **Backend:**
  - an on-premises SCADA host;
  - guidance follows IEC 62443 and NIST SP 800-82, and calls for local fail-safe control that does not rely on the network perimeter **[V, Cisco/Rockwell guide]**.
- **Typical practices:**
  - vendor VPNs are replaced with jump hosts with MFA;
  - forgotten cellular devices were found only through traffic analysis **[V]**.
- **For weft:**
  - Device inventory and attestation matter as much as updates.
  - Signed offline updates (USB, or a technician's laptop) are required, since a firmware update was done offline in the case study.
  - The overlay with a per-device identity replaces vendor VPNs.

### 3.4 Maritime and offshore

- **Hardware:** an onboard server or gateway per vessel, plus engine and navigation systems (separate, class-regulated).
- **Field network:** ship LAN, NMEA 0183/2000, Modbus to engine systems **[J]**.
- **Backhaul:**
  - hybrid satellite: Starlink LEO plus GEO VSAT (Ku band) plus L-band for backup and safety;
  - SD-WAN steers traffic per application **[V]**;
  - Starlink plans have data caps, and fleets can pay up to 3× more than with VSAT **[V]**;
  - at sea, a port or coastal LTE link can come and go.
- **Backend:** the shore office, whether cloud or own data center.
- **For weft:**
  - The ship is a site that is offline for days, with a local site authority (the master or chief engineer at site priority).
  - Transfers must be **link-aware**: big artifacts only over cheap links (port Wi-Fi, LTE), metadata over satellite. This is a transfer gate next to the activation gate **[J]**.
  - Delta updates and OCI layer de-duplication are worth real money here.

### 3.5 Rail and public transport

- **Hardware:**
  - EN 50155-certified onboard gateways and servers: Nexcom nROK, Eurotech BoltGATE/BoltCOR, Owasys owa5X, Teldat H2-Rail **[V]**;
  - mixed ARM and x86.
- **Field network:** train backbone (Ethernet Train Backbone, MVB/CAN on older stock), passenger Wi-Fi, CCTV and passenger information systems **[V/J]**.
- **Backhaul:**
  - multi-modem 4G/5G with link aggregation, sometimes satellite **[V]**;
  - depot Wi-Fi for large transfers **[J]**.
- **Backend:** the operator's control center, usually on-prem or private cloud.
- **For weft:**
  - Same link-aware transfer as maritime.
  - Activation gates are needed ("only in depot, standing, no passengers").
  - Very long life (30 years), so the crypto agility and post-quantum plan matter.

### 3.6 Mobile robots / AMR

- **Hardware:**
  - x86 or NVIDIA Jetson / ARM compute running ROS 2;
  - MCUs for motors and safety **[J]**.
- **Field network:** warehouse Wi-Fi with roaming. DDS multicast discovery copes badly with access points, and Husarion added rmw_zenoh for that reason **[V]**.
- **Typical practices:** Dexory moved a production ROS 2 fleet (> 200 nodes per robot) from DDS to Zenoh **[V]**.
- **Backhaul:** site LAN to fleet manager on premises, with optional cloud.
- **For weft:**
  - Strong fit for the Zenoh pub/sub layer and peer-to-peer topology.
  - Activation gate = "robot docked/charging".
  - ROS 2 is a natural first app framework to demo **[J]**.

### 3.7 EV charging

- **Hardware:**
  - controller board plus power board;
  - controller typically embedded Linux on NXP i.MX6ULL (e.g., chargebyte C300) and newer i.MX93 reference designs;
  - secure element (NXP SE050) for ISO 15118 and IEC 62443 **[V]**.
- **Field network:** ISO 15118 (HomePlug Green PHY) to the car, Modbus to meters and power electronics.
- **Backhaul:** mostly cellular (modems from Quectel, SIMCom, u-blox) **[V]**, plus Ethernet.
- **Backend:**
  - a charging station management system (CSMS) over OCPP 1.6 and 2.0.1;
  - OCPP includes firmware updates, signed in the newer versions, and backends must serve mixed versions **[V]**.
- **Typical practices:**
  - one 300-charger fleet moved from a self-hosted MQTT broker to AWS IoT Core for registry, shadow and jobs **[V]**;
  - multiple components (controller, payment, connectivity) must be updated in the right order **[V]**.
- **For weft:**
  - OCPP owns the charging business logic. weft can manage the OS, apps and config underneath, and either implement OCPP's firmware-update command as an adapter or coexist with it **[J]**.
  - Ordering between components is a gap: we have activation gates but no dependencies between components (see §5).

### 3.8 Building automation

- **Hardware:** Linux edge controllers and gateways, e.g., Honeywell ECG100 (Linux/RTOS, BACnet BBMD) **[V]**.
- **Field network:** BACnet/IP and BACnet MS/TP, KNX, Modbus, M-Bus for meters.
- **Backhaul:** building IT network, often with a cellular router for vendor access **[J]**.
- **Backend:** building management system (BMS: Niagara and others) on premises, plus cloud analytics. Containerized protocol gateways are becoming common **[V]**.
- **For weft:** Several parties again (owner, facility manager, several integrators per trade). Apps per trade map to component prefixes (`hvac/*`, `lighting/*`) and delegated rights **[J]**.

### 3.9 Agriculture (brief)

- **Hardware:** battery LoRaWAN sensors, plus gateways (often an embedded Linux router) **[V]**.
- **Backhaul:** cellular from the gateway. 5–15 km of LoRa range in open land **[V]**.
- **Backend:** a LoRaWAN network server (LNS) locally or in the cloud, then the farm application **[V]**.
- **For weft:** weft manages the gateway, not the sensors. A local LNS (ChirpStack, MIT) as a weft app keeps the farm working offline **[J]**.

### 3.10 Retail and kiosks (brief)

x86 mini PCs and small servers per store. K3s with Rancher Fleet GitOps is a documented pattern, as is offline-first local caching **[V]**. This is solved well enough by IT tools, so weft has little to add beyond what Rancher or Arc offer **[J]**.

## 4. Common setups

The most typical hardware, network and backend setup across the high-fit industries. This is a synthesis, **[J]** unless noted.

### Hardware

| Tier | Typical parts | Notes |
|---|---|---|
| Sensor/actuator MCU | STM32 (pros), ESP32 (most used overall, 38% in JetBrains 2025 **[S]**), nRF52/nRF54 (BLE/Thread), RP2040/RP2350 | ESP32 is Xtensa or RISC-V, **not** Cortex-M |
| Embedded Linux device | NXP i.MX6ULL / i.MX8M / i.MX93, TI AM62x, Raspberry Pi CM4/CM5 | Cortex-A7/A53/A55, 256 MB–2 GB RAM, eMMC |
| Gateway / industrial PC | ARM gateways (Moxa, Advantech, Siemens), x86 IPCs (Intel Atom/Core) **[V]** | −40…+85 °C, 10–15 year availability |
| Router | Cellular industrial routers (Teltonika, Robustel, Moxa, Siemens Scalance), many OpenWrt-based | Often the only thing with WAN access |

### Field network

Modbus RTU/TCP is the common denominator across energy, utilities, buildings and machines. Domain protocols sit on top:

- OPC UA and PROFINET (factory);
- BACnet (buildings);
- DNP3 (utilities);
- SunSpec and IEEE 2030.5 (energy);
- OCPP and ISO 15118 (charging);
- NMEA (maritime);
- ROS 2 (robots).

### Backhaul

- In order of how common they are: cellular LTE (Cat-1 to Cat-4, more and more 5G RedCap), customer or site Ethernet, then satellite (maritime, remote).
- Wi-Fi for mobile machines.
- Multi-link with failover is normal **[V]**.

### Backend

- Three patterns:
  1. **Vendor cloud** (energy, charging), often built on AWS IoT Core **[V]**;
  2. **On-prem** SCADA, historian or BMS (industry, utilities, buildings, rail);
  3. **hybrid**: the site works alone, and the cloud gets copies.
- Hyperscaler IoT services are not a stable base:
  - Google Cloud IoT Core shut down in 2023 **[S]**;
  - Microsoft announced, then withdrew, an IoT Central retirement in 2024 **[V/press]**.

  Customers have learned that the platform under them can disappear, which supports weft's "every layer exchangeable" principle.

### Regulation

- **CRA:** reporting of actively exploited vulnerabilities from 11 Sep 2026 (24 h / 72 h / 14 days). Full requirements and CE marking from 11 Dec 2027 **[S]**. This applies to almost every industry above that sells into the EU.
- **Sector rules:**
  - IEC 62443 (industry, utilities, charging);
  - EN 50155 and EN 50121 (rail);
  - grid codes (energy);
  - IEC 62304 (medical).

## 5. What this means for weft

New requirements and priority changes. All **[J]**, to be confirmed and then moved to docs/04 and docs/08.

1. **Link-aware transfers.** Maritime, rail and metered cellular need a transfer policy: which link classes (`metered`, `satellite`, `unmetered`, `local`) may carry which artifact sizes, next to the activation gates. Delta updates (SWUpdate delta/zchunk, OCI layer dedup) move up in priority.
2. **Component ordering.** Charging and energy need "update A before B" and "only activate A and B together". This is an executor-side plan from signed dependencies in the deployment document. The verifier only checks that they were signed. Today this is a gap.
3. **Domain-protocol coexistence.** OCPP, IEEE 2030.5 and DNP3 already carry their own firmware or config commands. weft should provide adapters that turn such commands into signed weft documents (or refuse them), rather than ignore them.
4. **Modbus first.** The first adapter demo should be Modbus RTU/TCP to *weft pub/sub* (the weft envelope and topic namespace), not to one transport. The same adapter then runs over Zenoh, MQTT or NATS by configuration. It covers the most industries. OPC UA comes second. See §6 for how gateways do this today.
5. **Targets** (decided 2026-10-18):
   - x86-64, arm64 and riscv64 for Linux;
   - Cortex-M4F/M7 (`thumbv7em`), Cortex-M33 (`thumbv8m.main`) and RISC-V MCUs (`riscv32imac`, e.g. ESP32-C3/C6) for bare metal.

   weft-verifier CI builds all six. Xtensa ESP32s (ESP32, S3) need Espressif's forked toolchain and are out of scope for now.
6. **Hardware reference set for CI and demos:**
   - i.MX93 or i.MX8M Plus;
   - Raspberry Pi CM5 (cheap, widely available);
   - one x86 IPC;
   - STM32U5 or nRF54;
   - ESP32-C6.
7. **Offline / on-prem as the documented default deployment**, with cloud as one option. A bridge to AWS IoT Core (most common vendor-cloud base) is the first cloud integration to offer.
8. **Inventory and attestation** (utilities): devices report what runs on them, signed. We already have reported state; add it to the "first demo" list.
9. **First target industries:**
   - **distributed energy** and **industrial gateways for machine builders**: they have the most fit criteria, large fleets, and the CRA pressure now;
   - **maritime/remote utilities** second, as the showcase for offline operation.

## 6. How industrial gateways usually work

A summary of how common gateways work, for example HMS Ewon Flexy, Moxa AIG/UC, Advantech WISE/ECU, Siemens IOT2050, Teltonika TRB, and software gateways such as Kepware or Ignition Edge on an industrial PC. Mostly **[J]**, from product documentation.

1. **Hardware:**
   - ARM (sometimes x86) on a DIN rail, 24 V DC, −40…+70/85 °C;
   - 1–4 RS-232/485 ports, 2+ Ethernet ports, digital I/O;
   - optional LTE modem, often eMMC plus an SD card.
2. **Southbound polling engine:**
   - **device templates** (register maps: address, type, scaling, unit), one per device model;
   - a scan rate per tag group;
   - the gateway is the Modbus master and polls in a loop.
3. **Tag database:** every value becomes a *tag* (name, value, quality, timestamp). This is the internal data model all other parts use.
4. **Processing:**
   - deadband / report by exception;
   - scaling, simple logic or scripts (BASIC on Ewon, Node-RED or Python on newer ones);
   - alarms.
5. **Store and forward:**
   - a local buffer, a ring buffer on flash, holding hours to weeks;
   - it is replayed when the link comes back.
6. **Northbound connectors:**
   - MQTT (often Sparkplug B), OPC UA server, REST;
   - SDKs for the vendor cloud and for AWS or Azure.
7. **Network role:**
   - often dual-homed: one NIC in the machine network, one in the plant or IT network, with **no routing** between them, so data flows only through the tag layer;
   - sometimes a firewall/NAT router with a VPN.
8. **Remote access:** a vendor VPN service (Ewon Talk2M, Siemens SINEMA RC, Teltonika RMS) for remote programming of PLCs. This is often the main reason it is bought.
9. **Management:**
   - a web UI per device, or the vendor's device manager;
   - firmware updates are usually whole-image, by the vendor;
   - config is an exported file;
   - newer models run Docker or containers.

**What follows for weft:**

- The gateway is an ordinary weft host (embedded Linux profile).
- Each part above becomes a swappable app:
  - southbound adapters (Modbus, OPC UA client, BACnet), configured with a signed **device template + tag mapping**, which is plain weft config;
  - the tag layer is weft pub/sub (tag = topic, quality and timestamp in the envelope);
  - store and forward is the ingestion buffer on the device;
  - northbound bridges (MQTT/Sparkplug, OPC UA server, cloud).
- The no-routing protocol break is a network conduit policy (docs/08).
- Vendor VPN remote access is replaced by the overlay with per-device identity and audited, time-bounded grants.
- Existing gateways can stay and integrate as a northbound source (MQTT/Sparkplug) without weft on them.

## Sources

- Eclipse Foundation, 2024 IoT & Embedded Developer Survey: https://newsroom.eclipse.org/news/announcements/eclipse-foundation-unveils-2024-iot-embedded-developer-survey-results, https://adtmag.com/articles/2024/12/03/eclipse-foundation-survey.aspx
- JetBrains, State of Embedded 2025: https://lp.jetbrains.com/the-state-of-embedded-2025
- Edge IR, 2025 edge deployment survey: https://reimagine.edgeir.com/2025-marks-a-shift-data-sovereignty-and-ai-drive-the-next-phase-of-edge-deployment-20251116
- HarfangLab, European businesses and sovereignty: https://harfanglab.io/blog/strategy/report-european-businesses-sovereignty/
- ARM vs x86 gateways: https://www.computer-automation.de/industrial-computing/the-intelligence-at-the-edge.htm, https://www.dusuniot.com/?p=26078
- Energy: https://www.scalosoft.com/case-studies/building-a-smarter-solar-infrastructure-through-embedded-engineering/, https://files.sma.de/downloads/SCUP-Remote%20Service-TI-en-10.pdf, https://helpcenter.solarmanpv.com/portal/en/kb/articles/firmware-upgrade, https://solectria.com/support/documentation/inverter-datasheets/remote-access-portal
- EV charging: https://hackernoon.com/aws-iot-core-in-production-how-we-built-ota-firmware-pipelines-for-300-physical-ev-chargers, https://josev.ecog.io/docs/references/compatible_hardware_examples, https://my.avnet.com/ebv/manufacturers/m/nxp/ev-charging-station-solutions/, https://autotechinsight.spglobal.com/news/5269978/nxp-partners-with-lumissil-and-carmedialab-to-develop-ev-charging-system-reference-design-platform, https://www.einfochips.com/blog/exploring-the-architecture-of-electric-vehicle-charging-and-ocpp-protocols/
- Water: https://www.cisco.com/c/en/us/solutions/design-zone/industrial-iot-cisco-validated/utilities-and-renewables/water-utilities-cisco-reference-guide.html, https://www.wwdmag.com/home/article/10919556/mission-communications-high-tech-in-the-low-country, https://pumpsandsystems.com/redundant-connections-replace-outdated-leased-lines-wastewater-scada-system, https://www.waterandwastewater.com/?p=83959
- Maritime: https://www.rivieramm.com/news-content-hub/oldendorff-installs-starlink-across-bulk-carrier-fleet-89165, https://offshore-mag.com/vessels/article/55134619/marlink-marlink-upgrades-network-for-solstad-offshore-fleet, https://satnews.com/2023/10/10/nyk-line-selects-marlink-to-equip-fleet/, https://www.resilio.com/blog/starlink-with-resilio
- Rail: https://owasys.com/owasys-achieves-en50155-comprehensive-railway-certifications/, https://railway-news.com/nexcom-always-connected-in-motion-vtc-1040-and-nrok-1040-power-mobile-gateways-from-road-to-rail-2/, https://www.electronicspecifier.com/news/events-news/eurotech-presents-products-for-rolling-stock-at-innotrans-2016/
- Robots: https://husarion.com/blog/tags/zenoh/, https://vimeo.com/1136375669, https://roscon.ros.org/fr/2025/img/workshops/workshop04.pdf
- Buildings: https://buildings.honeywell.com/au/en/products/by-category/building-management/controllers/global-controllers/ecg100-edge-data-managers, https://www.automatedbuildings.com/2025/08/how-linux-is-transforming-every-data-layer-in-building-automation/
- Agriculture: https://pmc.ncbi.nlm.nih.gov/articles/PMC7180486, https://www.industrialshields.com/blog/arduino-raspberry-pi-industrial-news-industry-4-0-iot-11/lorawan-for-smart-farming-a-field-to-server-architecture-with-industrial-shields-500
- Retail: https://oneuptime.com/blog/post/2026-03-20-k3s-retail-edge/view, https://www.shopify.com/hk-en/enterprise/blog/edge-computing-in-retail
- Platforms: https://www.theregister.com/2024/02/21/microsoft_azure_iot_central/, https://www.davra.com/resources/microsoft-retires-azure-iot-central/
- CRA: https://www.pwc.ch/en/insights/regulation/eu-cyber-resilience-act.html, https://www.dataguard.com/cyber-resilience-act/timeline/
