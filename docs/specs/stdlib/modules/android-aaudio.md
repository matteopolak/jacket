# Android/AAudio

## Public declaration inventory

This namespace exports the declarations listed here, including the type members below. ABI binding semantics are governed by the Android NDK or SDK API represented by each declaration; native handles, status codes, and ownership follow that API.

- `AAUDIO_UNSPECIFIED :: 0`
- `AAUDIO_SYSTEM_USAGE_OFFSET :: 1000`
- `AAUDIO_DIRECTION :: enum u32`
- `aaudio_direction_t :: s32`
- `AAUDIO_FORMAT :: enum s32`
- `AAUDIO :: enum s32`
- `AAUDIO_STREAM_STATE :: enum u32`
- `AAUDIO_SHARING_MODE :: enum u32`
- `AAUDIO_PERFORMANCE_MODE :: enum u32`
- `AAUDIO_1 :: enum u32`
- `aaudio_usage_t :: s32`
- `AAUDIO_CONTENT_TYPE :: enum u32`
- `AAUDIO_SPATIALIZATION_BEHAVIOR :: enum u32`
- `aaudio_spatialization_behavior_t :: s32`
- `AAUDIO_INPUT_PRESET :: enum u32`
- `aaudio_input_preset_t :: s32`
- `AAUDIO_ALLOW_CAPTURE_BY :: enum u32`
- `aaudio_allowed_capture_policy_t :: s32`
- `AAUDIO_SESSION_ID :: enum s32`
- `aaudio_session_id_t :: s32`
- `AAUDIO_CHANNEL :: enum s32`
- `AAudioStreamStruct :: struct`
- `AAudioStream :: AAudioStreamStruct`
- `AAudioStreamBuilderStruct :: struct`
- `AAudioStreamBuilder :: AAudioStreamBuilderStruct`
- `AAudio_convertResultToText :: (returnCode: AAUDIO) -> *u8 #foreign libaaudio`
- `AAudio_convertStreamStateToText :: (state: AAUDIO_STREAM_STATE) -> *u8 #foreign libaaudio`
- `AAudio_createStreamBuilder :: (builder: **AAudioStreamBuilder) -> AAUDIO #foreign libaaudio`
- `AAudioStreamBuilder_setDeviceId :: (builder: *AAudioStreamBuilder, deviceId: s32) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setPackageName :: (builder: *AAudioStreamBuilder, packageName: *u8) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setAttributionTag :: (builder: *AAudioStreamBuilder, attributionTag: *u8) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setSampleRate :: (builder: *AAudioStreamBuilder, sampleRate: s32) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setChannelCount :: (builder: *AAudioStreamBuilder, channelCount: s32) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setSamplesPerFrame :: (builder: *AAudioStreamBuilder, samplesPerFrame: s32) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setFormat :: (builder: *AAudioStreamBuilder, format: AAUDIO_FORMAT) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setSharingMode :: (builder: *AAudioStreamBuilder, sharingMode: AAUDIO_SHARING_MODE) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setDirection :: (builder: *AAudioStreamBuilder, direction: aaudio_direction_t) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setBufferCapacityInFrames :: (builder: *AAudioStreamBuilder, numFrames: s32) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setPerformanceMode :: (builder: *AAudioStreamBuilder, mode: AAUDIO_PERFORMANCE_MODE) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setUsage :: (builder: *AAudioStreamBuilder, usage: aaudio_usage_t) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setContentType :: (builder: *AAudioStreamBuilder, contentType: AAUDIO_CONTENT_TYPE) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setSpatializationBehavior :: (builder: *AAudioStreamBuilder, spatializationBehavior: aaudio_spatialization_behavior_t) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setIsContentSpatialized :: (builder: *AAudioStreamBuilder, isSpatialized: bool) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setInputPreset :: (builder: *AAudioStreamBuilder, inputPreset: aaudio_input_preset_t) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setAllowedCapturePolicy :: (builder: *AAudioStreamBuilder, capturePolicy: aaudio_allowed_capture_policy_t) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setSessionId :: (builder: *AAudioStreamBuilder, sessionId: aaudio_session_id_t) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setPrivacySensitive :: (builder: *AAudioStreamBuilder, privacySensitive: bool) -> void #foreign libaaudio`
- `AAUDIO_CALLBACK_RESULT :: enum u32`
- `AAudioStream_dataCallback :: #type (stream: *AAudioStream, userData: *void, audioData: *void, numFrames: s32) -> AAUDIO_CALLBACK_RESULT #c_call`
- `AAudioStreamBuilder_setDataCallback :: (builder: *AAudioStreamBuilder, callback: AAudioStream_dataCallback, userData: *void) -> void #foreign libaaudio`
- `AAudioStreamBuilder_setFramesPerDataCallback :: (builder: *AAudioStreamBuilder, numFrames: s32) -> void #foreign libaaudio`
- `AAudioStream_errorCallback :: #type (stream: *AAudioStream, userData: *void, error: AAUDIO) -> void #c_call`
- `AAudioStreamBuilder_setErrorCallback :: (builder: *AAudioStreamBuilder, callback: AAudioStream_errorCallback, userData: *void) -> void #foreign libaaudio`
- `AAudioStreamBuilder_openStream :: (builder: *AAudioStreamBuilder, stream: **AAudioStream) -> AAUDIO #foreign libaaudio`
- `AAudioStreamBuilder_delete :: (builder: *AAudioStreamBuilder) -> AAUDIO #foreign libaaudio`
- `AAudioStreamBuilder_setChannelMask :: (builder: *AAudioStreamBuilder, channelMask: AAUDIO_CHANNEL) -> void #foreign libaaudio`
- `AAudioStream_release :: (stream: *AAudioStream) -> AAUDIO #foreign libaaudio`
- `AAudioStream_close :: (stream: *AAudioStream) -> AAUDIO #foreign libaaudio`
- `AAudioStream_requestStart :: (stream: *AAudioStream) -> AAUDIO #foreign libaaudio`
- `AAudioStream_requestPause :: (stream: *AAudioStream) -> AAUDIO #foreign libaaudio`
- `AAudioStream_requestFlush :: (stream: *AAudioStream) -> AAUDIO #foreign libaaudio`
- `AAudioStream_requestStop :: (stream: *AAudioStream) -> AAUDIO #foreign libaaudio`
- `AAudioStream_getState :: (stream: *AAudioStream) -> AAUDIO_STREAM_STATE #foreign libaaudio`
- `AAudioStream_waitForStateChange :: (stream: *AAudioStream, inputState: AAUDIO_STREAM_STATE, nextState: *AAUDIO_STREAM_STATE, timeoutNanoseconds: s64) -> AAUDIO #foreign libaaudio`
- `AAudioStream_read :: (stream: *AAudioStream, buffer: *void, numFrames: s32, timeoutNanoseconds: s64) -> s32 #foreign libaaudio`
- `AAudioStream_write :: (stream: *AAudioStream, buffer: *void, numFrames: s32, timeoutNanoseconds: s64) -> s32 #foreign libaaudio`
- `AAudioStream_setBufferSizeInFrames :: (stream: *AAudioStream, numFrames: s32) -> s32 #foreign libaaudio`
- `AAudioStream_getBufferSizeInFrames :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getFramesPerBurst :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getBufferCapacityInFrames :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getFramesPerDataCallback :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getXRunCount :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getSampleRate :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getChannelCount :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getSamplesPerFrame :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getDeviceId :: (stream: *AAudioStream) -> s32 #foreign libaaudio`
- `AAudioStream_getFormat :: (stream: *AAudioStream) -> AAUDIO_FORMAT #foreign libaaudio`
- `AAudioStream_getSharingMode :: (stream: *AAudioStream) -> AAUDIO_SHARING_MODE #foreign libaaudio`
- `AAudioStream_getPerformanceMode :: (stream: *AAudioStream) -> AAUDIO_PERFORMANCE_MODE #foreign libaaudio`
- `AAudioStream_getDirection :: (stream: *AAudioStream) -> aaudio_direction_t #foreign libaaudio`
- `AAudioStream_getFramesWritten :: (stream: *AAudioStream) -> s64 #foreign libaaudio`
- `AAudioStream_getFramesRead :: (stream: *AAudioStream) -> s64 #foreign libaaudio`
- `AAudioStream_getSessionId :: (stream: *AAudioStream) -> aaudio_session_id_t #foreign libaaudio`
- `AAudioStream_getTimestamp :: (stream: *AAudioStream, clockid: clockid_t, framePosition: *s64, timeNanoseconds: *s64) -> AAUDIO #foreign libaaudio`
- `AAudioStream_getUsage :: (stream: *AAudioStream) -> aaudio_usage_t #foreign libaaudio`
- `AAudioStream_getContentType :: (stream: *AAudioStream) -> AAUDIO_CONTENT_TYPE #foreign libaaudio`
- `AAudioStream_getSpatializationBehavior :: (stream: *AAudioStream) -> aaudio_spatialization_behavior_t #foreign libaaudio`
- `AAudioStream_isContentSpatialized :: (stream: *AAudioStream) -> bool #foreign libaaudio`
- `AAudioStream_getInputPreset :: (stream: *AAudioStream) -> aaudio_input_preset_t #foreign libaaudio`
- `AAudioStream_getAllowedCapturePolicy :: (stream: *AAudioStream) -> aaudio_allowed_capture_policy_t #foreign libaaudio`
- `AAudioStream_isPrivacySensitive :: (stream: *AAudioStream) -> bool #foreign libaaudio`
- `AAudioStream_getChannelMask :: (stream: *AAudioStream) -> AAUDIO_CHANNEL #foreign libaaudio`

