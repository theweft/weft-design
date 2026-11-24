# CM5 lab carrier: every signal, under an agent's control (draft)

A carrier board for the Raspberry Pi Compute Module 5 that exposes every CM5 signal and lets an
agent (Claude Code, labgrid, CI) work on the module without a person present: switch and measure
its power, drive its boot and control pins, read its consoles, flash it, see its screen, and reach
every low-speed signal. It is weft's first board designed AI-first (docs/09, "AI-first circuit
design") and the reference device for the CM5 twin (docs/04) and the board simulation (docs/21).
Repository: [weft-cm5-carrier](https://github.com/theweft/weft-cm5-carrier).

Status: requirements and architecture, decided 2026-11-24 with the maintainer; parts are
candidates until checked against datasheets and stock. Nothing is laid out yet.

## 1. Decisions

| Question | Decision | Why |
|---|---|---|
| High-speed lanes (2× HDMI, 2× USB 3, 2× 4-lane MIPI, PCIe, GbE: about 70 of 200 pins) | **Native connectors**, impedance-controlled, with test points; everything else to headers | On a 2.54 mm header the pairs would not work at speed |
| Control | **An on-board supervisor that handles everything a microcontroller can** (§3.3), reached over one USB-C from the PC | A supervisor that reaches only some signals leaves the agent depending on a person for the rest |
| Starting point | **Official CM5 IO Board (KiCad) plus code**: reuse its connector footprints and high-speed sections; design the new blocks as code and merge them into one KiCad project | Proven layout where it is risky, AI-first where AI is strong |
| Power input | **USB-C PD, 5 V 5 A, negotiated by the carrier** (§3.1) | One cable, like the IO Board and the Pi 5 supply; the carrier keeps the contract while the CM5 is cut off |
| Size | **Larger than the IO Board** where the headers, supervisor and base need it | Probing room matters more than fitting the IO Board case |
| Base | **A stable base plate** under the board (§4) | It sits on a table and stays put; no loose cables |
| Signals on the module itself | **Reached too**: the test points on the CM5's underside (debug UART/SWD, RUN, nRESET_OUT and more) through pogo pins, using the 4.0 mm stacking connector; the U.FL antenna through a pigtail to the base (§3.4) | The boot console, SoC debug and a hard reset exist only there |
| Licence | **CERN-OHL-P-2.0** for the hardware design files, **MIT** for firmware and scripts | CERN-OHL-P is the permissive licence written for hardware; weft code stays MIT |

The IO Board's design files may be used, modified and distributed (Raspberry Pi's licensing page,
docs/21 §6); its bundled 3D models stay out. Its stackup is kept: 4 layers, 0.09 mm prepreg
between the outer and inner layers, which its impedance-controlled pairs rely on.

## 2. Requirements

- **R1 Every signal reachable by hand.** Every one of the CM5's 200 connector pins ends at a header
  pin, a test point or a native connector, labelled with its CM5 name. Ground pins: test points
  and ground pins spread through the headers. The module's own test points that the connectors
  do not carry (§3.4) end at a header too.
- **R2 Every low-speed signal reachable by the supervisor:** all 28 GPIOs individually (drive,
  read, capture), every control and status pin, the SD/eMMC bus (switchable between CM5 and PC),
  and the control side of every high-speed interface (§3.3).
- **R3 Power under control.** The supervisor switches the CM5's 5 V on and off (a real power cut
  with fast discharge, for power-loss tests), switches the USB 3 port power, the M.2 supply and the
  RTC battery, and measures voltage and current on the 5 V input, the CM5's 3.3 V and 1.8 V
  outputs, the USB ports and the M.2 supply.
- **R4 One cable for the agent.** One USB-C from the PC carries the supervisor (control, consoles,
  measurement), the CM5's USB 2.0 OTG port (rpiboot flashing, gadget mode), the SD card reader and
  the HDMI capture.
- **R5 Never back-power the CM5.** "No pins should be powered before the 5 V rail is active"
  (CM5 datasheet §3.1): nothing on the carrier drives a CM5 pin high while the CM5 is off.
