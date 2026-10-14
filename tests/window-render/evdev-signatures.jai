#import "Basic";
G :: #import "Gamepad";

main :: () {
    libevdev_change_fd: #type (dev: *G.libevdev, fd: s32) -> s32 #c_call = G.libevdev_change_fd;

    assert(libevdev_change_fd != null);
    libevdev_disable_event_code: #type (dev: *G.libevdev, type: u32, code: u32) -> s32 #c_call = G.libevdev_disable_event_code;

    assert(libevdev_disable_event_code != null);
    libevdev_disable_event_type: #type (dev: *G.libevdev, type: u32) -> s32 #c_call = G.libevdev_disable_event_type;

    assert(libevdev_disable_event_type != null);
    libevdev_enable_event_code: #type (dev: *G.libevdev, type: u32, code: u32, data: *void) -> s32 #c_call = G.libevdev_enable_event_code;

    assert(libevdev_enable_event_code != null);
    libevdev_enable_event_type: #type (dev: *G.libevdev, type: u32) -> s32 #c_call = G.libevdev_enable_event_type;

    assert(libevdev_enable_event_type != null);
    libevdev_enable_property: #type (dev: *G.libevdev, prop: u32) -> s32 #c_call = G.libevdev_enable_property;

    assert(libevdev_enable_property != null);
    libevdev_event_code_from_code_name: #type (name: *u8) -> s32 #c_call = G.libevdev_event_code_from_code_name;

    assert(libevdev_event_code_from_code_name != null);
    libevdev_event_code_from_code_name_n: #type (name: *u8, len: u64) -> s32 #c_call = G.libevdev_event_code_from_code_name_n;

    assert(libevdev_event_code_from_code_name_n != null);
    libevdev_event_code_from_name: #type (type: u32, name: *u8) -> s32 #c_call = G.libevdev_event_code_from_name;

    assert(libevdev_event_code_from_name != null);
    libevdev_event_code_from_name_n: #type (type: u32, name: *u8, len: u64) -> s32 #c_call = G.libevdev_event_code_from_name_n;

    assert(libevdev_event_code_from_name_n != null);
    libevdev_event_code_get_name: #type (type: u32, code: u32) -> *u8 #c_call = G.libevdev_event_code_get_name;

    assert(libevdev_event_code_get_name != null);
    #assert size_of(type_of(G.libevdev_event_is_code)) == size_of(*void);
    #assert size_of(type_of(G.libevdev_event_is_type)) == size_of(*void);
    libevdev_event_type_from_code_name: #type (name: *u8) -> s32 #c_call = G.libevdev_event_type_from_code_name;

    assert(libevdev_event_type_from_code_name != null);
    libevdev_event_type_from_code_name_n: #type (name: *u8, len: u64) -> s32 #c_call = G.libevdev_event_type_from_code_name_n;

    assert(libevdev_event_type_from_code_name_n != null);
    libevdev_event_type_from_name: #type (name: *u8) -> s32 #c_call = G.libevdev_event_type_from_name;

    assert(libevdev_event_type_from_name != null);
    libevdev_event_type_from_name_n: #type (name: *u8, len: u64) -> s32 #c_call = G.libevdev_event_type_from_name_n;

    assert(libevdev_event_type_from_name_n != null);
    libevdev_event_type_get_max: #type (type: u32) -> s32 #c_call = G.libevdev_event_type_get_max;

    assert(libevdev_event_type_get_max != null);
    libevdev_event_type_get_name: #type (type: u32) -> *u8 #c_call = G.libevdev_event_type_get_name;

    assert(libevdev_event_type_get_name != null);
    libevdev_event_value_from_name: #type (type: u32, code: u32, name: *u8) -> s32 #c_call = G.libevdev_event_value_from_name;

    assert(libevdev_event_value_from_name != null);
    libevdev_event_value_from_name_n: #type (type: u32, code: u32, name: *u8, len: u64) -> s32 #c_call = G.libevdev_event_value_from_name_n;

    assert(libevdev_event_value_from_name_n != null);
    libevdev_event_value_get_name: #type (type: u32, code: u32, value: s32) -> *u8 #c_call = G.libevdev_event_value_get_name;

    assert(libevdev_event_value_get_name != null);
    libevdev_fetch_event_value: #type (dev: *G.libevdev, type: u32, code: u32, value: *s32) -> s32 #c_call = G.libevdev_fetch_event_value;

    assert(libevdev_fetch_event_value != null);
    libevdev_fetch_slot_value: #type (dev: *G.libevdev, slot: u32, code: u32, value: *s32) -> s32 #c_call = G.libevdev_fetch_slot_value;

    assert(libevdev_fetch_slot_value != null);
    libevdev_free: #type (dev: *G.libevdev) -> void #c_call = G.libevdev_free;

    assert(libevdev_free != null);
    libevdev_get_abs_flat: #type (dev: *G.libevdev, code: u32) -> s32 #c_call = G.libevdev_get_abs_flat;

    assert(libevdev_get_abs_flat != null);
    libevdev_get_abs_fuzz: #type (dev: *G.libevdev, code: u32) -> s32 #c_call = G.libevdev_get_abs_fuzz;

    assert(libevdev_get_abs_fuzz != null);
    #assert size_of(type_of(G.libevdev_get_abs_info)) == size_of(*void);
    libevdev_get_abs_maximum: #type (dev: *G.libevdev, code: u32) -> s32 #c_call = G.libevdev_get_abs_maximum;

    assert(libevdev_get_abs_maximum != null);
    libevdev_get_abs_minimum: #type (dev: *G.libevdev, code: u32) -> s32 #c_call = G.libevdev_get_abs_minimum;

    assert(libevdev_get_abs_minimum != null);
    libevdev_get_abs_resolution: #type (dev: *G.libevdev, code: u32) -> s32 #c_call = G.libevdev_get_abs_resolution;

    assert(libevdev_get_abs_resolution != null);
    libevdev_get_current_slot: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_current_slot;

    assert(libevdev_get_current_slot != null);
    libevdev_get_driver_version: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_driver_version;

    assert(libevdev_get_driver_version != null);
    libevdev_get_event_value: #type (dev: *G.libevdev, type: u32, code: u32) -> s32 #c_call = G.libevdev_get_event_value;

    assert(libevdev_get_event_value != null);
    libevdev_get_fd: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_fd;

    assert(libevdev_get_fd != null);
    libevdev_get_id_bustype: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_id_bustype;

    assert(libevdev_get_id_bustype != null);
    libevdev_get_id_product: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_id_product;

    assert(libevdev_get_id_product != null);
    libevdev_get_id_vendor: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_id_vendor;

    assert(libevdev_get_id_vendor != null);
    libevdev_get_id_version: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_id_version;

    assert(libevdev_get_id_version != null);
    #assert size_of(type_of(G.libevdev_get_log_priority)) == size_of(*void);
    libevdev_get_name: #type (dev: *G.libevdev) -> *u8 #c_call = G.libevdev_get_name;

    assert(libevdev_get_name != null);
    libevdev_get_num_slots: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_get_num_slots;

    assert(libevdev_get_num_slots != null);
    libevdev_get_phys: #type (dev: *G.libevdev) -> *u8 #c_call = G.libevdev_get_phys;

    assert(libevdev_get_phys != null);
    libevdev_get_repeat: #type (dev: *G.libevdev, delay: *s32, period: *s32) -> s32 #c_call = G.libevdev_get_repeat;

    assert(libevdev_get_repeat != null);
    libevdev_get_slot_value: #type (dev: *G.libevdev, slot: u32, code: u32) -> s32 #c_call = G.libevdev_get_slot_value;

    assert(libevdev_get_slot_value != null);
    libevdev_get_uniq: #type (dev: *G.libevdev) -> *u8 #c_call = G.libevdev_get_uniq;

    assert(libevdev_get_uniq != null);
    #assert size_of(type_of(G.libevdev_grab)) == size_of(*void);
    libevdev_has_event_code: #type (dev: *G.libevdev, type: u32, code: u32) -> s32 #c_call = G.libevdev_has_event_code;

    assert(libevdev_has_event_code != null);
    libevdev_has_event_pending: #type (dev: *G.libevdev) -> s32 #c_call = G.libevdev_has_event_pending;

    assert(libevdev_has_event_pending != null);
    libevdev_has_event_type: #type (dev: *G.libevdev, type: u32) -> s32 #c_call = G.libevdev_has_event_type;

    assert(libevdev_has_event_type != null);
    libevdev_has_property: #type (dev: *G.libevdev, prop: u32) -> s32 #c_call = G.libevdev_has_property;

    assert(libevdev_has_property != null);
    #assert size_of(type_of(G.libevdev_kernel_set_abs_info)) == size_of(*void);
    #assert size_of(type_of(G.libevdev_kernel_set_led_value)) == size_of(*void);
    libevdev_kernel_set_led_values: #type (dev: *G.libevdev, __args: ..Any) -> s32 #c_call = G.libevdev_kernel_set_led_values;

    assert(libevdev_kernel_set_led_values != null);
    libevdev_new: #type () -> *G.libevdev #c_call = G.libevdev_new;

    assert(libevdev_new != null);
    libevdev_new_from_fd: #type (fd: s32, dev: **G.libevdev) -> s32 #c_call = G.libevdev_new_from_fd;

    assert(libevdev_new_from_fd != null);
    #assert size_of(type_of(G.libevdev_next_event)) == size_of(*void);
    libevdev_property_from_name: #type (name: *u8) -> s32 #c_call = G.libevdev_property_from_name;

    assert(libevdev_property_from_name != null);
    libevdev_property_from_name_n: #type (name: *u8, len: u64) -> s32 #c_call = G.libevdev_property_from_name_n;

    assert(libevdev_property_from_name_n != null);
    libevdev_property_get_name: #type (prop: u32) -> *u8 #c_call = G.libevdev_property_get_name;

    assert(libevdev_property_get_name != null);
    libevdev_set_abs_flat: #type (dev: *G.libevdev, code: u32, flat: s32) -> void #c_call = G.libevdev_set_abs_flat;

    assert(libevdev_set_abs_flat != null);
    libevdev_set_abs_fuzz: #type (dev: *G.libevdev, code: u32, fuzz: s32) -> void #c_call = G.libevdev_set_abs_fuzz;

    assert(libevdev_set_abs_fuzz != null);
    #assert size_of(type_of(G.libevdev_set_abs_info)) == size_of(*void);
    libevdev_set_abs_maximum: #type (dev: *G.libevdev, code: u32, max: s32) -> void #c_call = G.libevdev_set_abs_maximum;

    assert(libevdev_set_abs_maximum != null);
    libevdev_set_abs_minimum: #type (dev: *G.libevdev, code: u32, min: s32) -> void #c_call = G.libevdev_set_abs_minimum;

    assert(libevdev_set_abs_minimum != null);
    libevdev_set_abs_resolution: #type (dev: *G.libevdev, code: u32, resolution: s32) -> void #c_call = G.libevdev_set_abs_resolution;

    assert(libevdev_set_abs_resolution != null);
    libevdev_set_clock_id: #type (dev: *G.libevdev, clockid: s32) -> s32 #c_call = G.libevdev_set_clock_id;

    assert(libevdev_set_clock_id != null);
    libevdev_set_event_value: #type (dev: *G.libevdev, type: u32, code: u32, value: s32) -> s32 #c_call = G.libevdev_set_event_value;

    assert(libevdev_set_event_value != null);
    libevdev_set_fd: #type (dev: *G.libevdev, fd: s32) -> s32 #c_call = G.libevdev_set_fd;

    assert(libevdev_set_fd != null);
    libevdev_set_id_bustype: #type (dev: *G.libevdev, bustype: s32) -> void #c_call = G.libevdev_set_id_bustype;

    assert(libevdev_set_id_bustype != null);
    libevdev_set_id_product: #type (dev: *G.libevdev, product_id: s32) -> void #c_call = G.libevdev_set_id_product;

    assert(libevdev_set_id_product != null);
    libevdev_set_id_vendor: #type (dev: *G.libevdev, vendor_id: s32) -> void #c_call = G.libevdev_set_id_vendor;

    assert(libevdev_set_id_vendor != null);
    libevdev_set_id_version: #type (dev: *G.libevdev, version: s32) -> void #c_call = G.libevdev_set_id_version;

    assert(libevdev_set_id_version != null);
    #assert size_of(type_of(G.libevdev_set_log_priority)) == size_of(*void);
    libevdev_set_name: #type (dev: *G.libevdev, name: *u8) -> void #c_call = G.libevdev_set_name;

    assert(libevdev_set_name != null);
    libevdev_set_phys: #type (dev: *G.libevdev, phys: *u8) -> void #c_call = G.libevdev_set_phys;

    assert(libevdev_set_phys != null);
    libevdev_set_slot_value: #type (dev: *G.libevdev, slot: u32, code: u32, value: s32) -> s32 #c_call = G.libevdev_set_slot_value;

    assert(libevdev_set_slot_value != null);
    libevdev_set_uniq: #type (dev: *G.libevdev, uniq: *u8) -> void #c_call = G.libevdev_set_uniq;

    assert(libevdev_set_uniq != null);
    libevdev_uinput_create_from_device: #type (dev: *G.libevdev, uinput_fd: s32, uinput_dev: **G.libevdev_uinput) -> s32 #c_call = G.libevdev_uinput_create_from_device;

    assert(libevdev_uinput_create_from_device != null);
    libevdev_uinput_destroy: #type (uinput_dev: *G.libevdev_uinput) -> void #c_call = G.libevdev_uinput_destroy;

    assert(libevdev_uinput_destroy != null);
    libevdev_uinput_get_devnode: #type (uinput_dev: *G.libevdev_uinput) -> *u8 #c_call = G.libevdev_uinput_get_devnode;

    assert(libevdev_uinput_get_devnode != null);
    libevdev_uinput_get_fd: #type (uinput_dev: *G.libevdev_uinput) -> s32 #c_call = G.libevdev_uinput_get_fd;

    assert(libevdev_uinput_get_fd != null);
    libevdev_uinput_get_syspath: #type (uinput_dev: *G.libevdev_uinput) -> *u8 #c_call = G.libevdev_uinput_get_syspath;

    assert(libevdev_uinput_get_syspath != null);
    libevdev_uinput_write_event: #type (uinput_dev: *G.libevdev_uinput, type: u32, code: u32, value: s32) -> s32 #c_call = G.libevdev_uinput_write_event;

    assert(libevdev_uinput_write_event != null);
    assert(size_of(G.libevdev) == 0 && size_of(G.libevdev_uinput) == 0);
    print("ok\n");
}
