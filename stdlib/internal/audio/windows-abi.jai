// Public DirectSound ABI declarations from the contract.

DSSPEAKER :: enum {
    HEADPHONE :: 1;
    MONO :: 2;
    QUAD :: 3;
    STEREO :: 4;
    SURROUND :: 5;
    _5POINT1 :: 6;
    _5POINT1_BACK :: _5POINT1;
    _5POINT1_SURROUND :: 9;
    _7POINT1 :: 7;
    _7POINT1_SURROUND :: 8;
    _7POINT1_WIDE :: _7POINT1;
}

DSBUFFERCAPS :: struct {}

DSBUFFERDESC :: struct {
    dwSize: s32;
    dwFlags: s32;
    dwBufferBytes: s32;
    dwReserved: s32;
    lpwfxFormat: *Waveformatex;
    guid3DAlgorithm: GUID;
}

WAVEFORMATEXTENSIBLE :: struct {
    Format: Waveformatex;
    wValidBitsPerSample: s16;
    dwChannelMask: s32;
    SubFormat: GUID;
}

Enumeration_Callback_Data :: struct {
    outer_context: #Context;
    devices: [..] Output_Device;
}

IDirectSound8 :: struct {
    #as unknown: IUnknown;
    #overlay (unknown) using vtable: *IDirectSound8Vtable;
}

IDirectSoundBuffer8 :: struct {
    #as unknown: IUnknown;
    #overlay (unknown) using vtable: *IDirectSoundBuffer8Vtable;
}

IDirectSound8Vtable :: struct {
    #as using base: IUnknown_VTable;
    CreateSoundBuffer: (dsound: *IDirectSound8, desc: *DSBUFFERDESC, buffer: **IDirectSoundBuffer8, outer: **void = null) -> s32 #c_call;
    GetCaps: *void;
    DuplicateSoundBuffer: *void;
    SetCooperativeLevel: (dsound: *IDirectSound8, hwnd: HWND, level: s32) -> s32 #c_call;
    Compact: *void;
    GetSpeakerConfig: (dsound: *IDirectSound8, config: *s32) -> s32 #c_call;
    SetSpeakerConfig: *void;
    Initialize: *void;
}

IDirectSoundBuffer8Vtable :: struct {
    #as using base: IUnknown_VTable;
    GetCaps: (buffer: *IDirectSoundBuffer8, caps: *DSBUFFERCAPS) #c_call;
    GetCurrentPosition: (buffer: *IDirectSoundBuffer8, pdwCurrentPlayCursor: *s32, pdwCurrentWriteCursor: *s32) -> s32 #c_call;
    GetFormat: *void;
    GetVolume: *void;
    GetPan: *void;
    GetFrequency: *void;
    GetStatus: (buffer: *IDirectSoundBuffer8, pdwStatus: *s32) -> s32 #c_call;
    Initialize: *void;
    Lock: (buffer: *IDirectSoundBuffer8, dwOffset: s32, dwBytes: s32, pvAudioPtr1: **void, dwAudioBytes1: *s32, pvAudioPtr2: **void, dwAudioBytes2: *s32, dwFlags: s32) -> s32 #c_call;
    Play: (buffer: *IDirectSoundBuffer8, dwReserved1: s32, dwPriority: s32, dwFlags: s32) -> s32 #c_call;
    SetCurrentPosition: *void;
    SetFormat: (buffer: *IDirectSoundBuffer8, format: *Waveformatex) -> s32 #c_call;
    SetVolume: *void;
    SetPan: *void;
    SetFrequency: *void;
    Stop: (buffer: *IDirectSoundBuffer8) -> s32 #c_call;
    Unlock: (buffer: *IDirectSoundBuffer8, pvAudioPtr1: *void, dwAudioBytes1: s32, pvAudioPtr2: *void, dwAudioBytes2: s32) -> s32 #c_call;
    Restore: (buffer: *IDirectSoundBuffer8) -> s32 #c_call;
    SetFX: *void;
    AcquireResources: *void;
    GetObjectInPath: *void;
}