## Public type members

#### `AAUDIO_DIRECTION`

```jai
AAUDIO_DIRECTION :: enum u32 {
    OUTPUT :: 0;
    INPUT  :: 1;
}
```
#### `AAUDIO_FORMAT`

```jai
AAUDIO_FORMAT :: enum s32 {
    INVALID        :: -1;
    UNSPECIFIED    :: 0;
    PCM_I16        :: 1;
    PCM_FLOAT      :: 2;
    PCM_I24_PACKED :: 3;
    PCM_I32        :: 4;
}
```
#### `AAUDIO`

```jai
AAUDIO :: enum s32 {
    OK                     :: 0;
    ERROR_BASE             :: -900;
    ERROR_DISCONNECTED     :: -899;
    ERROR_ILLEGAL_ARGUMENT :: -898;
    ERROR_INTERNAL         :: -896;
    ERROR_INVALID_STATE    :: -895;
    ERROR_INVALID_HANDLE   :: -892;
    ERROR_UNIMPLEMENTED    :: -890;
    ERROR_UNAVAILABLE      :: -889;
    ERROR_NO_FREE_HANDLES  :: -888;
    ERROR_NO_MEMORY        :: -887;
    ERROR_NULL             :: -886;
    ERROR_TIMEOUT          :: -885;
    ERROR_WOULD_BLOCK      :: -884;
    ERROR_INVALID_FORMAT   :: -883;
    ERROR_OUT_OF_RANGE     :: -882;
    ERROR_NO_SERVICE       :: -881;
    ERROR_INVALID_RATE     :: -880;
}
```
#### `AAUDIO_STREAM_STATE`

