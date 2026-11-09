# Time

`#import "Extensions/Time"` provides civil dates, exact instants, explicit time
zones and calendar-aware spans. Calendar differences, balancing, rounding and totals
use the same ordered arithmetic as addition, including variable-length zoned days.

## Concepts

`Timestamp` identifies an instant independently of a zone. `Date`, `Time_Of_Day` and
`Date_Time` describe the proleptic Gregorian calendar and wall clock; they do not
identify instants. ISO weeks start on Monday and week one contains January 4. Years
include zero and run from -9999 through 9999.

`Time_Zone` maps an instant to an offset east of UTC. A zone is UTC, a fixed offset,
or immutable TZif/POSIX rules. Mapping civil time back to an instant can encounter a
gap or fold. `COMPATIBLE` resolves a gap later and a fold earlier; `EARLIER`,
`LATER` and `REJECT` make the other choices explicit. No conversion implicitly uses
the machine's zone.

`Zoned` combines an instant with its zone and resolved offset. Adding calendar units
first changes its civil reading and resolves it again. Clock units then advance the
exact timeline. One day therefore preserves the wall-clock time across a DST change;
24 hours advances exactly 24 hours.

`Span` keeps ten units separately, with one shared sign. Years, months and weeks
need a relative date or zoned value to acquire an elapsed length. Zoned days also
vary in length. Span conversions involving calendar units use overloads taking
`relative: Date` or `relative: Zoned`; clock-only conversions need neither.
`Signed_Duration` represents exact elapsed seconds and nanoseconds. Timestamp span
arithmetic rejects every calendar unit, including days.

## Representation and errors

Civil fields use compact widths; calculations widen to `s64` before narrowing. Span
calendar fields are `s32` and clock fields are `s64`; even the latter cannot safely
be multiplied into one nanosecond counter. Operations check range before narrowing,
using quotient/remainder decomposition for signed seconds and nanoseconds. Exact
clock sums use Basic `S128` intermediates before checked narrowing. `span_check`
excludes each field's signed minimum so negation is total. Without a relative value,
days balance as 24 hours; months, years and weeks require a relative value.
Timestamp span addition still rejects days.

Timestamp and duration nanoseconds have magnitude below one billion and the seconds'
sign when seconds are nonzero. With zero seconds either remainder sign is valid:
Unix -0.001 seconds is `{seconds = 0, nanoseconds = -1000000}`. Timestamps cover
-9999-01-02T01:59:59Z through 9999-12-30T22:00:00Z (inclusive), so every valid
offset maps into the civil range; conversions from civil values outside that instant
range return `OUT_OF_RANGE`.

Public fields are transparent: constructors and parsers validate, other procedures
assume valid values, and invalid literals have unspecified results but never permit
out-of-bounds access, crashes or unbounded work.

Fallible procedures return a value and `Time_Error`; `.NONE` means success. An error
has a static message and a zero-based byte offset for text/binary parsing, otherwise
-1. Errors never need freeing. Do not consume failed results. `compare` returns
-1/0/1; `equal` and `hash` use the same semantic fields. Zoned comparison and
hashing depend only on the instant. Span ordering requires `span_compare` and an
appropriate relative value. Zone ordering compares kind and rule content, never
pointer addresses.

## Ownership

Civil, instant, duration and span calculations allocate nothing. Formatting appends
to a caller's `String_Builder`. Zone rules copy their input and share immutable
storage with an atomic reference count. Explicit ownership is necessary because
ordinary Jai assignment has no retain/release hook.

Successful zone loaders return one owned reference. `time_zone_retain(zone)`
acquires another; `time_zone_free(*zone)` releases an owner and resets it to UTC.
Ordinary assignment borrows a handle. `Zoned`, transition cursors and abbreviation
slices borrow their zone: an owner must outlive them. UTC and fixed zones own no
storage. The synchronized named-zone cache retains up to 64 zones; eviction releases
its reference. Opaque handles are checked against a registry before use so invalid
public literals never dereference arbitrary pointers; recently validated handles are
found without taking the registry lock, and releasing the last owner clears them first.
A cached name is found by comparing the stored path with the requested directory and
name, so a hit allocates nothing and reads only `TZDIR`.

