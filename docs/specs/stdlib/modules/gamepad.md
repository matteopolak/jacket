# Gamepad

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `libevdev :: struct`
- `libevdev_new :: () -> *libevdev #foreign evdev_library`
- `libevdev_new_from_fd :: (fd: s32, dev: **libevdev) -> s32 #foreign evdev_library`
- `libevdev_free :: (dev: *libevdev) -> void #foreign evdev_library`
- `libevdev_set_log_priority :: (priority: libevdev_log_priority) -> void #foreign evdev_library`
- `libevdev_get_log_priority :: () -> libevdev_log_priority #foreign evdev_library`
- `libevdev_grab :: (dev: *libevdev, grab: libevdev_grab_mode) -> s32 #foreign evdev_library`
- `libevdev_set_fd :: (dev: *libevdev, fd: s32) -> s32 #foreign evdev_library`
- `libevdev_change_fd :: (dev: *libevdev, fd: s32) -> s32 #foreign evdev_library`
- `libevdev_get_fd :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_next_event :: (dev: *libevdev, flags: libevdev_read_flag, ev: *input_event) -> s32 #foreign evdev_library`
- `libevdev_has_event_pending :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_get_name :: (dev: *libevdev) -> *u8 #foreign evdev_library`
- `libevdev_set_name :: (dev: *libevdev, name: *u8) -> void #foreign evdev_library`
- `libevdev_get_phys :: (dev: *libevdev) -> *u8 #foreign evdev_library`
- `libevdev_set_phys :: (dev: *libevdev, phys: *u8) -> void #foreign evdev_library`
- `libevdev_get_uniq :: (dev: *libevdev) -> *u8 #foreign evdev_library`
- `libevdev_set_uniq :: (dev: *libevdev, uniq: *u8) -> void #foreign evdev_library`
- `libevdev_get_id_product :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_set_id_product :: (dev: *libevdev, product_id: s32) -> void #foreign evdev_library`
- `libevdev_get_id_vendor :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_set_id_vendor :: (dev: *libevdev, vendor_id: s32) -> void #foreign evdev_library`
- `libevdev_get_id_bustype :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_set_id_bustype :: (dev: *libevdev, bustype: s32) -> void #foreign evdev_library`
- `libevdev_get_id_version :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_set_id_version :: (dev: *libevdev, version: s32) -> void #foreign evdev_library`
- `libevdev_get_driver_version :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_has_property :: (dev: *libevdev, prop: u32) -> s32 #foreign evdev_library`
- `libevdev_enable_property :: (dev: *libevdev, prop: u32) -> s32 #foreign evdev_library`
- `libevdev_has_event_type :: (dev: *libevdev, type: u32) -> s32 #foreign evdev_library`
- `libevdev_has_event_code :: (dev: *libevdev, type: u32, code: u32) -> s32 #foreign evdev_library`
- `libevdev_get_abs_minimum :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library`
- `libevdev_get_abs_maximum :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library`
- `libevdev_get_abs_fuzz :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library`
- `libevdev_get_abs_flat :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library`
- `libevdev_get_abs_resolution :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library`
- `libevdev_get_abs_info :: (dev: *libevdev, code: u32) -> *input_absinfo #foreign evdev_library`
- `libevdev_get_event_value :: (dev: *libevdev, type: u32, code: u32) -> s32 #foreign evdev_library`
- `libevdev_set_event_value :: (dev: *libevdev, type: u32, code: u32, value: s32) -> s32 #foreign evdev_library`
- `libevdev_fetch_event_value :: (dev: *libevdev, type: u32, code: u32, value: *s32) -> s32 #foreign evdev_library`
- `libevdev_get_slot_value :: (dev: *libevdev, slot: u32, code: u32) -> s32 #foreign evdev_library`
- `libevdev_set_slot_value :: (dev: *libevdev, slot: u32, code: u32, value: s32) -> s32 #foreign evdev_library`
- `libevdev_fetch_slot_value :: (dev: *libevdev, slot: u32, code: u32, value: *s32) -> s32 #foreign evdev_library`
- `libevdev_get_num_slots :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_get_current_slot :: (dev: *libevdev) -> s32 #foreign evdev_library`
- `libevdev_set_abs_minimum :: (dev: *libevdev, code: u32, min: s32) -> void #foreign evdev_library`
- `libevdev_set_abs_maximum :: (dev: *libevdev, code: u32, max: s32) -> void #foreign evdev_library`
- `libevdev_set_abs_fuzz :: (dev: *libevdev, code: u32, fuzz: s32) -> void #foreign evdev_library`
- `libevdev_set_abs_flat :: (dev: *libevdev, code: u32, flat: s32) -> void #foreign evdev_library`
- `libevdev_set_abs_resolution :: (dev: *libevdev, code: u32, resolution: s32) -> void #foreign evdev_library`
- `libevdev_set_abs_info :: (dev: *libevdev, code: u32, abs: *input_absinfo) -> void #foreign evdev_library`
- `libevdev_enable_event_type :: (dev: *libevdev, type: u32) -> s32 #foreign evdev_library`
- `libevdev_disable_event_type :: (dev: *libevdev, type: u32) -> s32 #foreign evdev_library`
- `libevdev_enable_event_code :: (dev: *libevdev, type: u32, code: u32, data: *void) -> s32 #foreign evdev_library`
- `libevdev_disable_event_code :: (dev: *libevdev, type: u32, code: u32) -> s32 #foreign evdev_library`
- `libevdev_kernel_set_abs_info :: (dev: *libevdev, code: u32, abs: *input_absinfo) -> s32 #foreign evdev_library`
- `libevdev_kernel_set_led_value :: (dev: *libevdev, code: u32, value: libevdev_led_value) -> s32 #foreign evdev_library`
- `libevdev_kernel_set_led_values :: (dev: *libevdev, __args: ..Any) -> s32 #foreign evdev_library`
- `libevdev_set_clock_id :: (dev: *libevdev, clockid: s32) -> s32 #foreign evdev_library`
- `libevdev_event_is_type :: (ev: *input_event, type: u32) -> s32 #foreign evdev_library`
- `libevdev_event_is_code :: (ev: *input_event, type: u32, code: u32) -> s32 #foreign evdev_library`
- `libevdev_event_type_get_name :: (type: u32) -> *u8 #foreign evdev_library`
- `libevdev_event_code_get_name :: (type: u32, code: u32) -> *u8 #foreign evdev_library`
- `libevdev_event_value_get_name :: (type: u32, code: u32, value: s32) -> *u8 #foreign evdev_library`
- `libevdev_property_get_name :: (prop: u32) -> *u8 #foreign evdev_library`
- `libevdev_event_type_get_max :: (type: u32) -> s32 #foreign evdev_library`
- `libevdev_event_type_from_name :: (name: *u8) -> s32 #foreign evdev_library`
- `libevdev_event_type_from_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library`
- `libevdev_event_code_from_name :: (type: u32, name: *u8) -> s32 #foreign evdev_library`
- `libevdev_event_code_from_name_n :: (type: u32, name: *u8, len: u64) -> s32 #foreign evdev_library`
- `libevdev_event_value_from_name :: (type: u32, code: u32, name: *u8) -> s32 #foreign evdev_library`
- `libevdev_event_type_from_code_name :: (name: *u8) -> s32 #foreign evdev_library`
- `libevdev_event_type_from_code_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library`
- `libevdev_event_code_from_code_name :: (name: *u8) -> s32 #foreign evdev_library`
- `libevdev_event_code_from_code_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library`
- `libevdev_event_value_from_name_n :: (type: u32, code: u32, name: *u8, len: u64) -> s32 #foreign evdev_library`
- `libevdev_property_from_name :: (name: *u8) -> s32 #foreign evdev_library`
- `libevdev_property_from_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library`
- `libevdev_get_repeat :: (dev: *libevdev, delay: *s32, period: *s32) -> s32 #foreign evdev_library`
- `libevdev_uinput :: struct`
- `libevdev_uinput_create_from_device :: (dev: *libevdev, uinput_fd: s32, uinput_dev: **libevdev_uinput) -> s32 #foreign evdev_library`
- `libevdev_uinput_destroy :: (uinput_dev: *libevdev_uinput) -> void #foreign evdev_library`
- `libevdev_uinput_get_fd :: (uinput_dev: *libevdev_uinput) -> s32 #foreign evdev_library`
- `libevdev_uinput_get_syspath :: (uinput_dev: *libevdev_uinput) -> *u8 #foreign evdev_library`
- `libevdev_uinput_get_devnode :: (uinput_dev: *libevdev_uinput) -> *u8 #foreign evdev_library`
- `libevdev_uinput_write_event :: (uinput_dev: *libevdev_uinput, type: u32, code: u32, value: s32) -> s32 #foreign evdev_library`
- `Gamepad :: struct`
- `Gamepad_Code :: enum`

