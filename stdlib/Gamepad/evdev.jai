// Regenerate with tools/window_render_evdev.py and the source-built repository formatter.
libevdev :: struct {}
libevdev_uinput :: struct {}

// allocation
libevdev_free :: (dev: *libevdev) -> void #foreign evdev_library;
libevdev_new :: () -> *libevdev #foreign evdev_library;
libevdev_new_from_fd :: (fd: s32, dev: **libevdev) -> s32 #foreign evdev_library;

// devices
libevdev_change_fd :: (dev: *libevdev, fd: s32) -> s32 #foreign evdev_library;
libevdev_get_fd :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_id_bustype :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_id_product :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_id_vendor :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_id_version :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_log_priority :: () -> libevdev_log_priority #foreign evdev_library;
libevdev_get_phys :: (dev: *libevdev) -> *u8 #foreign evdev_library;
libevdev_get_uniq :: (dev: *libevdev) -> *u8 #foreign evdev_library;
libevdev_grab :: (dev: *libevdev, grab: libevdev_grab_mode) -> s32 #foreign evdev_library;
libevdev_set_clock_id :: (dev: *libevdev, clockid: s32) -> s32 #foreign evdev_library;
libevdev_set_fd :: (dev: *libevdev, fd: s32) -> s32 #foreign evdev_library;
libevdev_set_id_bustype :: (dev: *libevdev, bustype: s32) -> void #foreign evdev_library;
libevdev_set_id_product :: (dev: *libevdev, product_id: s32) -> void #foreign evdev_library;
libevdev_set_id_vendor :: (dev: *libevdev, vendor_id: s32) -> void #foreign evdev_library;
libevdev_set_id_version :: (dev: *libevdev, version: s32) -> void #foreign evdev_library;
libevdev_set_log_priority :: (priority: libevdev_log_priority) -> void #foreign evdev_library;
libevdev_set_phys :: (dev: *libevdev, phys: *u8) -> void #foreign evdev_library;
libevdev_set_uniq :: (dev: *libevdev, uniq: *u8) -> void #foreign evdev_library;

// capabilities
libevdev_enable_property :: (dev: *libevdev, prop: u32) -> s32 #foreign evdev_library;
libevdev_get_driver_version :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_repeat :: (dev: *libevdev, delay: *s32, period: *s32) -> s32 #foreign evdev_library;
libevdev_has_property :: (dev: *libevdev, prop: u32) -> s32 #foreign evdev_library;
libevdev_kernel_set_led_value :: (dev: *libevdev, code: u32, value: libevdev_led_value) -> s32 #foreign evdev_library;
libevdev_kernel_set_led_values :: (dev: *libevdev, __args: ..Any) -> s32 #foreign evdev_library;
libevdev_set_name :: (dev: *libevdev, name: *u8) -> void #foreign evdev_library;

