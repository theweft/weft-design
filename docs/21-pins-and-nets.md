# Pins and nets: how simulated chips connect (draft)

How weft models the physical pins of chips and the nets between them, so that boards (a HAT, a
CM5 carrier, an MCU next to a Linux SoC) can be simulated from the same design files the PCB is
made from. Builds on the time-aware models ([04-decisions.md](04-decisions.md), "Time in the
models"). Status: proposal; the first step (digital nets for I2C chips' control pins) is
implemented, see §10.

## 1. Principle: nets carry signals, interfaces carry transactions

- **The net is the unit.** Which pins share a net comes from the board design (§3). A model never
  talks to another model directly: it drives and reads its own pins, and the simulation resolves
  each net.
- **A net carries one of four signal kinds:**

  | Kind | Value | Several drivers on one net |
  |---|---|---|
  | Digital | level plus drive: push-pull high/low, open-drain low, high-Z, pull-up/down | wired logic: open-drain low wins, pulls apply to high-Z, two push-pulls that disagree are a contention error |
  | Analog | voltage as a function of time: constant, piecewise linear, waveform | one driver per net, plus a DC solver for resistor networks (§8); anything beyond goes to ngspice |
  | Periodic | frequency, duty, origin, deviation in ppm | edges are computed from the parameters only when someone needs them (PWM, clocks) |
  | Power | voltage, state (off, ramping, on), source | regulator models drive it; chips' power states follow it |

- **Interfaces sit on top of nets.** An interface groups nets (I2C: SDA and SCL; SPI: SCLK, MOSI,
  MISO, CS) and normally carries transactions (bytes, frames, packets) at the loosely timed level.
- **Model a link at the highest level both ends understand.** Linux to the ADS1115 model stays at
  transfer level. Only when one end works on bits (bit-banging, RP2040 PIO, a simulated logic
  analyser, a fault injected on one bit) does a transactor convert between transactions and edges.

## 2. Pin types

| Pin type | Level | Representation | Notes |
|---|---|---|---|
| GPIO | digital net | timestamped edges, drive, pulls | interrupt = edge; no natural lookahead, so tight sync or an assumed interrupt latency; Linux side via gpio-sim (weft-sim) |
| Analog in | analog net | the model samples the net's voltage function when its conversion completes | fixes the ADS1115 sampling-at-next-bus-event limit; source impedance and sampling capacitor errors are out of scope (ngspice) |
| Analog out | analog net | DAC drives a voltage, optional settling time | power-down resistors become loads in the DC solver |
| PWM in, PWM out | periodic | parameters change as single events; capture and tachometer models read them directly | a 25 kHz signal would be 50 000 edge events per second; RC filter to analog = duty × VDD (ideal) |
| UART, RS-485 | interface | timestamped bytes plus line settings per end | mismatched baud or framing gives framing errors or garbage; RTS/CTS and DE are digital nets; RS-485 multi-drop with collisions; lookahead one byte |
| SPI, QSPI | interface | frames per chip-select assertion, mode, bit order, speed, lanes | CS is a digital net (may be a GPIO); a mode the chip does not support (chip data) is an error |
| I2C, SMBus, PMBus | interface | exists (weft-parts) | later: multi-controller arbitration, clock stretching, 10-bit addresses, SMBus alert and PEC |
| CAN, CAN FD | interface | frames; arbitration by ID among frames starting at the same virtual time | unacknowledged frames, error counters, bus-off; the transceiver is its own model with a standby pin; QEMU and Renode have CAN models |
| Ethernet | frames, not pins | the emulators carry frames into the network simulation (Containerlab, turmoil) | MDIO is modelled at pin level: PHY models in weft-parts answer it like I2C chips, so Linux PHY drivers probe; link state is model state |
| USB | packets and transfers | USB/IP as the transport (QEMU and Renode support it) | VBUS is a power rail, ID a digital net; USB-C CC/PD as messages, later |
| Video (HDMI, DP, VGA) | control side only | no pixels at pin level | EDID over DDC is an I2C EEPROM (at24 model), HPD a digital net, CEC a slow single-wire interface |
| JTAG, SWD | debug interface | normally replaced by the emulators' gdbstub | when one device drives another's (CM5 flashing an MCU or FPGA, openocd bit-banging GPIOs): edges, a decoder, then memory writes into the target emulator; later |
| 1-Wire, I3C, MDIO, LIN, Modbus RTU, SDIO/eMMC, QSPI flash | interface | transactions like I2C/SPI | flash models get erase and program times like the at24 write cycle |
| I2S/PCM/TDM, MIPI CSI/DSI | streams | sample blocks or frames | control registers over I2C (camera sensors, display bridges) are ordinary weft-parts models |
| WS2812, SENT, IR | edges only | need a transactor | typically driven by PIO or SPI tricks |
| Quadrature encoder | periodic | two signals 90° apart; direction is the sign of the phase | |
| PCIe | interface | vfio-user | see the CM5 twin direction |
| Radio (BLE, LoRa, Wi-Fi) | packets | never at pin level | Renode can do BLE |

## 3. Control pins

Digital nets whose meaning lives in the chip model, in four patterns:

1. **Enable and power-down** (EN, PMIC_EN, GLOBAL_EN): switch the power state of a chip or rail.
2. **Reset** (active low, minimum pulse width): volatile state is lost; pulses that are too short
   are ignored or flagged.
3. **Straps and boot mode, sampled once when reset is released.** The model reads the net at that
   instant only. Address straps, already data in weft-parts, become dynamic too. On the CM5,
   `nRPIBOOT` low forces USB boot (rpiboot) instead of eMMC.
4. **Gates on operations:** write protect (`EEPROM_nWP`, the 24C32 WP pin) makes the chip refuse
   data bytes; MCP4728 LDAC becomes a pin. Outputs: power-good, interrupt and alert lines
   (open-drain), status LEDs.

## 4. Power and ground

- A rail has a source (regulator model: enable, ramp time, power-good, sequencing); a chip's power
  state follows its supply pins.
- **Below its brown-out threshold a chip resets and loses volatile state; non-volatile memory
  stays.** This brings power-loss tests to chip level: cutting a rail during an at24 write cycle
  leaves the page partly written (seeded, reproducible), and the same for flash erase and eMMC. It
  is a real crash in the sense of [04-decisions.md](04-decisions.md) (L1).
- Optional current per chip state for power budgets (battery-powered M1 devices): an estimate.
- GND is a reference and a connectivity check. Separate reference domains (isolated CAN or RS-485)
  are tracked; a signal crossing domains without an isolator is an error.
- Not modelled: ripple, transients, inrush current, ground offsets (ngspice where needed).

## 5. Further pin properties

- **Clock inputs** (crystal, oscillator): a periodic net with frequency and ppm that feeds the
  chip's `ClockDomain`, so an MCU's 8 MHz ±20 ppm crystal shows up in its UART baud rate.
- **Reference voltages** (VREF): analog nets that scale ADC and DAC models.
- **Voltage domains and level shifters:** every digital pin gets VIH/VIL and an absolute maximum
  from its chip file. A 5 V push-pull output on a 3.3 V input is an error, statically
  (weft-boardchecker) and while running. A level shifter is a pass-through model with a direction
  and two domains. Vendor IBIS models are a source for these values.
- **Passives** (pull-ups, dividers, straps) come from the board design into net resolution;
  capacitors only as RC filters.
- **Human and environment models:** button (test-driven, configurable bounce), LED (observes level
  or PWM duty), temperature as an environment value for sensors, actuators such as a relay or a
  fan with tachometer.
- **NC pins and test points:** static checks.
- **Fault injection at net level:** stuck-at-0/1, open, short to GND, analog noise, rail drop.

## 6. Where nets come from: formats

There is no industry standard for board-level behavioural simulation, but there are standards for
each piece weft needs. The simulation reads the same exports weft-boardchecker reads, plus one
schematic-level source.

| Need | Source | Standard? | Carries | Lacks |
|---|---|---|---|---|
| Connectivity of a laid-out PCB | IPC-2581 rev B/C (adopted, weft-boardchecker) | yes, open IPC standard; KiCad 8+, Altium, Allegro/OrCAD, Xpedition | components, every symbol field as a textual characteristic (Value included), pin-to-net map, layout | pin names and pin types |
| Fallback | IPC-D-356A + CSV BOM (adopted) | yes | nets and pads; part numbers from the BOM | everything else |
| Connectivity before layout | KiCad netlist (`kicad-cli sch export netlist`, S-expression or XML) | no, KiCad's own format, stable and machine-readable | every symbol field (Value, MPN, custom fields), pin names, pin electrical types (input, output, open collector, power in/out, passive, NC), hierarchical net names | layout; other EDA tools |
| Analog subcircuits | SPICE netlist (`kicad-cli ... --format spice`) | de facto (SPICE) | passives with values, device models | digital and protocol behaviour |
| Pin electrical data | IBIS (vendor models) | yes, ANSI/EIA-656 | VIH/VIL, drive strength, thresholds per I/O buffer | function |
| Several boards together (CM5 + carrier + HAT) | none fits: harness tools (WireViz) document cables, are GPL-3.0 and carry no electrical semantics | no | | |

**Checked 2026-11-24 with KiCad 10.0.8 (`kicad-cli`, nixpkgs `kicad-small`) on the official CM5 IO
Board (rev 2, 83 components, 210 nets):**

- `kicad-cli` exports everything headless: `sch export netlist` (`kicadsexpr`, `kicadxml`, `spice`,
  and others, with `--variant`) and `pcb export ipc2581`, `ipcd356` and `odb`. CI needs no GUI.
- **IPC-2581 (rev C):** all 39 BOM items carry `Value` and every other symbol field as textual
  characteristics; nets come as `<Set net="…">` blocks with `PinRef`s, the layout
  weft-boardchecker's parser already falls back to. No pin names, as expected.
- **KiCad netlist:** values (`1k`, `100n`), all fields, pin names (`PMIC_ENABLE_99`) and pin types
  per node; hierarchical net names (`/CM5_GPIO ( Ethernet, GPIO, SDCARD)/nRPIBOOT`), so the CM5
  control nets `PMIC_ENABLE`, `nRPIBOOT` and `EEPROM_nWP` are directly addressable.
- **Part numbers sit in arbitrary fields** (here `Field6` holds the MPN, `Field7` the maker), so the
  importer needs the field mapping weft-boardchecker already has.
- **Pin types are only as good as the symbol:** the CM5 symbol marks most module pins `passive`
  (`EEPROM_nWP`, `SDA0`). Electrical meaning therefore stays in weft-parts chip data; the
  netlist's types are a hint and a cross-check.
- **62 of 210 nets are auto-named** (`Net-(…)`): testbenches must use labelled nets or
  `refdes.pin`.
- **SPICE export** holds exactly the passive network with values (for example 2.2 kΩ pull-ups from
  `GPIO_VREF` to SDA1/SCL1); chips appear without models.
- **Licence:** the archive has no licence file; Raspberry Pi's
  [licensing page](https://www.raspberrypi.com/licensing/) grants "permission to use, copy,
  modify, and/or distribute these designs for any purpose" for most hosted design files, with
  third-party parts (the bundled 3D models) under their makers' terms. Exports can serve as test
  fixtures with a note on the source; the 3D models stay out.

Proposal:

1. **One tool-neutral board model, shared.** weft-boardchecker's `Board` (components, part
   numbers, pin to net) moves into a crate both the checker and the simulation use, extended with
   component values, fields, pin names and pin types where the source has them.
2. **Importers:** IPC-2581 (primary, any EDA tool, laid-out boards), the KiCad netlist (schematic
   level: pin names and types, values and fields; simulate a schematic before it is routed),
   IPC-D-356A plus BOM (fallback). SPICE exports go to ngspice for analog subcircuits, not into the
   board model.
3. **Chip models come from the part number**, as today (weft-parts lookup by MPN prefix).
   Optional per-symbol overrides in a field, following KiCad's own `Sim.*` field convention for
   ngspice models (for example `Weft.Model`), but designers should never need them.
4. **Everything about a test run lives outside the design**, the way SPICE separates circuit and
   analysis: a testbench TOML that names nets and pins (stimuli, initial values, faults, which
   emulator runs which processor). It references labelled nets or `refdes.pin`, never
   auto-generated net names such as `Net-(U3-Pad9)`, which change when the schematic changes.
5. **Several boards:** a small system file that mates connectors by designator and pin number
   (CM5 module, CM5 IO Board, HAT), with one weft-parts-style description per module connector.
   Raspberry Pi publishes the CM5 IO Board as a KiCad project (rev 2, KiCad 8), so the reference
   carrier can be imported directly; the CM5 module itself is a connector pinout plus the twin.

## 7. Model API changes (sketch)

- Chip files: pins get a `kind` (digital, analog, power, ground, clock, nc) and electrical data
  (domain, VIH/VIL, absolute maximum, drive, internal pulls, "sampled at reset release").
  Interfaces group pins (`[[interface]] kind = "i2c"`, `pins = { sda = "sda", scl = "scl" }`).
  Today's pin roles stay.
- Models declare their pins next to `I2cTarget`/`SpiTarget`. The simulation reports net changes
  with timestamps; the model drives outputs through a context. Everything uses the existing
  `advance`/`next_event` mechanism, no new time model.
- Static checks (pull-ups, domains, straps, terminations, NC) stay in weft-boardchecker; the
  simulation checks dynamic behaviour (contention, timing, power sequencing).

## 8. Analog scope

Decided 2026-11-24: **resistor networks only (DC).**

- The simulation solves the resistive network (pull-ups and pull-downs, dividers, straps behind
  resistors, DAC loads) for its DC operating point, once per change of a driver. Voltages change
  only at events, so the existing `advance`/`next_event` model is unchanged, and results are
  deterministic and cheap.
- **Configuration resistors** need no solver: a chip model reads the value from the board model
  (on the CM5 IO Board, R12 = 15 kΩ on U6's ILIM pin sets its current limit).
- **Capacitors are ignored.** Not covered: RC reset delays on enable pins, PWM-to-analog filter
  settling and ripple, ADC input settling, debounce RCs, I2C rise times. Firmware that reads too
  early is therefore not caught.
- **Possible later extension:** isolated first-order RC stages, recognised by topology and solved
  in closed form, v(t) = v∞ + (v₀ − v∞)·e^(−t/τ), evaluated when read, with threshold crossings
  as computed events. Coupled or higher-order networks, diodes, transistors and MOSFET level
  shifters stay out: that would be a circuit simulator, which ngspice already is (offline, see
  [09-prior-art.md](09-prior-art.md)).

## 9. Order

1. Digital nets, control pins, straps and reset (ALERT, LDAC, WP, CM5 control pins). Done for the
   modelled I2C chips (§10); reset, power-up strap sampling and the CM5 control pins remain.
2. Power rails with brown-out (chip-level power-loss tests).
3. Analog waveforms.
4. UART and SPI interfaces.
5. Periodic signals (PWM, clocks).
6. CAN.
7. USB and Ethernet through the emulators' transports.
8. Debug ports (JTAG, SWD).

The board-model crate and the KiCad netlist importer come first, since every step needs nets.

## 10. Implemented (2026-11-24)

In weft-parts (requirements PRT-011, PRT-012), used by weft-boardchecker:

- **The board model lives in weft-parts** (`weft_parts::board`): weft-boardchecker and the
  simulation both depend on weft-parts already, so no new crate is needed until the model needs
  its own release cadence. weft-boardchecker re-exports it; its IPC-2581, IPC-D-356A and BOM
  importers stay there, with their XML dependency. The strap address rule moved with the model,
  so the checker and the simulation derive addresses identically.
- **KiCad netlist importer** (`board::kicad`), no new dependency. Checked against the official CM5
  IO Board export (`just kicad-real`, not committed: Raspberry Pi's licence page cannot be fetched
  for a REUSE licence file). The S-expression netlist omits symbols marked `exclude_from_board`
  (78 of the board's 83 symbols; the five M.2 screws), which KiCad's XML netlist keeps.
- **Digital nets** (`sim::net`): drive (released, low, high), pull (none, up, down, both),
  resolution by wired logic, contention and floating as states.
- **Board simulation** (`sim::BoardSim`): models placed by part number, buses by SDA/SCL nets,
  addresses from straps, pulls from resistors to rails (rails from the chips' gnd/vdd nets), model
  pins on nets (ADS1115 ALERT/RDY, MCP4728 LDAC and RDY/BSY, 24C32 WP), timed edges, findings for
  floating inputs, contention and shorts against a rail. Nets settle after transfers, at model
  events and on external drives; an edge inside a transfer is recorded at its end.
- Not yet: reset and power-up strap sampling, the testbench file, SPI chips, multi-board systems,
  Linux GPIOs (weft-sim, via gpio-sim) and IPC-2581 boards in the simulation.

## Open questions

- Whether the testbench format should be shared with weft-agent's turmoil scenarios.