### Linux-specific declarations

- `init_gamepad :: ()`
- `update_gamepad :: ()`

### macOS-specific declarations

- `init_gamepad :: ()`
- `update_gamepad :: ()`

### Windows-specific declarations

- `init_gamepad :: ()`
- `update_gamepad :: ()`

- `gamepad: Gamepad`

## Public type members

#### `libevdev` (empty opaque Linux struct)

```jai
libevdev :: struct {}
```
#### `libevdev_uinput` (empty opaque Linux struct)

```jai
libevdev_uinput :: struct {}
```
#### `Gamepad` (struct)

- `connected: bool`
- `buttons: [] Key_Current_State`
- `left_stick:  Math.Vector2`
- `right_stick: Math.Vector2`
- `left_trigger:  float`
- `right_trigger: float`
- `generate_events: bool`
- `generate_left_stick_events: bool`
- `vendor_id:  u16 = 0xFFFF`
- `product_id: u16 = 0xFFFF`
- `left_stick_threshold    := 0.5`
- `right_stick_threshold   := 0.5`
- `left_trigger_threshold  := 0.3`
- `right_trigger_threshold := 0.3`

#### `Gamepad_Code` (enum)

- `DPAD_UP`
- `DPAD_DOWN`
- `DPAD_LEFT`
- `DPAD_RIGHT`
- `START`
- `BACK`
- `LEFT_SHOULDER`
- `RIGHT_SHOULDER`
- `LEFT_TRIGGER`
- `RIGHT_TRIGGER`
- `FACE_DOWN`
- `FACE_RIGHT`
- `FACE_LEFT`
- `FACE_UP`
- `LEFT_STICK_UP`
- `LEFT_STICK_DOWN`
- `LEFT_STICK_LEFT`
- `LEFT_STICK_RIGHT`
- `RIGHT_STICK_UP`
- `RIGHT_STICK_DOWN`
- `RIGHT_STICK_LEFT`
- `RIGHT_STICK_RIGHT`
- `LEFT_STICK_CLICK`
- `RIGHT_STICK_CLICK`
- `A :: FACE_DOWN`
- `B :: FACE_RIGHT`
- `X :: FACE_LEFT`
- `Y :: FACE_UP`

## Behavioral contract and constraints

The module supports gamepad/HID discovery and input through OS-specific backends, including libevdev on Linux and native device APIs on Apple platforms. Device handles and event data are backend-owned; callers should use the module's close and cleanup operations and treat event availability as asynchronous. Supported controls and error codes depend on the selected backend and connected device.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: `IOKit`, `hid`, `libevdev`.


Platform coverage note: exports for PS5 targets are unspecified.