// events
libevdev_disable_event_code :: (dev: *libevdev, type: u32, code: u32) -> s32 #foreign evdev_library;
libevdev_disable_event_type :: (dev: *libevdev, type: u32) -> s32 #foreign evdev_library;
libevdev_enable_event_code :: (dev: *libevdev, type: u32, code: u32, data: *void) -> s32 #foreign evdev_library;
libevdev_enable_event_type :: (dev: *libevdev, type: u32) -> s32 #foreign evdev_library;
libevdev_fetch_event_value :: (dev: *libevdev, type: u32, code: u32, value: *s32) -> s32 #foreign evdev_library;
libevdev_fetch_slot_value :: (dev: *libevdev, slot: u32, code: u32, value: *s32) -> s32 #foreign evdev_library;
libevdev_get_abs_flat :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library;
libevdev_get_abs_fuzz :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library;
libevdev_get_abs_info :: (dev: *libevdev, code: u32) -> *input_absinfo #foreign evdev_library;
libevdev_get_abs_maximum :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library;
libevdev_get_abs_minimum :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library;
libevdev_get_abs_resolution :: (dev: *libevdev, code: u32) -> s32 #foreign evdev_library;
libevdev_get_current_slot :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_event_value :: (dev: *libevdev, type: u32, code: u32) -> s32 #foreign evdev_library;
libevdev_get_num_slots :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_get_slot_value :: (dev: *libevdev, slot: u32, code: u32) -> s32 #foreign evdev_library;
libevdev_has_event_code :: (dev: *libevdev, type: u32, code: u32) -> s32 #foreign evdev_library;
libevdev_has_event_pending :: (dev: *libevdev) -> s32 #foreign evdev_library;
libevdev_has_event_type :: (dev: *libevdev, type: u32) -> s32 #foreign evdev_library;
libevdev_kernel_set_abs_info :: (dev: *libevdev, code: u32, abs: *input_absinfo) -> s32 #foreign evdev_library;
libevdev_next_event :: (dev: *libevdev, flags: libevdev_read_flag, ev: *input_event) -> s32 #foreign evdev_library;
libevdev_set_abs_flat :: (dev: *libevdev, code: u32, flat: s32) -> void #foreign evdev_library;
libevdev_set_abs_fuzz :: (dev: *libevdev, code: u32, fuzz: s32) -> void #foreign evdev_library;
libevdev_set_abs_info :: (dev: *libevdev, code: u32, abs: *input_absinfo) -> void #foreign evdev_library;
libevdev_set_abs_maximum :: (dev: *libevdev, code: u32, max: s32) -> void #foreign evdev_library;
libevdev_set_abs_minimum :: (dev: *libevdev, code: u32, min: s32) -> void #foreign evdev_library;
libevdev_set_abs_resolution :: (dev: *libevdev, code: u32, resolution: s32) -> void #foreign evdev_library;
libevdev_set_event_value :: (dev: *libevdev, type: u32, code: u32, value: s32) -> s32 #foreign evdev_library;
libevdev_set_slot_value :: (dev: *libevdev, slot: u32, code: u32, value: s32) -> s32 #foreign evdev_library;

// text
libevdev_event_code_from_code_name :: (name: *u8) -> s32 #foreign evdev_library;
libevdev_event_code_from_code_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library;
libevdev_event_code_from_name :: (type: u32, name: *u8) -> s32 #foreign evdev_library;
libevdev_event_code_from_name_n :: (type: u32, name: *u8, len: u64) -> s32 #foreign evdev_library;
libevdev_event_code_get_name :: (type: u32, code: u32) -> *u8 #foreign evdev_library;
libevdev_event_is_code :: (ev: *input_event, type: u32, code: u32) -> s32 #foreign evdev_library;
libevdev_event_is_type :: (ev: *input_event, type: u32) -> s32 #foreign evdev_library;
libevdev_event_type_from_code_name :: (name: *u8) -> s32 #foreign evdev_library;
libevdev_event_type_from_code_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library;
libevdev_event_type_from_name :: (name: *u8) -> s32 #foreign evdev_library;
libevdev_event_type_from_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library;
libevdev_event_type_get_max :: (type: u32) -> s32 #foreign evdev_library;
libevdev_event_type_get_name :: (type: u32) -> *u8 #foreign evdev_library;
libevdev_event_value_from_name :: (type: u32, code: u32, name: *u8) -> s32 #foreign evdev_library;
libevdev_event_value_from_name_n :: (type: u32, code: u32, name: *u8, len: u64) -> s32 #foreign evdev_library;
libevdev_event_value_get_name :: (type: u32, code: u32, value: s32) -> *u8 #foreign evdev_library;
libevdev_get_name :: (dev: *libevdev) -> *u8 #foreign evdev_library;
libevdev_property_from_name :: (name: *u8) -> s32 #foreign evdev_library;
libevdev_property_from_name_n :: (name: *u8, len: u64) -> s32 #foreign evdev_library;
libevdev_property_get_name :: (prop: u32) -> *u8 #foreign evdev_library;

// output
libevdev_uinput_create_from_device :: (dev: *libevdev, uinput_fd: s32, uinput_dev: **libevdev_uinput) -> s32 #foreign evdev_library;
libevdev_uinput_destroy :: (uinput_dev: *libevdev_uinput) -> void #foreign evdev_library;
libevdev_uinput_get_devnode :: (uinput_dev: *libevdev_uinput) -> *u8 #foreign evdev_library;
libevdev_uinput_get_fd :: (uinput_dev: *libevdev_uinput) -> s32 #foreign evdev_library;
libevdev_uinput_get_syspath :: (uinput_dev: *libevdev_uinput) -> *u8 #foreign evdev_library;
libevdev_uinput_write_event :: (uinput_dev: *libevdev_uinput, type: u32, code: u32, value: s32) -> s32 #foreign evdev_library;

#load "../internal/gamepad/linux-abi.jai";
