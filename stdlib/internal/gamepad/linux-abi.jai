// Contextual ABI types from libevdev 1.13.3 and Linux v6.12 public C headers.
// These support exported native signatures without adding facade exports.
#scope_module
evdev_library :: #library,system "evdev";

Linux :: #import "Linux";

input_event :: Linux.input_event;
input_absinfo :: Linux.input_absinfo;

libevdev_read_flag :: enum_flags s32 {
    LIBEVDEV_READ_FLAG_SYNC :: 1;
    LIBEVDEV_READ_FLAG_NORMAL :: 2;
    LIBEVDEV_READ_FLAG_FORCE_SYNC :: 4;
    LIBEVDEV_READ_FLAG_BLOCKING :: 8;
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