- **R6 The datasheet's power-up rules hold under supervisor control:** EEPROM_nWP low before
  power-up when protection is wanted; nRPIBOOT low within
  <!--f:ext.cm5_rpiboot_window-->2 ms<!--/f--> after 5 V rises for USB boot; 5 V rising
  monotonically to at least <!--f:ext.cm5_vin_min-->4.75 V<!--/f-->.
- **R7 Loads within limits.** The CM5's 3.3 V and 1.8 V outputs supply
  <!--f:ext.cm5_rail_out_ma-->600 mA<!--/f--> each and the GPIO bank
  <!--f:ext.cm5_gpio_total_ma-->50 mA<!--/f--> in total; carrier loads on them are budgeted and
  measured, and the supervisor's translators draw from its own supply.
- **R8 Usable by hand too:** power button, status LEDs, RTC battery, fan connector, all interfaces
  of the IO Board, and a manual override for every supervisor function that a person might want.
- **R9 Machine-checkable design:** the schematic comes from code and a KiCad project; ERC, DRC,
  weft-boardchecker and the board simulation run in CI (docs/09, docs/21).
- **R10 Stable on a table** (§4): base plate, all cables entering at one edge, probing from the
  top and front.

## 3. Architecture

```
 PC ── USB-C "lab" ──► USB 2.0 hub (4 ports) ─┬─► supervisor (RP2350B)
                                              ├─► USB mux ──► CM5 USB 2.0 OTG (rpiboot, gadget)
                                              ├─► SD card reader ◄─ SD mux ─► CM5 SD bus
                                              └─► HDMI capture (HDMI0, behind a splitter or mux)

 PSU ── USB-C "power" ──► PD sink ──► eFuse (supervisor EN, limit) ──► CM5 5 V ──► discharge

 supervisor ──► open-drain drivers ──► PMIC_EN, PWR_BUT, nRPIBOOT, EEPROM_nWP, WL/BT_nDisable
 supervisor ◄─► per-pin translators (isolate when the CM5 is off) ◄─► all 28 CM5 GPIOs
 supervisor ◄─► I2C: power monitors, IO expanders, EDID emulation on HDMI DDC, camera I2C
 supervisor ──► load switches: USB 3 VBUS, M.2 3.3 V, VBAT, Ethernet disconnect, GPIO_VREF select

 CM5 high-speed lanes ──► native connectors (2× HDMI, 2× USB 3 A, M.2 M-key, 2× MIPI FPC, RJ45)
 CM5 low-speed signals ──► labelled 2.54 mm headers + test points (+ logic analyser header)
```

### 3.1 Power input and the CC question

On the IO Board, USB-C VBUS feeds the CM5's 5 V directly and CC goes to the CM5, which negotiates
5 V/5 A itself. With a switch in the 5 V path, the CM5's CC termination may disappear while it is
off; a PD source can then remove VBUS and the carrier could never switch the CM5 on again. The
datasheet does not say how CM5's CC behaves unpowered.

Decided (2026-11-24, maintainer): **the carrier negotiates 5 V itself**, so it can take the CM5
off its supply completely without losing the PD contract. A **PD sink controller with dead-battery
support** (its CC termination stays when unpowered), powered from VBUS before the switch, holds the
contract; the CM5's CC pins stay unconnected and the CM5 is told the supply's capacity in software
(`PSU_MAX_CURRENT` in the bootloader configuration). To verify on hardware: that the CM5 boots
with CC unconnected and accepts the setting.

The supervisor is powered from the "lab" USB-C (always present when an agent is connected) and,
OR-ed, from the power input before the switch, so it keeps running across CM5 power cuts.

### 3.2 Never back-powering (R5)

- Control outputs are **open-drain only** (pull low or release): PMIC_EN, nRPIBOOT, EEPROM_nWP and
  WL/BT_nDisable have pull-ups inside the CM5; PWR_BUT is pulled up to 5 V inside (10 kΩ), so its
  driver must stand 5 V.
