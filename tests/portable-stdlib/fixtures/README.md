# Original portable-worker fixtures

`meow-vectors.bin` is non-executable test data generated on 2026-10-14 by compiling
the permitted official Molly Rocket C source with our own
`native/meow-c-check.cpp`. No supplied reference program or Jai implementation was
used. Reproduce/verify with `python3 -B tools/build_portable_meow_fixtures.py --verify-native`.
The wrapper preserves the unmodified C license and records its actual pre-fold
state through the C source's debug hook; streaming state is collected by its
normal finalization API.

Source SHA-256: `2328557212adc48ee3e8e8c9d25820a43b2fc0c48cf4296ab9b0b1ba52d376e2`.
Data SHA-256: `b4b8af0a8629e1cdc1a9853486158126e7e5756e72a062682e4eb354765ff639`.

Format: magic `MCV5`, little-endian u32 count, 128-byte expanded seed, followed by
317 records of 304 bytes: u64 input length, u32 offset, u32 mode, 16-byte one-shot
hash, 16-byte streaming hash, 128-byte one-shot pre-fold state, 128-byte streaming
pre-fold state. Inputs are an 8192-byte 4096-aligned array with byte i equal to
`(i * 73 + 19) & 255`; each source starts at `32 + offset`. Modes select default
seed, edited first seed byte, edited tail mask, edited shift adjustment, or the
provided expanded seed. The data pins the permitted calico algorithm, not any
unspecified Jai ancillary macro/diagnostic contract.
