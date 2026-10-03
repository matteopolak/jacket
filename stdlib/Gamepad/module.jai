Gamepad_Code :: enum {
    DPAD_UP :: 0;
    DPAD_DOWN :: 1;
    DPAD_LEFT :: 2;
    DPAD_RIGHT :: 3;
    START :: 4;
    BACK :: 5;
    LEFT_SHOULDER :: 6;
    RIGHT_SHOULDER :: 7;
    LEFT_TRIGGER :: 8;
    RIGHT_TRIGGER :: 9;
    FACE_DOWN :: 10;
    FACE_RIGHT :: 11;
    FACE_LEFT :: 12;
    FACE_UP :: 13;
    LEFT_STICK_UP :: 14;
    LEFT_STICK_DOWN :: 15;
    LEFT_STICK_LEFT :: 16;
    LEFT_STICK_RIGHT :: 17;
    RIGHT_STICK_UP :: 18;
    RIGHT_STICK_DOWN :: 19;
    RIGHT_STICK_LEFT :: 20;
    RIGHT_STICK_RIGHT :: 21;
    LEFT_STICK_CLICK :: 22;
    RIGHT_STICK_CLICK :: 23;
    A :: FACE_DOWN;
    B :: FACE_RIGHT;
    X :: FACE_LEFT;
    Y :: FACE_UP;
}

Gamepad :: struct {
    connected: bool;
    buttons: [] Key_Current_State;
    left_stick: Math.Vector2;
    right_stick: Math.Vector2;
    left_trigger: float;
    right_trigger: float;
    generate_events: bool;
    generate_left_stick_events: bool;
    vendor_id: u16 = 0xffff;
    product_id: u16 = 0xffff;
    left_stick_threshold := 0.5;
    right_stick_threshold := 0.5;
    left_trigger_threshold := 0.3;
    right_trigger_threshold := 0.3;
}

gamepad: Gamepad;

#if OS == .LINUX {
    #load "evdev.jai";
}

#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS {
    #load "../internal/gamepad/sdl.jai";
}

#scope_module
Math :: #import "Math";
Input :: #import "Input";

Key_Current_State :: Input.Key_Current_State;

Basic :: #import "Basic";
SDL :: #import "SDL";