ISO parsing of an owning `Time_Zone` or `Zoned` must return one owned zone
reference, released through the zone field. An unnamed ruleset has no lossless ISO
zone identifier; formatting it must return `UNSUPPORTED` rather than replace its
rules with a fixed offset.

## Text and interop

`parse(Type, text)` reads ISO dates, clock readings, date-times, RFC 3339
timestamps, offsets, RFC 9557 zoned values and ISO duration spans. Fractions have at
most nine digits; lowercase `t`/`z` and space date-time separators are accepted.
`format(*builder, value)` emits canonical text, `T`, UTC `Z` for timestamps, and
trimmed fractional zeros. Signed durations use seconds-only ISO duration text; zones
use bracketed names or fixed offsets. Zoned input rejects a conflicting numeric
offset and zone, and also accepts a bare zone annotation.

Pattern formatting/parsing uses English names and `%Y %m %d %e %H %I %M %S %f %.f %p
%j %a %A %b %B %z %:z %Z %Q %V %G %u %s %%`, with `-` and `_` padding flags. `%Q` is
the IANA name; `%Z` abbreviations need an explicit zone unless UTC/GMT. `%s` is Unix
seconds; `%f` has nine digits and `%.f` is an optional, trimmed fraction including
its dot. Missing required fields and conflicting fields are errors. No locale state
is consulted.

Json uses the `time_json_encode`/`time_json_decode` field-handler pair, aliased
inside a containing struct with `@json(with=time_json)`, following [Json's custom
field handlers](json.md). Rule zones and zoned values with rules encode normally,
but their custom Json decoder rejects them: Json has no cleanup hook for an owning
handler when a later field fails. Decode those ISO strings with `parse` and release
the returned zone explicitly instead. Civil values, instants, spans and fixed/UTC
zones support both directions. Basic interop uses qualified `Basic.Apollo_Time` and
`Basic.Calendar_Time`. Apollo conversions reject subnanosecond precision, and
conversion to Calendar_Time rejects submillisecond precision rather than silently
losing it. Calendar interop translates the zero-based month/day fields; a civil
value carries no implicit local zone.

## Compared with the community libraries