- The GPIO translators and every other signal path into the CM5 **are powered on the CM5 side from
  GPIO_VREF (or the CM5 rail of that signal) and isolate both sides when either supply is off.**
- Pull-ups on the carrier go to CM5 rails, not to the supervisor's rail.
- The board simulation checks this rule on the schematic: no driven-high net into a CM5 pin while
  the CM5's 5 V net is off (a finding, docs/21).

### 3.3 Supervisor: what "everything" covers

Everything a microcontroller can carry is wired to the supervisor; the high-speed data itself
(HDMI pixels, PCIe, USB 3, MIPI and Ethernet traffic) is not, and the supervisor controls those
interfaces around the data instead.

| Area | Supervisor can | How |
|---|---|---|
| Power | cut and restore the CM5's 5 V (real power cut), set the current limit, see faults; switch USB 3 VBUS per port, the M.2 3.3 V and the RTC battery; measure all of them | eFuse, discharge switch, load switches, power monitors |
| Boot and control | PMIC_EN, PWR_BUT, nRPIBOOT, EEPROM_nWP, WL_nDisable, BT_nDisable, in timed sequences | open-drain drivers |
| GPIO_VREF | choose 3.3 V or 1.8 V signalling, only while the CM5 is off, never floating | supply selector (power mux) |
| All 28 GPIOs | drive, read and capture each one; emulate peripherals (UART, I2C, SPI targets, a HAT ID EEPROM); logic analyser; fault injection (stuck-at, glitches) | per-pin translators with isolation, on RP2350B PIO pins |
| Consoles | CM5 UART0 and any other GPIO UART as USB serial ports | through the GPIO path |
| Status | LED_nACT, LED_nPWR, VBUS_EN, PCIE_PWR_EN, PCIE_nWAKE, Ethernet LEDs, SYNC_OUT, FAN_PWM; emulate FAN_TACHO | IO expander and translators |
| USB 2.0 OTG | connect the CM5's OTG port to the PC (rpiboot flashing, gadget mode) or to a host port | USB mux behind the hub |
| SD/eMMC bus | give the SD card to the CM5 or to the PC (reflash a CM5 Lite without touching it) | SD mux plus USB card reader behind the hub |
| HDMI | emulate a monitor (EDID over DDC, hot plug), CEC; see the picture | supervisor I2C on DDC (level-shifted to 5 V); USB HDMI capture behind the hub |
| MIPI | camera control I2C (SDA0/SCL0) and CAM_GPIO: emulate a sensor's registers | translators; no image lanes |
| PCIe/M.2 | power, reset, wake, current | load switch, IO expander |
| Ethernet | disconnect and reconnect the link (network partition tests), see link LEDs | GbE-rated analog switch on the MDI pairs |
| Buttons and fan | press PWR_BUT, read the fan PWM, fake a fan tachometer | as above |
| Module test points (§3.4) | boot console (debug UART), SoC debug over SWD (same two lines, after `enable_jtag_gpio=1`), hard reset (RUN), see resets (nRESET_OUT) | pogo pins under the module, translators; CMSIS-DAP probe firmware on the RP2350 |

Supervisor: RP2350B (48 GPIO) at 3.3 V I/O, QSPI flash, crystal, its own SWD header. The 28 CM5
GPIOs take 28 of its PIO-capable pins; slow control and status go through I2C IO expanders.
Firmware in Rust, with an agent interface over USB (a small command protocol, then a labgrid
driver) and USB serial ports for the consoles.

The supervisor cannot be a second USB device towards the CM5 (its one USB controller faces the
PC). Keyboard or mass-storage emulation into the CM5 would need a second small MCU on one of the
CM5's USB ports; left as an option.

### 3.4 Signals on the module itself

Besides its two 100-pin connectors, the CM5 has (datasheet §2.8, §4.1.2, Appendix A):

