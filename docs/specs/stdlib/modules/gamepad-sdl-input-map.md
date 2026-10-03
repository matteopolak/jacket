# Gamepad/SDL_Input_Map

## Public declaration inventory

This namespace exposes SDL-style controller mapping types and parsing helpers.

- `SDL_PLATFORM_WINDOWS :: "Windows"`
- `SDL_PLATFORM_MACOS :: "Mac OS X"`
- `SDL_PLATFORM_LINUX :: "Linux"`
- `SDL_Input_Target :: enum`
- `SDL_Input_Type :: enum`
- `SDL_Input_Map :: struct`
- `SDL_Gamepad_Mapping :: struct`
- `deinit :: (map: SDL_Gamepad_Mapping)`
- `SDL_HAT_UP_BIT :: 0x01`
- `SDL_HAT_RIGHT_BIT :: 0x02`
- `SDL_HAT_DOWN_BIT :: 0x04`
- `SDL_HAT_LEFT_BIT :: 0x08`
- `input_name_to_input_target :: (name: string) -> SDL_Input_Target`
- `parse_gamepad_mapping_string :: (input: string) -> map: SDL_Gamepad_Mapping, success: bool`
- `get_guid_string :: (bustype: s16, vendor: s16, product: s16, version: s16) -> temp_string: string`
- `SDL_BUILTIN_GAME_CONTROLLER_DB :: #string`

## Public type members

#### `SDL_Input_Target` (enum)

```jai
SDL_Input_Target :: enum {
    UNINITIALIZED;
    LEFTX;
    RIGHTX;
    LEFTY;
    RIGHTY;
    LEFT_TRIGGER;
    RIGHT_TRIGGER;
    DPAD_LEFT;
    DPAD_RIGHT;
    DPAD_UP;
    DPAD_DOWN;
    BACK;
    GUIDE;
    START;
    LEFT_SHOULDER;
    RIGHT_SHOULDER;
    LEFT_STICK;
    RIGHT_STICK;
    A;
    B;
    X;
    Y;
}
```
#### `SDL_Input_Type` (enum)

```jai
SDL_Input_Type :: enum {
    UNINITIALIZED;
    BUTTON;
    AXIS;
    HAT;
}
```
#### `SDL_Input_Map` (struct)

```jai
SDL_Input_Map :: struct {
    target: SDL_Input_Target;
    type: SDL_Input_Type;
    index: s32 = -1;
    subindex: s32 = -1;
}
```
#### `SDL_Gamepad_Mapping` (struct)

```jai
SDL_Gamepad_Mapping :: struct {
    guid: string;
    name: string;
    platform: string;
    mappings: [..] SDL_Input_Map;
}
```
## Behavioral contract and constraints

`parse_gamepad_mapping_string` parses an SDL controller mapping into owned strings and a growable map array; `deinit` frees the three strings and resets the array. The four hat constants describe the UP, RIGHT, DOWN, and LEFT bit positions used for hat mappings. Unknown input names are reported as an unsuccessful mapping parse. `SDL_BUILTIN_GAME_CONTROLLER_DB` is an immutable `#string` in the SDL 2.0.9 mapping format, containing 614 newline-delimited mapping records plus two comment lines. Its origin is attributed to the community database now published at [mdqinc/SDL_GameControllerDB](https://github.com/mdqinc/SDL_GameControllerDB), the successor location identified by the project's maintainers. For replacement implementations, preserve the public constant as a newline-delimited SDL mapping database and pin the selected database artifact in the implementation's own dependency metadata; byte-for-byte equality with the embedded data is not guaranteed. Controller GUID coverage and therefore the exact mappings produced by consumers can differ as that data changes. The embedded payload's source-byte fingerprint is SHA-256 `3ef1543d7367b50e62293743f454106438be2fd921865bf5de82b9964f7c005d`, computed over the payload from the first comment line through the final record, retaining its CRLF line endings.

`get_guid_string` formats the four 16-bit bus/vendor/product/version values as a 32-character hexadecimal GUID string, writing each 16-bit value's low byte before its high byte. Its result uses temporary storage and is valid only for the associated temporary-storage lifetime. The byte order is fixed by the GUID format and does not vary with host endianness.
