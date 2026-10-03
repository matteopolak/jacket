# Adpcm

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `ADPCM_CHANNELS_MAX :: 9`
- `Adpcm_Info :: struct`
- `ADPCM_State :: struct`
- `IMA_ADPCM_decode :: (info: Adpcm_Info, input_block: [] u8, dest_buffer: [] u8) -> s32`
- `step_table : [89] s32`
- `index_table : [16] s8`
- `IMA_ADPCM_nibble :: inline (state: *ADPCM_State, nybble: u8) -> s32`
- `Fill_IMA_ADPCM_block :: (dest: [] u8, dest_start_index: s32, encoded: [4] u8, channel: s32, numchannels: s32, state: *ADPCM_State) #no_alias`

## Public type members

#### `Adpcm_Info` (struct)

- `nChannels:        s32`
- `nBlockAlign:      s32`
- `wSamplesPerBlock: s32`

#### `ADPCM_State` (struct)

- `sample: s32`
- `index:  s8`

## Public array values

Both table declarations use the typed-constant form `name : Type : initializer`; the complete declarations and values appear below.

`step_table` and `index_table` use `: Type : initializer` typed-constant declarations (the Jai declaration uses the typed-constant `: Type : initializer` form), not mutable globals. Their array values are shown below. `IMA_ADPCM_nibble` reads these fixed tables; callers cannot change the decoder behavior by assigning to their elements.

```jai
step_table : [89] s32 : .[
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31,
    34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130,
    143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408,
    449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282,
    1411, 1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327,
    3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630,
    9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350,
    22385, 24623, 27086, 29794, 32767
];

index_table : [16] s8 : .[
    -1, -1, -1, -1, 2, 4, 6, 8,
    -1, -1, -1, -1, 2, 4, 6, 8
];
```

## Behavioral contract and constraints

`IMA_ADPCM_decode` decodes complete IMA ADPCM blocks into caller-provided little-endian signed 16-bit PCM bytes and returns the number of destination bytes written. Each channel's four-byte block header supplies its initial sample (little-endian) and step index; the reserved fourth header byte is ignored even when nonzero. The destination byte count must be divisible by `2 * nChannels`, and `nChannels` must not exceed `ADPCM_CHANNELS_MAX`; these conditions are asserted. The caller must also supply coherent positive `nBlockAlign`, `wSamplesPerBlock`, channel count, and sufficiently large source and destination buffers: exhaustive validation of malformed headers, inconsistent metadata, or truncated payload is not provided.

For each encoded data byte, `Fill_IMA_ADPCM_block` emits the low nibble's sample before the high nibble's sample. It writes eight samples for the selected channel, interleaved by channel into `dest` beginning at `dest_start_index`; it updates the passed channel state. `IMA_ADPCM_nibble` expects a nibble in `[0, 15]`; it mutates `sample` and `index`, clamps the index into `[0, 88]` before table lookup and the resulting sample into signed 16-bit range, then returns the updated sample. The post-update index may fall outside `[0, 88]` and is clamped on the next call. The decoder initializes each block's channel states from its headers. Its special short-tail loop condition is based on two remaining bytes per channel even though header reads consume four bytes per channel, so callers must not rely on this path; other incomplete/truncated block behavior is not a supported contract. This codec variant has known interoperability differences with some ADPCM WAV producers/consumers, so byte compatibility with a given toolchain is not guaranteed.