[rluba/jai-date](https://github.com/rluba/jai-date) is one small file of civil dates: a `Date` of year, month and day, with add, compare, ISO and German text, and a leap-year test. It needs the `uniform` regex module for parsing.

| Feature | Time | jai-date |
| --- | --- | --- |
| Value types | `Date`, `Time_Of_Day`, `Date_Time`, `Timestamp`, `Zoned`, `Span`, `Signed_Duration` | `Date` only |
| Instants | `Timestamp`, nanoseconds, -9999 to 9999 | none; `date_from_unix_timestamp` keeps only the day |
| Time zones | UTC, fixed offsets, TZif files, POSIX rules, never implicit (`time_zone_system` opts in to the machine zone) | UTC or the machine's zone, through libc `localtime_r` (POSIX only) |
| DST gap and fold | `COMPATIBLE`, `EARLIER`, `LATER`, `REJECT` | no, no zoned times |
| Add to a date | days, weeks, months, years, with checked range | days, months, years; month end clamps; `int` amount wraps silently into `s16` year |
| Differences | `until`/`since` with largest and smallest unit | no, compare only (`compare` returns a subtraction, not -1/0/1) |
| Spans | ten units, calendar-aware, relative conversions | none |
| Rounding | `Rounding` modes and increments | none |
| Parsing | ISO dates, times, RFC 3339, RFC 9557, ISO durations, patterns | `YYYY-MM-DD` prefix by regex, `DD.MM.YYYY`; trailing text is ignored |
| Formatting | canonical ISO, 20 pattern directives, English names | ISO and German `DD.MM.YYYY` |
| Weeks | ISO week number and year, weekday, nth weekday of month | none |
| Leap years and range | `is_leap_year`, `days_in_month`; years -9999 to 9999, checked | `is_leap_year`; `s16` year, unchecked |
| Errors | `Time_Error` with a message and byte offset | `bool`; invalid dates assert in some paths |
| Json | `time_json` field handler | none |
| Basic interop | `Apollo_Time`, `Calendar_Time` | `to_apollo_time`, current date from `Calendar_Time` |
| Allocation | calculations allocate nothing | parsing builds a compiled regex once; `string_from_date` allocates |
| End of month or year | no helper; use `days_in_month` | `end_of` |

Time is a full time library; jai-date is a date adder. The differences that matter are explicit zones with DST handling, exact instants, spans, and parse errors that say where. jai-date's only extra is `end_of`, which Time doesn't offer as one call, and German dates, which a pattern covers (`%d.%m.%Y`).

## Configuration and dependencies

`Time_Options` bounds text bytes, TZif bytes, transitions, local-time types and
abbreviation bytes before allocation. Every text/TZif parser takes it. Negative
limits are errors. No unbounded allocation is permitted from file counts.

Named-zone lookup uses an explicit `directory` argument when supplied, otherwise
`TZDIR`, otherwise `/usr/share/zoneinfo` on Unix. Names must not escape that
directory. Windows requires an explicit directory or `TZDIR`. System-zone lookup
uses `TZ` (POSIX string, `:name` or IANA name), then `/etc/localtime` on Unix;
Windows without `TZ` is unsupported. TZif versions 1–4 and their POSIX footers are
in scope; leap seconds are rejected.

The module imports `Basic`, `String`, `File`, `Hash_Table`, `Atomics`,
`Extensions/Environment` and `Extensions/Json`. Exact time reuses
`Basic.current_time_consensus()` for `timestamp_now`, filesystem and environment
modules for zone loading, and existing synchronization/cache conventions. No
embedded tz database or external service is required.

## How to change it

`module.jai` only loads the module's files; public contracts and examples sit
beside their code. Each file starts with its public procedures and keeps private
helpers after `#scope_module`.

| File | Holds |
| --- | --- |
| `types.jai` | value, error and option types |
| `numeric.jai` | checked integer and 128-bit arithmetic |
| `civil.jai` | dates, clock readings, civil arithmetic |
| `instants.jai` | timestamps, durations, offsets |
| `spans.jai` | `until`/`since`, rounding, balancing, totals |
| `zone_load.jai` | TZif and POSIX loading, the zone cache, the handle registry |
| `zoned.jai` | offset lookup, civil-to-instant resolution, transitions |
| `ordering.jai` | `compare`, `equal`, `hash` |
| `text.jai` | ISO `parse` and `format` |
| `pattern.jai` | pattern `format` and `parse_format` |
| `interop.jai` | `Basic` time values and Json |

Calendar differences start from an estimate taken from the calendar fields and widen
and bisect from it, so a difference costs a few additions rather than a search over
each unit. Civil-to-instant resolution only considers offsets in effect within 26 hours of
the reading. Before implementing a phase, write its regression tests under
this module. Add bounded TZif conformance data
through the portable data-fixture workflow, not host zone files. Validate native and
interpreter behavior, allocation balance and malformed-input offsets.

```sh
jacket test --path stdlib/Extensions/Time
jacket test --path stdlib/Extensions/Time --interp
jailint -D warnings stdlib/Extensions/Time/module.jai
jaifmt --check stdlib/Extensions/Time/*.jai
jacket doc stdlib/Extensions/Time --check
jacket bench --path stdlib/Extensions/Time
```

Conformance data is generated by `PYTHONPATH=tools python3
tools/portable/build_portable.py data`. `tools/time_conformance/fetch.py` pins a
tzdata wheel, unpacks its TZif files and writes offsets, abbreviations and gap/fold
vectors using Python `zoneinfo`. `JACKET_OUTPUT_DIR` relocates these fixtures along
with other repository output.

Benchmarks live in `benches/bench.jai`. They measure civil add/until, zone conversion, RFC
3339 text, cached lookup, and offset lookup with table and POSIX rules. Python's `datetime` uses microseconds, so its
parse/format comparison does less precision work.