```jai
AAUDIO_STREAM_STATE :: enum u32 {
    UNINITIALIZED :: 0;
    UNKNOWN       :: 1;
    OPEN          :: 2;
    STARTING      :: 3;
    STARTED       :: 4;
    PAUSING       :: 5;
    PAUSED        :: 6;
    FLUSHING      :: 7;
    FLUSHED       :: 8;
    STOPPING      :: 9;
    STOPPED       :: 10;
    CLOSING       :: 11;
    CLOSED        :: 12;
    DISCONNECTED  :: 13;
}
```
#### `AAUDIO_SHARING_MODE`

```jai
AAUDIO_SHARING_MODE :: enum u32 {
    EXCLUSIVE :: 0;
    SHARED    :: 1;
}
```
#### `AAUDIO_PERFORMANCE_MODE`

```jai
AAUDIO_PERFORMANCE_MODE :: enum u32 {
    NONE         :: 10;
    POWER_SAVING :: 11;
    LOW_LATENCY  :: 12;
}
```
#### `AAUDIO_1`

```jai
AAUDIO_1 :: enum u32 {
    USAGE_MEDIA                          :: 1;
    USAGE_VOICE_COMMUNICATION            :: 2;
    USAGE_VOICE_COMMUNICATION_SIGNALLING :: 3;
    USAGE_ALARM                          :: 4;
    USAGE_NOTIFICATION                   :: 5;
    USAGE_NOTIFICATION_RINGTONE          :: 6;
    USAGE_NOTIFICATION_EVENT             :: 10;
    USAGE_ASSISTANCE_ACCESSIBILITY       :: 11;
    USAGE_ASSISTANCE_NAVIGATION_GUIDANCE :: 12;
    USAGE_ASSISTANCE_SONIFICATION        :: 13;
    USAGE_GAME                           :: 14;
    USAGE_ASSISTANT                      :: 16;
    SYSTEM_USAGE_EMERGENCY               :: 1000;
    SYSTEM_USAGE_SAFETY                  :: 1001;
    SYSTEM_USAGE_VEHICLE_STATUS          :: 1002;
    SYSTEM_USAGE_ANNOUNCEMENT            :: 1003;
}
```
#### `AAUDIO_CONTENT_TYPE`