DSBCAPS_CTRLVOLUME :: 0x80;
DSBCAPS_GETCURRENTPOSITION2 :: 0x10000;
DSBCAPS_GLOBALFOCUS :: 0x08000;
DSBCAPS_PRIMARYBUFFER :: 0x1;
DSBPLAY_LOOPING :: 0x01;
DSBSTATUS_BUFFERLOST :: 0x002;
DSBSTATUS_LOCHARDWARE :: 0x008;
DSBSTATUS_LOCSOFTWARE :: 0x010;
DSBSTATUS_LOOPING :: 0x004;
DSBSTATUS_PLAYING :: 0x001;
DSBSTATUS_TERMINATED :: 0x020;
DS_OK :: 0;
KSAUDIO_SPEAKER_5POINT1 :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT;
KSAUDIO_SPEAKER_5POINT1_SURROUND :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_SIDE_LEFT | SPEAKER_SIDE_RIGHT;
KSAUDIO_SPEAKER_7POINT1 :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT | SPEAKER_FRONT_LEFT_OF_CENTER | SPEAKER_FRONT_RIGHT_OF_CENTER;
KSAUDIO_SPEAKER_7POINT1_SURROUND :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT | SPEAKER_SIDE_LEFT | SPEAKER_SIDE_RIGHT;
KSAUDIO_SPEAKER_MONO :: SPEAKER_FRONT_CENTER;
KSAUDIO_SPEAKER_QUAD :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT;
KSAUDIO_SPEAKER_STEREO :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT;
KSAUDIO_SPEAKER_SURROUND :: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_BACK_CENTER;
PRIMARY_BUFFER_SIZE_IN_BYTES :: 0;
SPEAKER_BACK_CENTER :: 0x100;
SPEAKER_BACK_LEFT :: 0x10;
SPEAKER_BACK_RIGHT :: 0x20;
SPEAKER_FRONT_CENTER :: 0x4;
SPEAKER_FRONT_LEFT :: 0x1;
SPEAKER_FRONT_LEFT_OF_CENTER :: 0x40;
SPEAKER_FRONT_RIGHT :: 0x2;
SPEAKER_FRONT_RIGHT_OF_CENTER :: 0x80;
SPEAKER_LOW_FREQUENCY :: 0x8;
SPEAKER_SIDE_LEFT :: 0x200;
SPEAKER_SIDE_RIGHT :: 0x400;
SPEAKER_TOP_BACK_CENTER :: 0x10000;
SPEAKER_TOP_BACK_LEFT :: 0x8000;
SPEAKER_TOP_BACK_RIGHT :: 0x20000;
SPEAKER_TOP_CENTER :: 0x800;
SPEAKER_TOP_FRONT_CENTER :: 0x2000;
SPEAKER_TOP_FRONT_LEFT :: 0x1000;
SPEAKER_TOP_FRONT_RIGHT :: 0x4000;
WAVE_FORMAT_EXTENSIBLE :: 0xFFFE;

CONFIG :: (speaker_config: s32) -> DSSPEAKER {
    return cast(DSSPEAKER) (speaker_config & 0xff);
}

GEOMETRY :: (speaker_config: s32) -> u8 {
    return cast,truncate(u8) (speaker_config >> 16);
}

init_KSDATAFORMAT_SUBTYPE_PCM :: (dest: *GUID) {
    dest.* = .{ 1, 0, 0x10, .[0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71] };
}

init_wave_format :: (using w: *Waveformatex, nchannels: s16) {
    assert(nchannels > 0 && nchannels <= MAX_CHANNELS_PER_SOUND);
    wFormatTag = 1;
    nChannels = nchannels;
    nSamplesPerSec = OUTPUT_SAMPLING_RATE;
    wBitsPerSample = 16;
    nBlockAlign = nchannels * 2;
    nAvgBytesPerSec = nBlockAlign * nSamplesPerSec;
    cbSize = 0;
}

#scope_file
W :: #import "Windows";

GUID :: W.GUID;
HWND :: W.HWND;
IUnknown :: W.IUnknown;
IUnknown_VTable :: W.IUnknown_VTable;
Waveformatex :: Wav.Waveformatex;
