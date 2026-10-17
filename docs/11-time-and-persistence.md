# Time handling and atomic persistence (draft design)

## Atomic persistence

**Linux (rename protocol)**: write the new file in the *same directory* → `fsync(file)` →
`rename(tmp, final)` (atomic on POSIX) → `fsync(directory)`. After any crash: old or new, never torn.

**MCUs / raw flash**: two slots with sequence number + checksum (read the highest valid one), or
Zephyr's power-loss-safe backends (NVS, ZMS, littlefs).

**Document store**: content-addressed (file name = digest), immutable, idempotent writes. Only the
small state file needs the atomic swap.

**Order per step (hard rule)**:
1. store newly accepted documents,
2. **write the new verifier state atomically**,
3. execute instructions,
4. delete superseded documents.

Committing state before acting closes a rollback window (crash after install but before state
commit would let an attacker get an older document accepted). Crashing between 2 and 3 is harmless:
instructions are idempotent and level-triggered, the verifier re-issues them.

**Unreadable state**: never start fresh silently (loses anti-rollback). Fall back to the second copy;
if both fail, safe mode (accept nothing new, report "state lost", wait for an operator), or accept
only documents above a minimum version baked into the firmware.

**Testing**: fault injection (kill at random points, dm-flakey, QEMU power cuts); assert state is
always old or new.

## Time

| Need | Clock |
|---|---|
| Ordering documents, anti-rollback | **None: version numbers** |
| Timeouts, retries, backoff, health deadlines, watchdog, rate limits, heartbeats, local dwell times | **Monotonic** |
| Delegation validity windows, X.509 validity in TLS | **Wall time, lower bound only** |
| Freshness (freeze-attack detection) | Wall time |
| Scheduled changes, maintenance windows | Wall time |
| Telemetry, log and event timestamps | Wall time **with quality flag** (+ boot id + monotonic for later correction) |
| Ordering across devices | Hybrid logical clock |
| Expiry of external tokens | Wall time |

Rules:
1. Versions order, clocks don't.
2. Wall time is a **lower bound that only moves forward** (NTS, Roughtime, RTC, signed `issued_at`,
   planned signed timestamp documents). Plain NTP only as fallback, forward only within limits.
3. Failure direction per check: delegation expiry is lenient with an old floor (accept late, never
   refuse valid); freeze detection alarms but does not stop the device.
4. Prefer relative durations ("valid 3,600 s after receipt", monotonic); after reboot treat as expired.
5. Report time quality (NTS-synced / RTC only / unknown) in reported state and telemetry.
6. Clocks: Linux `CLOCK_BOOTTIME` for durations including suspend (Rust `Instant` uses
   `CLOCK_MONOTONIC`, which stops during suspend); `CLOCK_REALTIME` for wall time; Zephyr `k_uptime`
   + floor in flash.
7. UTC only, except maintenance windows: authoring tools convert local windows to UTC ranges per
   period (time-zone rules in documents only if ever needed).
