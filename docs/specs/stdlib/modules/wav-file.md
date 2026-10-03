# Wav_File


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `WAVE_FORMAT_PCM :: 0x0001`
- `WAVE_FORMAT_DVI_ADPCM :: 0x0011`
- `Waveformatex :: struct`
- `Wave_Extra_Info :: struct`
- `get_wav_header :: (file_data: string) -> (format: Waveformatex, samples: string, success: bool = false, extra : Wave_Extra_Info)`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Wave_Extra_Info`

```jai
Wave_Extra_Info :: struct {
    wSamplesPerBlock : s32;
    wSamplesAccordingToFactChunk: s32;
}
```
### `Waveformatex`

```jai
Waveformatex :: struct {
    wFormatTag       : s16;
    nChannels        : s16;
    nSamplesPerSec   : s32;
    nAvgBytesPerSec  : s32;
    nBlockAlign      : s16;
    wBitsPerSample   : s16;
    cbSize           : s16;
}
```
## Behavioral contract

`get_wav_header` parses a RIFF/WAVE byte view, returning format metadata and a non-owning view of the first data chunk. It accepts PCM and DVI/IMA ADPCM format metadata and captures samples-per-block/fact count when present. Unknown chunks are skipped. The parser tolerates a RIFF-declared length that exceeds available bytes by limiting reads to available input, and clamps oversized chunk lengths; malformed/truncated data can still fail. It is not a general WAV decoder and does not decode samples. `Waveformatex` is packed without padding. Depends on byte-order and basic file/string helpers; input memory must remain alive for the returned sample view.