```jai
AAUDIO_CONTENT_TYPE :: enum u32 {
    SPEECH       :: 1;
    MUSIC        :: 2;
    MOVIE        :: 3;
    SONIFICATION :: 4;
}
```
#### `AAUDIO_SPATIALIZATION_BEHAVIOR`

```jai
AAUDIO_SPATIALIZATION_BEHAVIOR :: enum u32 {
    AUTO  :: 1;
    NEVER :: 2;
}
```
#### `AAUDIO_INPUT_PRESET`

```jai
AAUDIO_INPUT_PRESET :: enum u32 {
    GENERIC             :: 1;
    CAMCORDER           :: 5;
    VOICE_RECOGNITION   :: 6;
    VOICE_COMMUNICATION :: 7;
    UNPROCESSED         :: 9;
    VOICE_PERFORMANCE   :: 10;
}
```
#### `AAUDIO_ALLOW_CAPTURE_BY`

```jai
AAUDIO_ALLOW_CAPTURE_BY :: enum u32 {
    ALL    :: 1;
    SYSTEM :: 2;
    NONE   :: 3;
}
```
#### `AAUDIO_SESSION_ID`

```jai
AAUDIO_SESSION_ID :: enum s32 {
    NONE     :: -1;
    ALLOCATE :: 0;
}
```
#### `AAUDIO_CHANNEL`

```jai
AAUDIO_CHANNEL :: enum s32 {
    INVALID               :: -1;
    FRONT_LEFT            :: 1;
    FRONT_RIGHT           :: 2;
    FRONT_CENTER          :: 4;
    LOW_FREQUENCY         :: 8;
    BACK_LEFT             :: 16;
    BACK_RIGHT            :: 32;
    FRONT_LEFT_OF_CENTER  :: 64;
    FRONT_RIGHT_OF_CENTER :: 128;
    BACK_CENTER           :: 256;
    SIDE_LEFT             :: 512;
    SIDE_RIGHT            :: 1024;
    TOP_CENTER            :: 2048;
    TOP_FRONT_LEFT        :: 4096;
    TOP_FRONT_CENTER      :: 8192;
    TOP_FRONT_RIGHT       :: 16384;
    TOP_BACK_LEFT         :: 32768;
    TOP_BACK_CENTER       :: 65536;
    TOP_BACK_RIGHT        :: 131072;
    TOP_SIDE_LEFT         :: 262144;
    TOP_SIDE_RIGHT        :: 524288;
    BOTTOM_FRONT_LEFT     :: 1048576;
    BOTTOM_FRONT_CENTER   :: 2097152;
    BOTTOM_FRONT_RIGHT    :: 4194304;
    LOW_FREQUENCY_2       :: 8388608;
    FRONT_WIDE_LEFT       :: 16777216;
    FRONT_WIDE_RIGHT      :: 33554432;
    MONO                  :: 1;
    STEREO                :: 3;
    _2POINT1              :: 11;
    TRI                   :: 7;
    TRI_BACK              :: 259;
    _3POINT1              :: 15;
    _2POINT0POINT2        :: 786435;
    _2POINT1POINT2        :: 786443;
    _3POINT0POINT2        :: 786439;
    _3POINT1POINT2        :: 786447;
    QUAD                  :: 51;
    QUAD_SIDE             :: 1539;
    SURROUND              :: 263;
    PENTA                 :: 55;
    _5POINT1              :: 63;
    _5POINT1_SIDE         :: 1551;
    _6POINT1              :: 319;
    _7POINT1              :: 1599;
    _5POINT1POINT2        :: 786495;
    _5POINT1POINT4        :: 184383;
    _7POINT1POINT2        :: 788031;
    _7POINT1POINT4        :: 185919;
    _9POINT1POINT4        :: 50517567;
    _9POINT1POINT6        :: 51303999;
    FRONT_BACK            :: 260;
}
```
#### `AAudioStreamStruct`

Opaque handle marker; no public fields are declared.

```jai
AAudioStreamStruct :: struct {}
```
#### `AAudioStreamBuilderStruct`

Opaque handle marker; no public fields are declared.

```jai
AAudioStreamBuilderStruct :: struct {}
```
#### `AAUDIO_CALLBACK_RESULT`

```jai
AAUDIO_CALLBACK_RESULT :: enum u32 {
    CONTINUE :: 0;
    STOP     :: 1;
}
```