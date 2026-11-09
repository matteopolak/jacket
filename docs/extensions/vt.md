# VT byte parser

`Extensions/Vt` parses a terminal byte stream without performing I/O or maintaining a screen.
`examples/term/vt` consumes its actions and owns the terminal grid, modes, attributes and replies.

```jai
#import "Basic";
#import "Extensions/Vt";

on_action :: (data: *void, action: Vt_Action) {
    if action.kind == .PRINT print("codepoint: %\n", action.code);
}

main :: () {
    parser: Vt_Parser;
    error := vt_feed(*parser, "hello\x1b[31m", on_action);
    assert(error.kind == .NONE);
    vt_finish(*parser, on_action);
}
```

## How it works

`vt_feed` accepts arbitrary chunks, including splits inside UTF-8 and escape sequences. The
parser implements ground, escape, escape intermediate, CSI entry/param/intermediate/ignore,
DCS entry/param/intermediate/passthrough/ignore, OSC string and SOS/PM/APC string states.
SOS/PM/APC payloads are discarded. CAN and SUB cancel a sequence and emit `EXECUTE`.
ESC interrupts a sequence and starts another escape; leaving DCS emits `DCS_UNHOOK`.

Callbacks receive `PRINT`, `EXECUTE`, `CSI_DISPATCH`, `ESC_DISPATCH`, `OSC_DISPATCH`,
`DCS_HOOK`, `DCS_PUT` or `DCS_UNHOOK`. `code` contains a Unicode codepoint for print, a byte
for execute/put, or the final byte for a dispatch/hook. CSI and DCS carry a flat `params` view:
a parameter with `subparam = true` follows a colon, otherwise it begins a semicolon group.
Empty numeric fields are zero; an absent parameter list is represented by one zero.
For `CSI ?38:2::255;1m`, the values are `38, 2, 0, 255, 1`, with flags
`false, true, true, true, false`; `private_marker` is `?`. Intermediates are separate bytes.

OSC fields split on semicolons, preserving empty fields; an empty OSC has one empty field.
BEL terminates OSC. ESC dispatches the accumulated OSC and enters escape state, so a following
backslash produces an `ESC_DISPATCH` for ST. C0 bytes other than terminators/cancellation
are discarded inside OSC and DCS headers; DCS passthrough emits them as `DCS_PUT`.
DEL is ignored except inside OSC. C0 bytes execute in ground, escape and CSI states.

Ground state decodes UTF-8 across calls. A completed overlong sequence, surrogate or out-of-range
codepoint produces one U+FFFD. An interrupted partial sequence produces U+FFFD before processing
the interrupting byte; stray invalid bytes each produce U+FFFD. `vt_finish` replaces an incomplete
UTF-8 tail, unhooks an active DCS and discards unfinished headers/OSC. It resets the parser while
retaining options and the absolute byte offset. It does not dispatch an unterminated OSC.

Action slices borrow parser storage and expire when the callback returns. Copy anything retained.
Do not reenter or mutate the parser from its callback. Set options before a stream begins and keep
them unchanged until `vt_finish`. A parser owns only inline storage: there is nothing to free and
no output queue that can grow. Callbacks are responsible for bounding their own retained output.

## Configuration and errors

`Vt_Options` defaults to 32 numeric fields (including subparameters) or OSC fields, saturated numeric
values at 65535, two intermediates, 4096 OSC bytes including separators, and `c1 = false`.
Hard capacities are 64 fields, eight intermediates and 4096 OSC bytes. Numeric saturation can be
set to any `u32`. Zero OSC bytes/intermediates are allowed; at least one field is required.

With `c1 = true`, raw 8-bit C1 bytes introduce CSI/DCS/OSC/SOS/PM/APC, terminate strings with ST,
or execute other C1 controls. In ground state a pending UTF-8 continuation takes precedence.
With C1 disabled, raw high bytes in ground are decoded as UTF-8; encoded C1 codepoints are printed.

`vt_feed` consumes the full input and returns the first error in that call, with a static message,
kind and zero-based absolute byte offset. `LIMIT` discards the entire offending sequence and
resynchronizes at its terminator or cancellation; it never dispatches a truncated command.
Numeric saturation is intentional and does not report an error. Malformed transitions follow the
state machine's ignore rules. `OPTIONS` consumes no input and also covers a null callback.

The terminal example keeps its numeric saturation at 1000000 and interprets only its existing
semicolon SGR and supported commands. Its grid does not interpret DCS or colon SGR actions.

## Validation and changes

The state machine and public types are in `stdlib/Extensions/Vt/module.jai`. Change `step` for
global controls, strings or ground UTF-8, and `header_step` for escape/CSI/DCS headers. Add a
regression before changing transitions and check whole feeds against every chunk boundary.
Increasing a hard capacity requires updating the inline arrays and option validation together.

The independent data port in `tests/conformance.jai` covers all 34 named parser tests from
[vte's pinned test source](https://github.com/alacritty/vte/blob/89c9c22f5b8fcc3feba0a9d9f50a020801e74acc/src/lib.rs),
under its MIT license (notice in the test file). Expected actions adapt private markers, overflow
rejection, fixed-buffer options and opt-in C1 to this API. No ported case is skipped. Tests also
exercise cancellation, malformed headers, all states, numeric saturation, invalid options,
random byte chunking and zero allocations with a counting allocator.

```sh
jacket test --path stdlib/Extensions/Vt
jacket test --path examples/term
PYTHONPATH=tools python3 tools/vt_conformance/fetch.py   # only to re-derive the ported cases
```

The fetcher uses `tools/toolkit/download.py` and verifies the source SHA-256.
The compiled tests use the committed data port and need no network or downloaded source.

## Benchmark and dependencies

The runtime depends only on `Basic`; it has no OS, terminal, allocator or external library requirement.
The benchmark generates text, SGR and cursor commands and feeds a counting callback without a screen. Setup is outside the measured loop. The separate Python reference accepts a recorded stream and requires `pyte`; Python may batch text callbacks.

```sh
jacket bench --path stdlib/Extensions/Vt
```

Unrestricted random-byte differential testing against pyte is not included: its stream exposes
emulator commands, lacks the DCS/subparameter action interface, and treats malformed CSI,
C1 and cancelled sequences differently. Random-byte tests here compare whole and bytewise feeds.
