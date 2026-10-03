SDL_PLATFORM_WINDOWS :: "Windows";
SDL_PLATFORM_MACOS :: "Mac OS X";
SDL_PLATFORM_LINUX :: "Linux";
SDL_HAT_UP_BIT :: 0x01;
SDL_HAT_RIGHT_BIT :: 0x02;
SDL_HAT_DOWN_BIT :: 0x04;
SDL_HAT_LEFT_BIT :: 0x08;

SDL_Input_Type :: enum {
    UNINITIALIZED :: 0;
    BUTTON :: 1;
    AXIS :: 2;
    HAT :: 3;
}

SDL_Input_Target :: enum {
    UNINITIALIZED :: 0;
    LEFTX :: 1;
    RIGHTX :: 2;
    LEFTY :: 3;
    RIGHTY :: 4;
    LEFT_TRIGGER :: 5;
    RIGHT_TRIGGER :: 6;
    DPAD_LEFT :: 7;
    DPAD_RIGHT :: 8;
    DPAD_UP :: 9;
    DPAD_DOWN :: 10;
    BACK :: 11;
    GUIDE :: 12;
    START :: 13;
    LEFT_SHOULDER :: 14;
    RIGHT_SHOULDER :: 15;
    LEFT_STICK :: 16;
    RIGHT_STICK :: 17;
    A :: 18;
    B :: 19;
    X :: 20;
    Y :: 21;
}

SDL_Input_Map :: struct {
    target: SDL_Input_Target;
    type: SDL_Input_Type;
    index: s32 = -1;
    subindex: s32 = -1;
}

SDL_Gamepad_Mapping :: struct {
    guid: string;
    name: string;
    platform: string;
    mappings: [..] SDL_Input_Map;
}

#load "../../internal/gamepad/mapping.jai";
#load "database.jai";

#scope_module
#import "Basic";