- an optional 3-pin, 1 mm JST-SH **debug connector** (BM03B-SRSS-TB, often not fitted) that
  "provides the same functionality as Raspberry Pi 5": the BCM2712's debug UART (boot console) by
  default, and the ARM debug port as **SWD** (SWDIO, SWCLK) on the same lines when
  `enable_jtag_gpio=1` is set in config.txt (on the Pi 5 this option no longer uses GPIO22 to 27).
  Its two signals are also test points TP35 and TP36;
- **test points on the underside**, at datasheet coordinates relative to the module: RUN (TP2),
  nRESET_OUT (TP21), nRPIBOOT, PWR_BUT, 5v_Sense, the USB 2.0 data lines, the Ethernet pairs,
  GPIO_VREF, the LEDs, power and ground, and five pins named SOC_TRST_N, SOC_TDI, SOC_TDO, SOC_TMS
  and SOC_TCK (TP48 to TP52). Raspberry Pi documents no use for these; the documented debug path is
  SWD on the debug connector, so they only go to a header for experiments;
- a **U.FL connector** for an external Wi-Fi/Bluetooth antenna (selected at boot in config.txt).

The carrier reaches the test points with **spring-loaded pogo pins** under the module. This needs
the 4.0 mm stacking connector (Amphenol 10164227-1004A1RLF: 2.5 mm clearance under the CM5,
instead of none with the 1.5 mm part), and pogo pins short enough for that gap. The supervisor
uses TP35/TP36 as the boot console and, when enabled, as SWD (both on the same two lines, which
the RP2350 can switch between; Raspberry Pi's Debug Probe firmware already does CMSIS-DAP and UART
on this chip family), RUN (hold in reset without a power cut) and nRESET_OUT (see every reset); the
rest go to a header for probing. The datasheet warns that component positions may change slightly
between module revisions, so the pogo pins need tolerance and the footprint is checked against the
module on hand. The U.FL antenna goes through a pigtail to a bulkhead SMA on the base plate.

### 3.5 Pin allocation

| CM5 group (pins) | Goes to |
|---|---|
| GND (51) | ground plane, test points, ground pins in every header |
| 5 V in, 3.3 V and 1.8 V out, VBAT (11) | power path above; test points; monitors |
| GPIO0-27 incl. ID_SC/ID_SD (28) | the 40-pin HAT header and 2× 20-pin breakout headers in Raspberry Pi order; every line also to the supervisor through translators |
| SD/eMMC (13), SD_PWR_ON, SD_VDD_OVERRIDE | microSD slot through the SD mux; header with test points |
| control (PMIC_EN, PWR_BUT, nRPIBOOT, EEPROM_nWP, WL/BT_nDisable, CC) | supervisor drivers; header with test points |
| status (LED_nACT, LED_nPWR, VBUS_EN, PCIE_PWR_EN, PCIE_nWAKE, FAN_PWM/TACHO, Ethernet LEDs, SYNC_OUT) | LEDs, fan connector, supervisor; header |
| HDMI0/1 (24 incl. DDC, CEC, hot plug) | 2× HDMI; DDC, CEC and hot plug to the supervisor and a header; HDMI0 also to the capture |
| USB 3.0 ×2 (12), USB 2.0 OTG (3) | 2× USB 3 A with supervisor-switched VBUS; the OTG mux |
| PCIe (8 + control) | M.2 M-key with a supervisor-switched 3.3 V supply |
| MIPI0/1 (20 + CAM_GPIO) | 2× 22-pin FPC; CAM_GPIO and SDA0/SCL0 to the supervisor and a header |
| Ethernet (8 pairs + LEDs) | RJ45 MagJack through the disconnect switch |

### 3.6 Part candidates (to check)

