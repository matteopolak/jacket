# CHIP-8

A CHIP-8 and SUPER-CHIP interpreter with a Simp/OpenGL window, keyboard input, and a generated 441 Hz square-wave beep through `Sound_Player`. The CPU is a separate, deterministic library: no files, windows, clock reads, or mutable global state. Only original, tiny assembly programs are included.

```sh
# From the repository root, with jaic/jacket on PATH and JAIC_STDLIB set:
export T=$TMPDIR/chip8   # any build directory
jaic examples/chip8/tools/asm/main.jai -exe "$T/chip8-asm"
"$T/chip8-asm" examples/chip8/programs/paddle.asm "$T/paddle.ch8"
jaic examples/chip8/main.jai -exe "$T/chip8"
"$T/chip8" "$T/paddle.ch8" --seed 42
(cd examples/chip8 && jacket test -j 2)
sh examples/chip8/tools/smoke.sh
```

The paddle demo uses Q/E to move left/right and tries to catch the falling ball. A miss draws F; R starts again. All ROMs are assembled into scratch storage, never committed as binaries.

## Controls and configuration

The conventional hexadecimal keypad maps to:

```text
CHIP-8       keyboard
1 2 3 C      1 2 3 4
4 5 6 D      Q W E R
7 8 9 E      A S D F
A 0 B F      Z X C V
```

P toggles pause, N executes one instruction while paused, R resets including the RNG, Escape quits. R is also CHIP-8 key D. Pause freezes CPU and timers and silences the beep; single step does not advance time. `FX0A` waits for a fresh key press followed by its release.

| Flag | Default | Meaning |
| --- | --- | --- |
| `--preset` | `modern` | `chip8`, `schip`, or `modern` |
| `--seed` | `1` | 32-bit PCG seed; identical ROM/input/time reproduces execution |
| `--cycles-per-frame` | `12` | Instruction budget per 60 Hz frame, 1..10000 |
| `--scale` | `6` | Initial window size is 128x64 times this integer |
| `--foreground`, `--background` | `A8FF60`, `101820` | Six hexadecimal RGB digits |
| `--no-audio` | off | Do not open an audio device |
| `--smoke` | off | Headless; print low/high decimal Meow hash lanes and exit |
| `--cycles` | `1200` | Headless cycle-slot budget, 1..100000000 |
| `--window-frames` | `0` | Close after N frames; 0 means interactive |

A resized window centers the display at the largest fitting integer scale (at least one); low resolution uses 64x32, high resolution 128x64. Missing audio devices leave the app silent. No external asset files are needed. `T` chooses smoke build storage, `JAIC` selects the compiler, and `SKIP_WINDOW=1` skips the optional window/audio smoke when native display access is unavailable. On Linux a missing `DISPLAY` skips it; macOS needs an Aqua session with AppKit access. The script also recognizes the explicit AppKit activation-policy failure from restricted sessions.

## Quirks and supported machine

| Quirk | `chip8` | `schip` | `modern` |
| --- | --- | --- | --- |
| Shifts read VY | yes | no | no |
| BNNN adds VX (X from NNN) | no | yes | no |
| FX55/FX65 increment I | yes | no | no |
| Draw waits until next 60 Hz tick | yes | no | no |
| Sprites clip at right/bottom | yes | yes | yes |
| OR/AND/XOR clear VF | yes | no | no |

`Quirks` can be set independently, including wrapping instead of clipping. These are explicit compatibility policies rather than claims of cycle-accurate hardware emulation. All profiles support SUPER-CHIP: 00Cn, 00FB/FC, 00FD, 00FE/FF, DXY0 16x16 sprites, FX30 large font, and eight RPL flags via FX75/85. Mode changes clear the display. Low-resolution scrolling uses logical pixels. Large glyphs are a locally generated doubling of the built-in 4x5 font. RPL flags persist through mode changes, but reset with the machine. Legacy 0NNN machine-code calls and XO-CHIP instructions are rejected. Memory is 4 KiB, ROM begins at 0x200, stack depth is 16, and I addition wraps to 12 bits. Bad opcodes and out-of-range accesses halt with an error, never index host memory unchecked.

## How it works and how to change it

- `core/module.jai`: `reset`, `set_key`, `step(cycles)`, `advance_time(nanoseconds)`, `frame_hash`. Each machine owns RAM, registers, framebuffer, RNG, and quirk settings. `step` returns actual instructions run; waits and halts stop it early. Timers use integer nanoseconds with carried fractional ticks and are independent of the CPU budget.
- `main.jai`: Args flags, ROM loading and headless runner. Headless mode schedules up to `cycles_per_frame` slots then advances one 60 Hz frame (16,666,667 ns); a wait consumes the remaining slots, and a halted ROM ends early. Change deterministic scheduling here, preserving the smoke hashes deliberately.
- `window.jai`: real-time timer advancement, a fixed 60 Hz CPU scheduler, integer pixel drawing, keypad mapping and generated audio. It clamps catch-up to 250 ms to avoid a long stall monopolizing the UI. Changing window controls does not affect CPU tests.
- `tools/asm/module.jai`: a two-pass assembler plus a lossless raw disassembler. Extend syntax here and add round-trip/error tests. Error strings and returned ROM/text are owned by the caller. Labels refer into the source string only during assembly.
- `programs/`: `opcodes.asm` checks arithmetic, calls, and branches and draws A on pass/F on failure; `quirks.asm` draws 4 or 1 according to shift policy; `super.asm` exercises high resolution, large font, scrolling, wide XOR sprites, RPL, and low mode; `paddle.asm` is playable.
- `tests/`: grouped per-opcode checks, all quirk presets, timer rates, display collision and boundaries, memory/stack failures, seed determinism, assembler round-trip, and Meow framebuffer hashes.

Assembler syntax is intentionally small. Hex operands need no prefix; labels end in `:`; `;` starts a comment. `op` emits one 16-bit big-endian opcode, `byte` emits data, and `jp`, `call`, `index` accept a label or hex address. Instructions must be aligned; forward labels are supported. No expressions, macros or third-party ROMs:

```text
start:
op 600A       ; V0 = A
op F029       ; I = small font glyph V0
op 6100
op 6200
op D125       ; draw glyph at V1,V2
jp start
```

## Dependencies and verification

The CLI uses `Extensions/Args`, `File` and `Basic`; the core uses PCG's caller-owned RNG and `Extensions/Meow_Portable.hash_bytes`. That portable scalar API produces the same Meow 0.5 lanes on ARM and x64 without needing AES-NI. The window uses `Simp`, `Window_Creation`, `Input`, `Math` and `Sound_Player`, and their platform OpenGL/audio libraries. No packages are installed by this app.

Smoke rebuilds both Jai executables, assembles all four ROMs, checks twelve committed framebuffer digests across the three presets, checks replay/seed differences while the paddle demo is active, rejects invalid inputs, and attempts twelve real window frames when display access is available. Frame hashes include all 8192 bytes of the 128-stride framebuffer, including zero inactive cells; they measure emulation output, not GPU rendering. Native window interaction, actual pixel appearance and audible output require a usable desktop/audio device and manual acceptance.
