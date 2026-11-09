# Time

`#import "Extensions/Time"` declares civil dates, exact instants, explicit time
zones and calendar-aware spans. This is the API declaration phase: bodies are
stubs, fallible calls return `UNSUPPORTED`, and no time calculations are
implemented yet. The contracts below describe the intended implementation.

## Concepts

`Timestamp` identifies an instant independently of a zone. `Date`, `Time_Of_Day`
and `Date_Time` describe the proleptic Gregorian calendar and wall clock; they
do not identify instants. ISO weeks start on Monday and week one contains
January 4. Years include zero and run from -9999 through 9999.

`Time_Zone` maps an instant to an offset east of UTC. A zone is UTC, a fixed
offset, or immutable TZif/POSIX rules. Mapping civil time back to an instant can
encounter a gap or fold. `COMPATIBLE` resolves a gap later and a fold earlier;
`EARLIER`, `LATER` and `REJECT` make the other choices explicit. No conversion
implicitly uses the machine's zone.

`Zoned` combines an instant with its zone and resolved offset. Adding calendar
units first changes its civil reading and resolves it again. Clock units then
advance the exact timeline. One day therefore preserves the wall-clock time
across a DST change; 24 hours advances exactly 24 hours.

`Span` keeps ten units separately, with one shared sign. Years, months and weeks
need a relative date or zoned value to acquire an elapsed length. Zoned days
also vary in length. Span conversions involving calendar units use overloads taking `relative: Date` or `relative: Zoned`; clock-only conversions need neither.
`Signed_Duration` represents exact elapsed seconds and nanoseconds. Timestamp
span arithmetic rejects every calendar unit, including days.

## Representation and errors

Civil fields retain the brief's compact widths; calculations must widen to
checked `s64` intermediates before narrowing. Span calendar fields are `s32` and
clock fields are `s64`; even the latter cannot safely be multiplied into one
nanosecond counter. The implementation must check operations before arithmetic,
using quotient/remainder decomposition where a scaled intermediate would
overflow. Field widths alone do not make arbitrary arithmetic safe.

Timestamp and duration nanoseconds have magnitude below one billion and the
seconds' sign when seconds are nonzero. With zero seconds either remainder sign
is valid: Unix -0.001 seconds is `{seconds = 0, nanoseconds = -1000000}`.
Timestamps cover -9999-01-02T01:59:59Z through 9999-12-30T22:00:00Z
(inclusive), so every valid offset maps into the civil range; conversions from
civil values outside that instant range return `OUT_OF_RANGE`.

Public fields are transparent: constructors and parsers validate, other procedures
assume valid values, and invalid literals have unspecified results but never permit
out-of-bounds access, crashes or unbounded work.

Fallible procedures return a value and `Time_Error`; `.NONE` means success.
An error has a static message and a zero-based byte offset for text/binary
parsing, otherwise -1. Errors never need freeing. Do not consume failed results.
`compare` returns -1/0/1; `equal` and `hash` use the same semantic fields. Zoned
comparison and hashing depend only on the instant. Span ordering requires
`span_compare` and an appropriate relative value. Zone ordering compares kind
and rule content, never pointer addresses.

## Ownership

Civil, instant, duration and span calculations allocate nothing. Formatting
appends to a caller's `String_Builder`. Zone rules copy their input and share
immutable storage with an atomic reference count. Explicit ownership is necessary
because ordinary Jai assignment has no retain/release hook.

Successful zone loaders return one owned reference. `time_zone_retain(zone)`
acquires another; `time_zone_free(*zone)` releases an owner and resets it to UTC.
Ordinary assignment borrows a handle. `Zoned`, transition cursors and abbreviation
slices borrow their zone: an owner must outlive them. UTC and fixed zones own no
storage. The named-zone cache holds its own reference and must synchronize lookup
and insertion; caller release must not invalidate cached rules.

ISO parsing of an owning `Time_Zone` or `Zoned` must return one owned zone
reference, released through the zone field. An unnamed ruleset has no lossless
ISO zone identifier; formatting it must return `UNSUPPORTED` rather than replace
its rules with a fixed offset.

## Text and interop

`parse(Type, text)` reads ISO dates, clock readings, date-times, RFC 3339
timestamps, offsets, RFC 9557 zoned values and ISO duration spans. Fractions have
at most nine digits; lowercase `t`/`z` and space date-time separators are accepted.
`format(*builder, value)` emits canonical text, `T`, UTC `Z` for timestamps, and
trimmed fractional zeros. Signed durations use seconds-only ISO duration text;
zones use bracketed names or fixed offsets. Zoned input rejects a conflicting
numeric offset and zone, and also accepts a bare zone annotation.

Pattern formatting/parsing uses English names and `%Y %m %d %e %H %I %M %S %f
%.f %p %j %a %A %b %B %z %:z %Z %Q %V %G %u %s %%`, with `-` and `_` padding
flags. `%Q` is the IANA name. Missing required fields and conflicting fields are
errors. No locale state is consulted.

Json uses the `time_json_encode`/`time_json_decode` field-handler pair, aliased
inside a containing struct with `@json(with=time_json)`, following
[Json's custom field handlers](json.md). Basic interop uses qualified
`Basic.Apollo_Time` and `Basic.Calendar_Time`. Apollo conversions reject
subnanosecond precision, and conversion to Calendar_Time rejects submillisecond
precision rather than silently losing it. Calendar interop translates the
zero-based month/day fields; a civil value carries no implicit local zone.

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

The declaration imports `Basic`, `String` and `Extensions/Json`. Implementation
must reuse `Basic.current_time_consensus()` for `timestamp_now`, filesystem and
environment modules for zone loading, and existing synchronization/cache
conventions. No embedded tz database or external service is required.

## How to change it

Keep public contracts and examples in `module.jai`. Before implementing a phase,
write its regression tests under this module. Add bounded TZif conformance data
through the portable data-fixture workflow, not host zone files. Validate native
and interpreter behavior, allocation balance and malformed-input offsets.

```sh
jacket test --path stdlib/Extensions/Time
jacket test --path stdlib/Extensions/Time --interp
jailint -D warnings stdlib/Extensions/Time/module.jai
jaifmt stdlib/Extensions/Time/module.jai
jacket doc stdlib/Extensions/Time --check
```
