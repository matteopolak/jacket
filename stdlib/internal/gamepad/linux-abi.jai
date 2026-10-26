// Contextual ABI types from libevdev 1.13.3 and Linux v6.12 public C headers.
// These support exported native signatures without adding facade exports.
#scope_module
evdev_library :: #library,system "evdev";

Kernel_Time :: struct {
    tv_sec: s64;
    tv_usec: s64;
}

input_event :: struct {
    time: Kernel_Time;
    type: u16;
    code: u16;
    value: s32;
}

input_absinfo :: struct {
    value: s32;
    minimum: s32;
    maximum: s32;
    fuzz: s32;
    flat: s32;
    resolution: s32;
}

libevdev_read_flag :: enum_flags s32 {
    SYNC :: 1;
    NORMAL :: 2;
    FORCE_SYNC :: 4;
    BLOCKING :: 8;
}

libevdev_grab_mode :: enum s32 {
    LIBEVDEV_GRAB :: 3;
    LIBEVDEV_UNGRAB :: 4;
}

libevdev_log_priority :: enum s32 {
    LIBEVDEV_LOG_ERROR :: 10;
    LIBEVDEV_LOG_INFO :: 20;
    LIBEVDEV_LOG_DEBUG :: 30;
}

libevdev_led_value :: enum s32 {
    LIBEVDEV_LED_ON :: 3;
    LIBEVDEV_LED_OFF :: 4;
}