| Block | Candidate | Note |
|---|---|---|
| Supervisor | RP2350B + W25Q16JV + 12 MHz crystal | weft's lab MCU family (docs/17) |
| USB hub | 4-port USB 2.0 hub (Microchip USB2514B class) | high speed for rpiboot and capture |
| USB mux | TS3USB221 class | CM5 OTG to hub or host port |
| SD mux and reader | TS3A27518E class SD switch; GL823K or USB2244 class card reader | as in USB-SD-Mux designs |
| HDMI capture | MS2130-class USB capture chip, or a USB capture stick on an internal port | HDMI0 through a passive splitter is not possible; a 1:2 HDMI switch or redriver feeds monitor or capture |
| PD sink | a controller with dead-battery Rd and 5 V/5 A request | see 3.1 |
| 5 V switch | eFuse in the TPS25947 class: 5 A+, adjustable limit, current monitor, power good | plus a discharge MOSFET |
| Load switches | TPS22918 class for USB VBUS, M.2 3.3 V, VBAT | supervisor-controlled with the CM5's own enables OR-ed in |
| Power monitors | INA228 on 5 V in; INA3221 or INA228s on 3.3 V, 1.8 V, USB, M.2 | shunts sized for 5 A and 600 mA |
| GPIO translators | per-pin direction (SN74LXC1T45 class) or auto-direction with isolation (NTS0104/TXS0108E class, which tolerate the CM5's 1.8 kΩ pull-ups on GPIO2/3) | to decide in the schematic: drive strength against simplicity |
| I2C translators | PCA9306/TCA9517 class; 5 V on HDMI DDC | |
| GPIO_VREF select | TPS2116 class supply selector | switches only while the CM5 is off |
| Ethernet disconnect | TS3L501E class GbE LAN switch | check insertion loss at 1 Gbit/s |
| IO expanders | TCA6424 class | slow control and status |
| Module test points | low-profile pogo pins (compressed height under 2.5 mm), 4.0 mm stacking connector 10164227-1004A1RLF | check height and pad size against a real CM5 |
| Reused from the IO Board | DF40 connectors, HDMI, USB 3 switch (AP22653), M.2 buck (AP3441), RJ45 MagJack, ESD (TPD4EUSB30), RTC crystal | as designed by Raspberry Pi |

## 4. Base and mechanics (R10)

- The board sits on four or more standoffs on a **base plate** larger than the board: aluminium
  (laser-cut) or 3D-printed with a steel insert for weight, with rubber feet. Plate drawings live
  in the repository with the board.
- **All cables enter at one edge** (the back): power USB-C, lab USB-C, Ethernet, HDMI, USB 3, so
  the board stays put and the cables can be tied to the plate.
- **Probing from the top and front:** headers, test points and the logic analyser header face the
  user; the CM5 and its cooler sit where they do not block them.
- An optional clear top cover with cut-outs for the headers.

## 5. How it is built (AI-first, docs/09)

1. **Chip data first:** CM5 (connector pinout from the datasheet and the IO Board symbol), RP2350B
   and the new parts as weft-parts chip files, so design, checks and simulation share one source.
2. **New blocks as code** (tscircuit, SKiDL as fallback): supervisor, power path, measurement,
   translators, muxes, headers. **Reused blocks from the IO Board's KiCad sheets.** Both merge into
   one KiCad project; the netlist is regenerated from code on every change.
3. **Checks in CI:** KiCad ERC; weft-boardchecker; the board simulation with the R5 back-power rule
   and the power-up sequence of R6; ngspice for the power path (eFuse limit, discharge time).
4. **Layout in KiCad:** the reused high-speed sections keep the IO Board's placement and routing;
   new blocks are placed by hand or by an agent through KiCad's IPC API, routed with Freerouting
   or by hand; DRC in CI; fabrication outputs with KiBot.
5. **Bring-up as tests:** supervisor firmware first, then power, then CM5 boot with rpiboot, every
   step a scripted test an agent can repeat.

## 6. Open questions

- Verify on hardware that the CM5 boots with CC unconnected and takes `PSU_MAX_CURRENT` (3.1).
- GPIO translator type (3.6): per-pin direction or auto-direction.
- HDMI capture: an on-board capture chip or a USB capture stick on an internal hub port.
- Keyboard and mass-storage emulation into the CM5 (a second MCU, 3.3).
- Pogo pins: a part short enough for 2.5 mm clearance, and how the test point positions vary
  between CM5 revisions (3.4).
- Whether TP48 to TP52 (named as JTAG pins) do anything on production modules.
