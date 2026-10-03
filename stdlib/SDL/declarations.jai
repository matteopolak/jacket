// Generated from docs/specs/stdlib/modules/sdl.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#if OS == .LINUX {
    #import "X11";
}

SDL2 :: #library,system "SDL2";
#scope_export

// Types and layouts

Hack :: struct {

    K :: inline (scancode: SDL_Scancode) -> s32 {
        return cast(s32) scancode | (1 << 30);
    }
}

SDL_AudioCVT :: struct {
    needed: s32;
    src_format: SDL_AudioFormat;
    dst_format: SDL_AudioFormat;
    rate_incr: float64;
    buf: *u8;
    len: s32;
    len_cvt: s32;
    len_mult: s32;
    len_ratio: float64;
    filters: [10] SDL_AudioFilter;
    filter_index: s32;
}

SDL_AudioDeviceEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;

}

SDL_AudioSpec :: struct {
    freq: s32;
    format: SDL_AudioFormat;
    channels: u8;
    silence: u8;
    samples: u16;
    padding: u16;
    size: u32;
    callback: SDL_AudioCallback;
    userdata: *void;
}

using SDL_AudioStatus :: enum u32 {
    SDL_AUDIO_STOPPED :: 0;
    SDL_AUDIO_PLAYING :: 1;
    SDL_AUDIO_PAUSED :: 2;
}

SDL_AudioStream :: struct {}

using SDL_BlendFactor :: enum u32 {
    SDL_BLENDFACTOR_ZERO :: 1;
    SDL_BLENDFACTOR_ONE :: 2;
    SDL_BLENDFACTOR_SRC_COLOR :: 3;
    SDL_BLENDFACTOR_ONE_MINUS_SRC_COLOR :: 4;
    SDL_BLENDFACTOR_SRC_ALPHA :: 5;
    SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA :: 6;
    SDL_BLENDFACTOR_DST_COLOR :: 7;
    SDL_BLENDFACTOR_ONE_MINUS_DST_COLOR :: 8;
    SDL_BLENDFACTOR_DST_ALPHA :: 9;
    SDL_BLENDFACTOR_ONE_MINUS_DST_ALPHA :: 10;
}

using SDL_BlendMode :: enum u32 {
    SDL_BLENDMODE_NONE :: 0;
    SDL_BLENDMODE_BLEND :: 1;
    SDL_BLENDMODE_ADD :: 2;
    SDL_BLENDMODE_MOD :: 4;
    SDL_BLENDMODE_INVALID :: 2147483647;
}

using SDL_BlendOperation :: enum u32 {
    SDL_BLENDOPERATION_ADD :: 1;
    SDL_BLENDOPERATION_SUBTRACT :: 2;
    SDL_BLENDOPERATION_REV_SUBTRACT :: 3;
    SDL_BLENDOPERATION_MINIMUM :: 4;
    SDL_BLENDOPERATION_MAXIMUM :: 5;
}

SDL_BlitInfo :: struct {
    src: *u8;
    src_w, src_h: s32;
    src_pitch: s32;
    src_skip: s32;
    dst: *u8;
    dst_w, dst_h: s32;
    dst_pitch: s32;
    dst_skip: s32;
    src_fmt: *SDL_PixelFormat;
    dst_fmt: *SDL_PixelFormat;
    table: *u8;
    flags: s32;
    colorkey: u32;
    r, g, b, a: u8;
}

SDL_BlitMap :: struct {
    dst: *SDL_Surface;
    identity: s32;
    blit: SDL_blit;
    data: *void;
    info: SDL_BlitInfo;
    dst_palette_version: u32;
    src_palette_version: u32;
}

using SDL_bool :: enum u32 {
    SDL_FALSE :: 0;
    SDL_TRUE :: 1;
}

SDL_Color :: struct {
    r: u8;
    g: u8;
    b: u8;
    a: u8;
}

SDL_CommonEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
}

SDL_ControllerAxisEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: SDL_JoystickID;
    axis: u8;
    padding1: u8;
    padding2: u8;
    padding3: u8;
    value: s16;
    padding4: u16;
}

SDL_ControllerButtonEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: SDL_JoystickID;
    button: u8;
    state: u8;
    padding1: u8;
    padding2: u8;
}

SDL_ControllerDeviceEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: s32;
}

SDL_DisplayMode :: struct {
    format: u32;
    w: s32;
    h: s32;
    refresh_rate: s32;
    driverdata: *void;
}

SDL_DollarGestureEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;

}

SDL_DropEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    file: *u8;
    windowID: u32;
}

SDL_Event :: union {
    type: SDL_EventType;
    common: SDL_CommonEvent;
    window: SDL_WindowEvent;
    key: SDL_KeyboardEvent;
    edit: SDL_TextEditingEvent;
    text: SDL_TextInputEvent;
    motion: SDL_MouseMotionEvent;
    button: SDL_MouseButtonEvent;
    wheel: SDL_MouseWheelEvent;
    jaxis: SDL_JoyAxisEvent;
    jball: SDL_JoyBallEvent;
    jhat: SDL_JoyHatEvent;
    jbutton: SDL_JoyButtonEvent;
    jdevice: SDL_JoyDeviceEvent;
    caxis: SDL_ControllerAxisEvent;
    cbutton: SDL_ControllerButtonEvent;
    cdevice: SDL_ControllerDeviceEvent;
    adevice: SDL_AudioDeviceEvent;
    quit: SDL_QuitEvent;
    user: SDL_UserEvent;
    syswm: SDL_SysWMEvent;
    tfinger: SDL_TouchFingerEvent;
    mgesture: SDL_MultiGestureEvent;
    dgesture: SDL_DollarGestureEvent;
    drop: SDL_DropEvent;

    padding: [56] u8;
}

using SDL_eventaction :: enum u32 {
    SDL_ADDEVENT;
    SDL_PEEKEVENT;
    SDL_GETEVENT;
}

using SDL_EventType :: enum u32 {
    SDL_FIRSTEVENT :: 0;

    SDL_QUIT :: 0x100;

    SDL_APP_TERMINATING;
    SDL_APP_LOWMEMORY;
    SDL_APP_WILLENTERBACKGROUND;
    SDL_APP_DIDENTERBACKGROUND;
    SDL_APP_WILLENTERFOREGROUND;
    SDL_APP_DIDENTERFOREGROUND;

    SDL_WINDOWEVENT :: 0x200;
    SDL_SYSWMEVENT;

    SDL_KEYDOWN :: 0x300;
    SDL_KEYUP;
    SDL_TEXTEDITING;
    SDL_TEXTINPUT;
    SDL_KEYMAPCHANGED;

    SDL_MOUSEMOTION :: 0x400;
    SDL_MOUSEBUTTONDOWN;
    SDL_MOUSEBUTTONUP;
    SDL_MOUSEWHEEL;

    SDL_JOYAXISMOTION :: 0x600;
    SDL_JOYBALLMOTION;
    SDL_JOYHATMOTION;
    SDL_JOYBUTTONDOWN;
    SDL_JOYBUTTONUP;
    SDL_JOYDEVICEADDED;
    SDL_JOYDEVICEREMOVED;

    SDL_CONTROLLERAXISMOTION :: 0x650;
    SDL_CONTROLLERBUTTONDOWN;
    SDL_CONTROLLERBUTTONUP;
    SDL_CONTROLLERDEVICEADDED;
    SDL_CONTROLLERDEVICEREMOVED;
    SDL_CONTROLLERDEVICEREMAPPED;

    SDL_FINGERDOWN :: 0x700;
    SDL_FINGERUP;
    SDL_FINGERMOTION;

    SDL_DOLLARGESTURE :: 0x800;
    SDL_DOLLARRECORD;
    SDL_MULTIGESTURE;

    SDL_CLIPBOARDUPDATE :: 0x900;

    SDL_DROPFILE :: 0x1000;
    SDL_DROPTEXT;
    SDL_DROPBEGIN;
    SDL_DROPCOMPLETE;

    SDL_AUDIODEVICEADDED :: 0x1100;
    SDL_AUDIODEVICEREMOVED;

    SDL_RENDER_TARGETS_RESET :: 0x2000;
    SDL_RENDER_DEVICE_RESET;

    SDL_USEREVENT :: 0x8000;
    SDL_LASTEVENT :: 0xFFFF;
}

SDL_GameController :: struct {}

using SDL_GameControllerAxis :: enum s32 {
    SDL_CONTROLLER_AXIS_INVALID :: -1;
    SDL_CONTROLLER_AXIS_LEFTX :: 0;
    SDL_CONTROLLER_AXIS_LEFTY :: 1;
    SDL_CONTROLLER_AXIS_RIGHTX :: 2;
    SDL_CONTROLLER_AXIS_RIGHTY :: 3;
    SDL_CONTROLLER_AXIS_TRIGGERLEFT :: 4;
    SDL_CONTROLLER_AXIS_TRIGGERRIGHT :: 5;
    SDL_CONTROLLER_AXIS_MAX :: 6;
}

using SDL_GameControllerBindType :: enum u32 {
    SDL_CONTROLLER_BINDTYPE_NONE :: 0;
    SDL_CONTROLLER_BINDTYPE_BUTTON :: 1;
    SDL_CONTROLLER_BINDTYPE_AXIS :: 2;
    SDL_CONTROLLER_BINDTYPE_HAT :: 3;
}

using SDL_GameControllerButton :: enum s32 {
    SDL_CONTROLLER_BUTTON_INVALID :: -1;
    SDL_CONTROLLER_BUTTON_A :: 0;
    SDL_CONTROLLER_BUTTON_B :: 1;
    SDL_CONTROLLER_BUTTON_X :: 2;
    SDL_CONTROLLER_BUTTON_Y :: 3;
    SDL_CONTROLLER_BUTTON_BACK :: 4;
    SDL_CONTROLLER_BUTTON_GUIDE :: 5;
    SDL_CONTROLLER_BUTTON_START :: 6;
    SDL_CONTROLLER_BUTTON_LEFTSTICK :: 7;
    SDL_CONTROLLER_BUTTON_RIGHTSTICK :: 8;
    SDL_CONTROLLER_BUTTON_LEFTSHOULDER :: 9;
    SDL_CONTROLLER_BUTTON_RIGHTSHOULDER :: 10;
    SDL_CONTROLLER_BUTTON_DPAD_UP :: 11;
    SDL_CONTROLLER_BUTTON_DPAD_DOWN :: 12;
    SDL_CONTROLLER_BUTTON_DPAD_LEFT :: 13;
    SDL_CONTROLLER_BUTTON_DPAD_RIGHT :: 14;
    SDL_CONTROLLER_BUTTON_MAX :: 15;
}

SDL_GameControllerButtonBind :: struct {
    bindType: SDL_GameControllerBindType;
    value: union {
        button: s32;
        axis: s32;
        hat: struct {
            hat: s32;
            hat_mask: s32;
        };
    };
}

using SDL_GLattr :: enum u32 {
    SDL_GL_RED_SIZE;
    SDL_GL_GREEN_SIZE;
    SDL_GL_BLUE_SIZE;
    SDL_GL_ALPHA_SIZE;
    SDL_GL_BUFFER_SIZE;
    SDL_GL_DOUBLEBUFFER;
    SDL_GL_DEPTH_SIZE;
    SDL_GL_STENCIL_SIZE;
    SDL_GL_ACCUM_RED_SIZE;
    SDL_GL_ACCUM_GREEN_SIZE;
    SDL_GL_ACCUM_BLUE_SIZE;
    SDL_GL_ACCUM_ALPHA_SIZE;
    SDL_GL_STEREO;
    SDL_GL_MULTISAMPLEBUFFERS;
    SDL_GL_MULTISAMPLESAMPLES;
    SDL_GL_ACCELERATED_VISUAL;
    SDL_GL_RETAINED_BACKING;
    SDL_GL_CONTEXT_MAJOR_VERSION;
    SDL_GL_CONTEXT_MINOR_VERSION;
    SDL_GL_CONTEXT_EGL;
    SDL_GL_CONTEXT_FLAGS;
    SDL_GL_CONTEXT_PROFILE_MASK;
    SDL_GL_SHARE_WITH_CURRENT_CONTEXT;
    SDL_GL_FRAMEBUFFER_SRGB_CAPABLE;
    SDL_GL_CONTEXT_RELEASE_BEHAVIOR;
    SDL_GL_CONTEXT_RESET_NOTIFICATION;
    SDL_GL_CONTEXT_NO_ERROR;
}

using SDL_GLcontextFlag :: enum_flags u32 {
    SDL_GL_CONTEXT_DEBUG_FLAG :: 0x0001;
    SDL_GL_CONTEXT_FORWARD_COMPATIBLE_FLAG :: 0x0002;
    SDL_GL_CONTEXT_ROBUST_ACCESS_FLAG :: 0x0004;
    SDL_GL_CONTEXT_RESET_ISOLATION_FLAG :: 0x0008;
}

using SDL_GLcontextReleaseFlag :: enum_flags u32 {
    SDL_GL_CONTEXT_RELEASE_BEHAVIOR_NONE :: 0x0000;
    SDL_GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH :: 0x0001;
}

using SDL_GLContextResetNotification :: enum_flags u32 {
    SDL_GL_CONTEXT_RESET_NO_NOTIFICATION :: 0x0000;
    SDL_GL_CONTEXT_RESET_LOSE_CONTEXT :: 0x0001;
}

using SDL_GLprofile :: enum_flags u32 {
    SDL_GL_CONTEXT_PROFILE_CORE :: 0x0001;
    SDL_GL_CONTEXT_PROFILE_COMPATIBILITY :: 0x0002;
    SDL_GL_CONTEXT_PROFILE_ES :: 0x0004;
}

using SDL_HintPriority :: enum u32 {
    SDL_HINT_DEFAULT :: 0;
    SDL_HINT_NORMAL :: 1;
    SDL_HINT_OVERRIDE :: 2;
}

using SDL_HitTestResult :: enum u32 {
    SDL_HITTEST_NORMAL;
    SDL_HITTEST_DRAGGABLE;
    SDL_HITTEST_RESIZE_TOPLEFT;
    SDL_HITTEST_RESIZE_TOP;
    SDL_HITTEST_RESIZE_TOPRIGHT;
    SDL_HITTEST_RESIZE_RIGHT;
    SDL_HITTEST_RESIZE_BOTTOMRIGHT;
    SDL_HITTEST_RESIZE_BOTTOM;
    SDL_HITTEST_RESIZE_BOTTOMLEFT;
    SDL_HITTEST_RESIZE_LEFT;
}

using SDL_INIT_FLAGS :: enum_flags u32 {
    SDL_INIT_TIMER :: 0x00000001;
    SDL_INIT_AUDIO :: 0x00000010;
    SDL_INIT_VIDEO :: 0x00000020;
    SDL_INIT_JOYSTICK :: 0x00000200;
    SDL_INIT_HAPTIC :: 0x00001000;
    SDL_INIT_GAMECONTROLLER :: 0x00002000;
    SDL_INIT_EVENTS :: 0x00004000;
    SDL_INIT_NOPARACHUTE :: 0x00100000;
    SDL_INIT_EVERYTHING :: (SDL_INIT_TIMER | SDL_INIT_AUDIO | SDL_INIT_VIDEO | SDL_INIT_EVENTS | SDL_INIT_JOYSTICK | SDL_INIT_HAPTIC | SDL_INIT_GAMECONTROLLER);
}

SDL_JoyAxisEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: SDL_JoystickID;
    axis: u8;
    padding1: u8;
    padding2: u8;
    padding3: u8;
    value: s16;
    padding4: u16;
}

SDL_JoyBallEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: SDL_JoystickID;
    ball: u8;
    padding1: u8;
    padding2: u8;
    padding3: u8;
    xrel: s16;
    yrel: s16;
}

SDL_JoyButtonEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: SDL_JoystickID;
    button: u8;
    state: u8;
    padding1: u8;
    padding2: u8;
}

SDL_JoyDeviceEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: s32;
}

SDL_JoyHatEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    which: SDL_JoystickID;
    hat: u8;
    value: u8;
    padding1: u8;
    padding2: u8;
}

SDL_JoystickGUID :: struct {
    data: [16] u8;
}

SDL_JoystickPowerLevel :: enum s32 {
    SDL_JOYSTICK_POWER_UNKNOWN :: -1;
    SDL_JOYSTICK_POWER_EMPTY;
    SDL_JOYSTICK_POWER_LOW;
    SDL_JOYSTICK_POWER_MEDIUM;
    SDL_JOYSTICK_POWER_FULL;
    SDL_JOYSTICK_POWER_WIRED;
    SDL_JOYSTICK_POWER_MAX;
}

SDL_JoystickType :: enum s32 {
    SDL_JOYSTICK_TYPE_UNKNOWN;
    SDL_JOYSTICK_TYPE_GAMECONTROLLER;
    SDL_JOYSTICK_TYPE_WHEEL;
    SDL_JOYSTICK_TYPE_ARCADE_STICK;
    SDL_JOYSTICK_TYPE_FLIGHT_STICK;
    SDL_JOYSTICK_TYPE_DANCE_PAD;
    SDL_JOYSTICK_TYPE_GUITAR;
    SDL_JOYSTICK_TYPE_DRUM_KIT;
    SDL_JOYSTICK_TYPE_ARCADE_PAD;
    SDL_JOYSTICK_TYPE_THROTTLE;
}

SDL_KeyboardEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    windowID: u32;
    state: u8;
    repeat: u8;
    padding2: u8;
    padding3: u8;
    keysym: SDL_Keysym;
}

using SDL_Keycode :: enum s32 {
    SDLK_UNKNOWN :: 0;

    SDLK_RETURN :: #char "\r";
    SDLK_ESCAPE :: 27;
    SDLK_BACKSPACE :: 8;
    SDLK_TAB :: #char "\t";
    SDLK_SPACE :: #char " ";
    SDLK_EXCLAIM :: #char "!";
    SDLK_QUOTEDBL :: #char "\"";
    SDLK_HASH :: #char "#";
    SDLK_PERCENT :: #char "%";
    SDLK_DOLLAR :: #char "$";
    SDLK_AMPERSAND :: #char "&";
    SDLK_QUOTE :: #char "'";
    SDLK_LEFTPAREN :: #char "(";
    SDLK_RIGHTPAREN :: #char ")";
    SDLK_ASTERISK :: #char "*";
    SDLK_PLUS :: #char "+";
    SDLK_COMMA :: #char ",";
    SDLK_MINUS :: #char "-";
    SDLK_PERIOD :: #char ".";
    SDLK_SLASH :: #char "/";
    SDLK_0 :: #char "0";
    SDLK_1 :: #char "1";
    SDLK_2 :: #char "2";
    SDLK_3 :: #char "3";
    SDLK_4 :: #char "4";
    SDLK_5 :: #char "5";
    SDLK_6 :: #char "6";
    SDLK_7 :: #char "7";
    SDLK_8 :: #char "8";
    SDLK_9 :: #char "9";
    SDLK_COLON :: #char ":";
    SDLK_SEMICOLON :: #char ";";
    SDLK_LESS :: #char "<";
    SDLK_EQUALS :: #char "=";
    SDLK_GREATER :: #char ">";
    SDLK_QUESTION :: #char "?";
    SDLK_AT :: #char "@";

    SDLK_LEFTBRACKET :: #char "[";
    SDLK_BACKSLASH :: #char "\\";
    SDLK_RIGHTBRACKET :: #char "]";
    SDLK_CARET :: #char "^";
    SDLK_UNDERSCORE :: #char "_";
    SDLK_BACKQUOTE :: #char "`";
    SDLK_a :: #char "a";
    SDLK_b :: #char "b";
    SDLK_c :: #char "c";
    SDLK_d :: #char "d";
    SDLK_e :: #char "e";
    SDLK_f :: #char "f";
    SDLK_g :: #char "g";
    SDLK_h :: #char "h";
    SDLK_i :: #char "i";
    SDLK_j :: #char "j";
    SDLK_k :: #char "k";
    SDLK_l :: #char "l";
    SDLK_m :: #char "m";
    SDLK_n :: #char "n";
    SDLK_o :: #char "o";
    SDLK_p :: #char "p";
    SDLK_q :: #char "q";
    SDLK_r :: #char "r";
    SDLK_s :: #char "s";
    SDLK_t :: #char "t";
    SDLK_u :: #char "u";
    SDLK_v :: #char "v";
    SDLK_w :: #char "w";
    SDLK_x :: #char "x";
    SDLK_y :: #char "y";
    SDLK_z :: #char "z";

    SDLK_SCANCODE_MASK :: 1 << 30;

    SDLK_CAPSLOCK :: #run Hack.K(SDL_SCANCODE_CAPSLOCK);

    SDLK_F1 :: #run Hack.K(SDL_SCANCODE_F1);
    SDLK_F2 :: #run Hack.K(SDL_SCANCODE_F2);
    SDLK_F3 :: #run Hack.K(SDL_SCANCODE_F3);
    SDLK_F4 :: #run Hack.K(SDL_SCANCODE_F4);
    SDLK_F5 :: #run Hack.K(SDL_SCANCODE_F5);
    SDLK_F6 :: #run Hack.K(SDL_SCANCODE_F6);
    SDLK_F7 :: #run Hack.K(SDL_SCANCODE_F7);
    SDLK_F8 :: #run Hack.K(SDL_SCANCODE_F8);
    SDLK_F9 :: #run Hack.K(SDL_SCANCODE_F9);
    SDLK_F10 :: #run Hack.K(SDL_SCANCODE_F10);
    SDLK_F11 :: #run Hack.K(SDL_SCANCODE_F11);
    SDLK_F12 :: #run Hack.K(SDL_SCANCODE_F12);

    SDLK_PRINTSCREEN :: #run Hack.K(SDL_SCANCODE_PRINTSCREEN);
    SDLK_SCROLLLOCK :: #run Hack.K(SDL_SCANCODE_SCROLLLOCK);
    SDLK_PAUSE :: #run Hack.K(SDL_SCANCODE_PAUSE);
    SDLK_INSERT :: #run Hack.K(SDL_SCANCODE_INSERT);
    SDLK_HOME :: #run Hack.K(SDL_SCANCODE_HOME);
    SDLK_PAGEUP :: #run Hack.K(SDL_SCANCODE_PAGEUP);
    SDLK_DELETE :: 127;
    SDLK_END :: #run Hack.K(SDL_SCANCODE_END);
    SDLK_PAGEDOWN :: #run Hack.K(SDL_SCANCODE_PAGEDOWN);
    SDLK_RIGHT :: #run Hack.K(SDL_SCANCODE_RIGHT);
    SDLK_LEFT :: #run Hack.K(SDL_SCANCODE_LEFT);
    SDLK_DOWN :: #run Hack.K(SDL_SCANCODE_DOWN);
    SDLK_UP :: #run Hack.K(SDL_SCANCODE_UP);

    SDLK_NUMLOCKCLEAR :: #run Hack.K(SDL_SCANCODE_NUMLOCKCLEAR);
    SDLK_KP_DIVIDE :: #run Hack.K(SDL_SCANCODE_KP_DIVIDE);
    SDLK_KP_MULTIPLY :: #run Hack.K(SDL_SCANCODE_KP_MULTIPLY);
    SDLK_KP_MINUS :: #run Hack.K(SDL_SCANCODE_KP_MINUS);
    SDLK_KP_PLUS :: #run Hack.K(SDL_SCANCODE_KP_PLUS);
    SDLK_KP_ENTER :: #run Hack.K(SDL_SCANCODE_KP_ENTER);
    SDLK_KP_1 :: #run Hack.K(SDL_SCANCODE_KP_1);
    SDLK_KP_2 :: #run Hack.K(SDL_SCANCODE_KP_2);
    SDLK_KP_3 :: #run Hack.K(SDL_SCANCODE_KP_3);
    SDLK_KP_4 :: #run Hack.K(SDL_SCANCODE_KP_4);
    SDLK_KP_5 :: #run Hack.K(SDL_SCANCODE_KP_5);
    SDLK_KP_6 :: #run Hack.K(SDL_SCANCODE_KP_6);
    SDLK_KP_7 :: #run Hack.K(SDL_SCANCODE_KP_7);
    SDLK_KP_8 :: #run Hack.K(SDL_SCANCODE_KP_8);
    SDLK_KP_9 :: #run Hack.K(SDL_SCANCODE_KP_9);
    SDLK_KP_0 :: #run Hack.K(SDL_SCANCODE_KP_0);
    SDLK_KP_PERIOD :: #run Hack.K(SDL_SCANCODE_KP_PERIOD);

    SDLK_APPLICATION :: #run Hack.K(SDL_SCANCODE_APPLICATION);
    SDLK_POWER :: #run Hack.K(SDL_SCANCODE_POWER);
    SDLK_KP_EQUALS :: #run Hack.K(SDL_SCANCODE_KP_EQUALS);
    SDLK_F13 :: #run Hack.K(SDL_SCANCODE_F13);
    SDLK_F14 :: #run Hack.K(SDL_SCANCODE_F14);
    SDLK_F15 :: #run Hack.K(SDL_SCANCODE_F15);
    SDLK_F16 :: #run Hack.K(SDL_SCANCODE_F16);
    SDLK_F17 :: #run Hack.K(SDL_SCANCODE_F17);
    SDLK_F18 :: #run Hack.K(SDL_SCANCODE_F18);
    SDLK_F19 :: #run Hack.K(SDL_SCANCODE_F19);
    SDLK_F20 :: #run Hack.K(SDL_SCANCODE_F20);
    SDLK_F21 :: #run Hack.K(SDL_SCANCODE_F21);
    SDLK_F22 :: #run Hack.K(SDL_SCANCODE_F22);
    SDLK_F23 :: #run Hack.K(SDL_SCANCODE_F23);
    SDLK_F24 :: #run Hack.K(SDL_SCANCODE_F24);
    SDLK_EXECUTE :: #run Hack.K(SDL_SCANCODE_EXECUTE);
    SDLK_HELP :: #run Hack.K(SDL_SCANCODE_HELP);
    SDLK_MENU :: #run Hack.K(SDL_SCANCODE_MENU);
    SDLK_SELECT :: #run Hack.K(SDL_SCANCODE_SELECT);
    SDLK_STOP :: #run Hack.K(SDL_SCANCODE_STOP);
    SDLK_AGAIN :: #run Hack.K(SDL_SCANCODE_AGAIN);
    SDLK_UNDO :: #run Hack.K(SDL_SCANCODE_UNDO);
    SDLK_CUT :: #run Hack.K(SDL_SCANCODE_CUT);
    SDLK_COPY :: #run Hack.K(SDL_SCANCODE_COPY);
    SDLK_PASTE :: #run Hack.K(SDL_SCANCODE_PASTE);
    SDLK_FIND :: #run Hack.K(SDL_SCANCODE_FIND);
    SDLK_MUTE :: #run Hack.K(SDL_SCANCODE_MUTE);
    SDLK_VOLUMEUP :: #run Hack.K(SDL_SCANCODE_VOLUMEUP);
    SDLK_VOLUMEDOWN :: #run Hack.K(SDL_SCANCODE_VOLUMEDOWN);
    SDLK_KP_COMMA :: #run Hack.K(SDL_SCANCODE_KP_COMMA);
    SDLK_KP_EQUALSAS400 :: #run Hack.K(SDL_SCANCODE_KP_EQUALSAS400);

    SDLK_ALTERASE :: #run Hack.K(SDL_SCANCODE_ALTERASE);
    SDLK_SYSREQ :: #run Hack.K(SDL_SCANCODE_SYSREQ);
    SDLK_CANCEL :: #run Hack.K(SDL_SCANCODE_CANCEL);
    SDLK_CLEAR :: #run Hack.K(SDL_SCANCODE_CLEAR);
    SDLK_PRIOR :: #run Hack.K(SDL_SCANCODE_PRIOR);
    SDLK_RETURN2 :: #run Hack.K(SDL_SCANCODE_RETURN2);
    SDLK_SEPARATOR :: #run Hack.K(SDL_SCANCODE_SEPARATOR);
    SDLK_OUT :: #run Hack.K(SDL_SCANCODE_OUT);
    SDLK_OPER :: #run Hack.K(SDL_SCANCODE_OPER);
    SDLK_CLEARAGAIN :: #run Hack.K(SDL_SCANCODE_CLEARAGAIN);
    SDLK_CRSEL :: #run Hack.K(SDL_SCANCODE_CRSEL);
    SDLK_EXSEL :: #run Hack.K(SDL_SCANCODE_EXSEL);

    SDLK_KP_00 :: #run Hack.K(SDL_SCANCODE_KP_00);
    SDLK_KP_000 :: #run Hack.K(SDL_SCANCODE_KP_000);
    SDLK_THOUSANDSSEPARATOR :: #run Hack.K(SDL_SCANCODE_THOUSANDSSEPARATOR);
    SDLK_DECIMALSEPARATOR :: #run Hack.K(SDL_SCANCODE_DECIMALSEPARATOR);
    SDLK_CURRENCYUNIT :: #run Hack.K(SDL_SCANCODE_CURRENCYUNIT);
    SDLK_CURRENCYSUBUNIT :: #run Hack.K(SDL_SCANCODE_CURRENCYSUBUNIT);
    SDLK_KP_LEFTPAREN :: #run Hack.K(SDL_SCANCODE_KP_LEFTPAREN);
    SDLK_KP_RIGHTPAREN :: #run Hack.K(SDL_SCANCODE_KP_RIGHTPAREN);
    SDLK_KP_LEFTBRACE :: #run Hack.K(SDL_SCANCODE_KP_LEFTBRACE);
    SDLK_KP_RIGHTBRACE :: #run Hack.K(SDL_SCANCODE_KP_RIGHTBRACE);
    SDLK_KP_TAB :: #run Hack.K(SDL_SCANCODE_KP_TAB);
    SDLK_KP_BACKSPACE :: #run Hack.K(SDL_SCANCODE_KP_BACKSPACE);
    SDLK_KP_A :: #run Hack.K(SDL_SCANCODE_KP_A);
    SDLK_KP_B :: #run Hack.K(SDL_SCANCODE_KP_B);
    SDLK_KP_C :: #run Hack.K(SDL_SCANCODE_KP_C);
    SDLK_KP_D :: #run Hack.K(SDL_SCANCODE_KP_D);
    SDLK_KP_E :: #run Hack.K(SDL_SCANCODE_KP_E);
    SDLK_KP_F :: #run Hack.K(SDL_SCANCODE_KP_F);
    SDLK_KP_XOR :: #run Hack.K(SDL_SCANCODE_KP_XOR);
    SDLK_KP_POWER :: #run Hack.K(SDL_SCANCODE_KP_POWER);
    SDLK_KP_PERCENT :: #run Hack.K(SDL_SCANCODE_KP_PERCENT);
    SDLK_KP_LESS :: #run Hack.K(SDL_SCANCODE_KP_LESS);
    SDLK_KP_GREATER :: #run Hack.K(SDL_SCANCODE_KP_GREATER);
    SDLK_KP_AMPERSAND :: #run Hack.K(SDL_SCANCODE_KP_AMPERSAND);
    SDLK_KP_DBLAMPERSAND :: #run Hack.K(SDL_SCANCODE_KP_DBLAMPERSAND);
    SDLK_KP_VERTICALBAR :: #run Hack.K(SDL_SCANCODE_KP_VERTICALBAR);
    SDLK_KP_DBLVERTICALBAR :: #run Hack.K(SDL_SCANCODE_KP_DBLVERTICALBAR);
    SDLK_KP_COLON :: #run Hack.K(SDL_SCANCODE_KP_COLON);
    SDLK_KP_HASH :: #run Hack.K(SDL_SCANCODE_KP_HASH);
    SDLK_KP_SPACE :: #run Hack.K(SDL_SCANCODE_KP_SPACE);
    SDLK_KP_AT :: #run Hack.K(SDL_SCANCODE_KP_AT);
    SDLK_KP_EXCLAM :: #run Hack.K(SDL_SCANCODE_KP_EXCLAM);
    SDLK_KP_MEMSTORE :: #run Hack.K(SDL_SCANCODE_KP_MEMSTORE);
    SDLK_KP_MEMRECALL :: #run Hack.K(SDL_SCANCODE_KP_MEMRECALL);
    SDLK_KP_MEMCLEAR :: #run Hack.K(SDL_SCANCODE_KP_MEMCLEAR);
    SDLK_KP_MEMADD :: #run Hack.K(SDL_SCANCODE_KP_MEMADD);
    SDLK_KP_MEMSUBTRACT :: #run Hack.K(SDL_SCANCODE_KP_MEMSUBTRACT);
    SDLK_KP_MEMMULTIPLY :: #run Hack.K(SDL_SCANCODE_KP_MEMMULTIPLY);
    SDLK_KP_MEMDIVIDE :: #run Hack.K(SDL_SCANCODE_KP_MEMDIVIDE);
    SDLK_KP_PLUSMINUS :: #run Hack.K(SDL_SCANCODE_KP_PLUSMINUS);
    SDLK_KP_CLEAR :: #run Hack.K(SDL_SCANCODE_KP_CLEAR);
    SDLK_KP_CLEARENTRY :: #run Hack.K(SDL_SCANCODE_KP_CLEARENTRY);
    SDLK_KP_BINARY :: #run Hack.K(SDL_SCANCODE_KP_BINARY);
    SDLK_KP_OCTAL :: #run Hack.K(SDL_SCANCODE_KP_OCTAL);
    SDLK_KP_DECIMAL :: #run Hack.K(SDL_SCANCODE_KP_DECIMAL);
    SDLK_KP_HEXADECIMAL :: #run Hack.K(SDL_SCANCODE_KP_HEXADECIMAL);

    SDLK_LCTRL :: #run Hack.K(SDL_SCANCODE_LCTRL);
    SDLK_LSHIFT :: #run Hack.K(SDL_SCANCODE_LSHIFT);
    SDLK_LALT :: #run Hack.K(SDL_SCANCODE_LALT);
    SDLK_LGUI :: #run Hack.K(SDL_SCANCODE_LGUI);
    SDLK_RCTRL :: #run Hack.K(SDL_SCANCODE_RCTRL);
    SDLK_RSHIFT :: #run Hack.K(SDL_SCANCODE_RSHIFT);
    SDLK_RALT :: #run Hack.K(SDL_SCANCODE_RALT);
    SDLK_RGUI :: #run Hack.K(SDL_SCANCODE_RGUI);

    SDLK_MODE :: #run Hack.K(SDL_SCANCODE_MODE);

    SDLK_AUDIONEXT :: #run Hack.K(SDL_SCANCODE_AUDIONEXT);
    SDLK_AUDIOPREV :: #run Hack.K(SDL_SCANCODE_AUDIOPREV);
    SDLK_AUDIOSTOP :: #run Hack.K(SDL_SCANCODE_AUDIOSTOP);
    SDLK_AUDIOPLAY :: #run Hack.K(SDL_SCANCODE_AUDIOPLAY);
    SDLK_AUDIOMUTE :: #run Hack.K(SDL_SCANCODE_AUDIOMUTE);
    SDLK_MEDIASELECT :: #run Hack.K(SDL_SCANCODE_MEDIASELECT);
    SDLK_WWW :: #run Hack.K(SDL_SCANCODE_WWW);
    SDLK_MAIL :: #run Hack.K(SDL_SCANCODE_MAIL);
    SDLK_CALCULATOR :: #run Hack.K(SDL_SCANCODE_CALCULATOR);
    SDLK_COMPUTER :: #run Hack.K(SDL_SCANCODE_COMPUTER);
    SDLK_AC_SEARCH :: #run Hack.K(SDL_SCANCODE_AC_SEARCH);
    SDLK_AC_HOME :: #run Hack.K(SDL_SCANCODE_AC_HOME);
    SDLK_AC_BACK :: #run Hack.K(SDL_SCANCODE_AC_BACK);
    SDLK_AC_FORWARD :: #run Hack.K(SDL_SCANCODE_AC_FORWARD);
    SDLK_AC_STOP :: #run Hack.K(SDL_SCANCODE_AC_STOP);
    SDLK_AC_REFRESH :: #run Hack.K(SDL_SCANCODE_AC_REFRESH);
    SDLK_AC_BOOKMARKS :: #run Hack.K(SDL_SCANCODE_AC_BOOKMARKS);

    SDLK_BRIGHTNESSDOWN :: #run Hack.K(SDL_SCANCODE_BRIGHTNESSDOWN);
    SDLK_BRIGHTNESSUP :: #run Hack.K(SDL_SCANCODE_BRIGHTNESSUP);
    SDLK_DISPLAYSWITCH :: #run Hack.K(SDL_SCANCODE_DISPLAYSWITCH);
    SDLK_KBDILLUMTOGGLE :: #run Hack.K(SDL_SCANCODE_KBDILLUMTOGGLE);
    SDLK_KBDILLUMDOWN :: #run Hack.K(SDL_SCANCODE_KBDILLUMDOWN);
    SDLK_KBDILLUMUP :: #run Hack.K(SDL_SCANCODE_KBDILLUMUP);
    SDLK_EJECT :: #run Hack.K(SDL_SCANCODE_EJECT);
    SDLK_SLEEP :: #run Hack.K(SDL_SCANCODE_SLEEP);
    SDLK_APP1 :: #run Hack.K(SDL_SCANCODE_APP1);
    SDLK_APP2 :: #run Hack.K(SDL_SCANCODE_APP2);

}

using SDL_Keymod :: enum u16 {
    KMOD_NONE :: 0x0000;
    KMOD_LSHIFT :: 0x0001;
    KMOD_RSHIFT :: 0x0002;
    KMOD_SHIFT :: KMOD_LSHIFT | KMOD_RSHIFT;
    KMOD_LCTRL :: 0x0040;
    KMOD_RCTRL :: 0x0080;
    KMOD_CTRL :: KMOD_LCTRL | KMOD_RCTRL;
    KMOD_LALT :: 0x0100;
    KMOD_RALT :: 0x0200;
    KMOD_ALT :: KMOD_LALT | KMOD_RALT;
    KMOD_LGUI :: 0x0400;
    KMOD_RGUI :: 0x0800;
    KMOD_GUI :: KMOD_LGUI | KMOD_RGUI;
    KMOD_NUM :: 0x1000;
    KMOD_CAPS :: 0x2000;
    KMOD_MODE :: 0x4000;
    KMOD_RESERVED :: 0x8000;
}

SDL_Keysym :: struct {
    scancode: SDL_Scancode;
    sym: SDL_Keycode;
    mod: SDL_Keymod;
    unused: u16;
}

SDL_MouseButtonEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    windowID: u32;
    which: u32;
    button: u8;
    state: u8;
    clicks: u8;
    padding1: u8;
    x: s32;
    y: s32;
}

SDL_MouseMotionEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    windowID: u32;
    which: u32;
    state: u32;
    x: s32;
    y: s32;
    xrel: s32;
    yrel: s32;
}

SDL_MouseWheelDirection :: enum s32 {
    SDL_MOUSEWHEEL_NORMAL;
    SDL_MOUSEWHEEL_FLIPPED;
}

SDL_MouseWheelEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    windowID: u32;
    which: u32;
    x: s32;
    y: s32;
    direction: u32;
}

SDL_MultiGestureEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;

}

SDL_Palette :: struct {
    ncolors: s32;
    colors: *SDL_Color;
    version: u32;
    refcount: s32;
}

SDL_PixelFormat :: struct {
    format: u32;
    palette: *SDL_Palette;
    BitsPerPixel: u8;
    BytesPerPixel: u8;
    padding: [2] u8;
    Rmask: u32;
    Gmask: u32;
    Bmask: u32;
    Amask: u32;
    Rloss: u8;
    Gloss: u8;
    Bloss: u8;
    Aloss: u8;
    Rshift: u8;
    Gshift: u8;
    Bshift: u8;
    Ashift: u8;
    refcount: s32;
    next: *SDL_PixelFormat;
}

SDL_Point :: struct {
    x: s32;
    y: s32;
}

SDL_QuitEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;

}

SDL_Rect :: struct {
    x, y: s32;
    w, h: s32;
}

SDL_Renderer :: struct {}

SDL_RendererFlags :: enum_flags u32 {
    SOFTWARE :: 1;
    ACCELERATED :: 2;
    PRESENTVSYNC :: 4;
    TARGETTEXTURE :: 8;
}

SDL_RendererFlip :: enum u32 {
    NONE :: 0;
    HORIZONTAL :: 1;
    VERTICAL :: 2;
}

SDL_RendererInfo :: struct {
    name: *s8;
    flags: u32;
    num_texture_formats: u32;
    texture_formats: [16] u32;
    max_texture_width: s32;
    max_texture_height: s32;
}

SDL_RWops :: struct {
    size: (*SDL_RWops) -> s64;
    seek: (*SDL_RWops, s64, int) -> s64;
    read: (*SDL_RWops, *void, int, int) -> int;
    write: (*SDL_RWops, *void, int, int) -> int;
    close: (*SDL_RWops) -> int;
    type: u32;
}

using SDL_Scancode :: enum s32 {
    SDL_SCANCODE_UNKNOWN :: 0;

    SDL_SCANCODE_A :: 4;
    SDL_SCANCODE_B :: 5;
    SDL_SCANCODE_C :: 6;
    SDL_SCANCODE_D :: 7;
    SDL_SCANCODE_E :: 8;
    SDL_SCANCODE_F :: 9;
    SDL_SCANCODE_G :: 10;
    SDL_SCANCODE_H :: 11;
    SDL_SCANCODE_I :: 12;
    SDL_SCANCODE_J :: 13;
    SDL_SCANCODE_K :: 14;
    SDL_SCANCODE_L :: 15;
    SDL_SCANCODE_M :: 16;
    SDL_SCANCODE_N :: 17;
    SDL_SCANCODE_O :: 18;
    SDL_SCANCODE_P :: 19;
    SDL_SCANCODE_Q :: 20;
    SDL_SCANCODE_R :: 21;
    SDL_SCANCODE_S :: 22;
    SDL_SCANCODE_T :: 23;
    SDL_SCANCODE_U :: 24;
    SDL_SCANCODE_V :: 25;
    SDL_SCANCODE_W :: 26;
    SDL_SCANCODE_X :: 27;
    SDL_SCANCODE_Y :: 28;
    SDL_SCANCODE_Z :: 29;

    SDL_SCANCODE_1 :: 30;
    SDL_SCANCODE_2 :: 31;
    SDL_SCANCODE_3 :: 32;
    SDL_SCANCODE_4 :: 33;
    SDL_SCANCODE_5 :: 34;
    SDL_SCANCODE_6 :: 35;
    SDL_SCANCODE_7 :: 36;
    SDL_SCANCODE_8 :: 37;
    SDL_SCANCODE_9 :: 38;
    SDL_SCANCODE_0 :: 39;

    SDL_SCANCODE_RETURN :: 40;
    SDL_SCANCODE_ESCAPE :: 41;
    SDL_SCANCODE_BACKSPACE :: 42;
    SDL_SCANCODE_TAB :: 43;
    SDL_SCANCODE_SPACE :: 44;

    SDL_SCANCODE_MINUS :: 45;
    SDL_SCANCODE_EQUALS :: 46;
    SDL_SCANCODE_LEFTBRACKET :: 47;
    SDL_SCANCODE_RIGHTBRACKET :: 48;
    SDL_SCANCODE_BACKSLASH :: 49;
    SDL_SCANCODE_NONUSHASH :: 50;
    SDL_SCANCODE_SEMICOLON :: 51;
    SDL_SCANCODE_APOSTROPHE :: 52;
    SDL_SCANCODE_GRAVE :: 53;
    SDL_SCANCODE_COMMA :: 54;
    SDL_SCANCODE_PERIOD :: 55;
    SDL_SCANCODE_SLASH :: 56;

    SDL_SCANCODE_CAPSLOCK :: 57;

    SDL_SCANCODE_F1 :: 58;
    SDL_SCANCODE_F2 :: 59;
    SDL_SCANCODE_F3 :: 60;
    SDL_SCANCODE_F4 :: 61;
    SDL_SCANCODE_F5 :: 62;
    SDL_SCANCODE_F6 :: 63;
    SDL_SCANCODE_F7 :: 64;
    SDL_SCANCODE_F8 :: 65;
    SDL_SCANCODE_F9 :: 66;
    SDL_SCANCODE_F10 :: 67;
    SDL_SCANCODE_F11 :: 68;
    SDL_SCANCODE_F12 :: 69;

    SDL_SCANCODE_PRINTSCREEN :: 70;
    SDL_SCANCODE_SCROLLLOCK :: 71;
    SDL_SCANCODE_PAUSE :: 72;
    SDL_SCANCODE_INSERT :: 73;
    SDL_SCANCODE_HOME :: 74;
    SDL_SCANCODE_PAGEUP :: 75;
    SDL_SCANCODE_DELETE :: 76;
    SDL_SCANCODE_END :: 77;
    SDL_SCANCODE_PAGEDOWN :: 78;
    SDL_SCANCODE_RIGHT :: 79;
    SDL_SCANCODE_LEFT :: 80;
    SDL_SCANCODE_DOWN :: 81;
    SDL_SCANCODE_UP :: 82;

    SDL_SCANCODE_NUMLOCKCLEAR :: 83;
    SDL_SCANCODE_KP_DIVIDE :: 84;
    SDL_SCANCODE_KP_MULTIPLY :: 85;
    SDL_SCANCODE_KP_MINUS :: 86;
    SDL_SCANCODE_KP_PLUS :: 87;
    SDL_SCANCODE_KP_ENTER :: 88;
    SDL_SCANCODE_KP_1 :: 89;
    SDL_SCANCODE_KP_2 :: 90;
    SDL_SCANCODE_KP_3 :: 91;
    SDL_SCANCODE_KP_4 :: 92;
    SDL_SCANCODE_KP_5 :: 93;
    SDL_SCANCODE_KP_6 :: 94;
    SDL_SCANCODE_KP_7 :: 95;
    SDL_SCANCODE_KP_8 :: 96;
    SDL_SCANCODE_KP_9 :: 97;
    SDL_SCANCODE_KP_0 :: 98;
    SDL_SCANCODE_KP_PERIOD :: 99;

    SDL_SCANCODE_NONUSBACKSLASH :: 100;
    SDL_SCANCODE_APPLICATION :: 101;
    SDL_SCANCODE_POWER :: 102;
    SDL_SCANCODE_KP_EQUALS :: 103;
    SDL_SCANCODE_F13 :: 104;
    SDL_SCANCODE_F14 :: 105;
    SDL_SCANCODE_F15 :: 106;
    SDL_SCANCODE_F16 :: 107;
    SDL_SCANCODE_F17 :: 108;
    SDL_SCANCODE_F18 :: 109;
    SDL_SCANCODE_F19 :: 110;
    SDL_SCANCODE_F20 :: 111;
    SDL_SCANCODE_F21 :: 112;
    SDL_SCANCODE_F22 :: 113;
    SDL_SCANCODE_F23 :: 114;
    SDL_SCANCODE_F24 :: 115;
    SDL_SCANCODE_EXECUTE :: 116;
    SDL_SCANCODE_HELP :: 117;
    SDL_SCANCODE_MENU :: 118;
    SDL_SCANCODE_SELECT :: 119;
    SDL_SCANCODE_STOP :: 120;
    SDL_SCANCODE_AGAIN :: 121;
    SDL_SCANCODE_UNDO :: 122;
    SDL_SCANCODE_CUT :: 123;
    SDL_SCANCODE_COPY :: 124;
    SDL_SCANCODE_PASTE :: 125;
    SDL_SCANCODE_FIND :: 126;
    SDL_SCANCODE_MUTE :: 127;
    SDL_SCANCODE_VOLUMEUP :: 128;
    SDL_SCANCODE_VOLUMEDOWN :: 129;
    SDL_SCANCODE_KP_COMMA :: 133;
    SDL_SCANCODE_KP_EQUALSAS400 :: 134;

    SDL_SCANCODE_INTERNATIONAL1 :: 135;
    SDL_SCANCODE_INTERNATIONAL2 :: 136;
    SDL_SCANCODE_INTERNATIONAL3 :: 137;
    SDL_SCANCODE_INTERNATIONAL4 :: 138;
    SDL_SCANCODE_INTERNATIONAL5 :: 139;
    SDL_SCANCODE_INTERNATIONAL6 :: 140;
    SDL_SCANCODE_INTERNATIONAL7 :: 141;
    SDL_SCANCODE_INTERNATIONAL8 :: 142;
    SDL_SCANCODE_INTERNATIONAL9 :: 143;
    SDL_SCANCODE_LANG1 :: 144;
    SDL_SCANCODE_LANG2 :: 145;
    SDL_SCANCODE_LANG3 :: 146;
    SDL_SCANCODE_LANG4 :: 147;
    SDL_SCANCODE_LANG5 :: 148;
    SDL_SCANCODE_LANG6 :: 149;
    SDL_SCANCODE_LANG7 :: 150;
    SDL_SCANCODE_LANG8 :: 151;
    SDL_SCANCODE_LANG9 :: 152;

    SDL_SCANCODE_ALTERASE :: 153;
    SDL_SCANCODE_SYSREQ :: 154;
    SDL_SCANCODE_CANCEL :: 155;
    SDL_SCANCODE_CLEAR :: 156;
    SDL_SCANCODE_PRIOR :: 157;
    SDL_SCANCODE_RETURN2 :: 158;
    SDL_SCANCODE_SEPARATOR :: 159;
    SDL_SCANCODE_OUT :: 160;
    SDL_SCANCODE_OPER :: 161;
    SDL_SCANCODE_CLEARAGAIN :: 162;
    SDL_SCANCODE_CRSEL :: 163;
    SDL_SCANCODE_EXSEL :: 164;

    SDL_SCANCODE_KP_00 :: 176;
    SDL_SCANCODE_KP_000 :: 177;
    SDL_SCANCODE_THOUSANDSSEPARATOR :: 178;
    SDL_SCANCODE_DECIMALSEPARATOR :: 179;
    SDL_SCANCODE_CURRENCYUNIT :: 180;
    SDL_SCANCODE_CURRENCYSUBUNIT :: 181;
    SDL_SCANCODE_KP_LEFTPAREN :: 182;
    SDL_SCANCODE_KP_RIGHTPAREN :: 183;
    SDL_SCANCODE_KP_LEFTBRACE :: 184;
    SDL_SCANCODE_KP_RIGHTBRACE :: 185;
    SDL_SCANCODE_KP_TAB :: 186;
    SDL_SCANCODE_KP_BACKSPACE :: 187;
    SDL_SCANCODE_KP_A :: 188;
    SDL_SCANCODE_KP_B :: 189;
    SDL_SCANCODE_KP_C :: 190;
    SDL_SCANCODE_KP_D :: 191;
    SDL_SCANCODE_KP_E :: 192;
    SDL_SCANCODE_KP_F :: 193;
    SDL_SCANCODE_KP_XOR :: 194;
    SDL_SCANCODE_KP_POWER :: 195;
    SDL_SCANCODE_KP_PERCENT :: 196;
    SDL_SCANCODE_KP_LESS :: 197;
    SDL_SCANCODE_KP_GREATER :: 198;
    SDL_SCANCODE_KP_AMPERSAND :: 199;
    SDL_SCANCODE_KP_DBLAMPERSAND :: 200;
    SDL_SCANCODE_KP_VERTICALBAR :: 201;
    SDL_SCANCODE_KP_DBLVERTICALBAR :: 202;
    SDL_SCANCODE_KP_COLON :: 203;
    SDL_SCANCODE_KP_HASH :: 204;
    SDL_SCANCODE_KP_SPACE :: 205;
    SDL_SCANCODE_KP_AT :: 206;
    SDL_SCANCODE_KP_EXCLAM :: 207;
    SDL_SCANCODE_KP_MEMSTORE :: 208;
    SDL_SCANCODE_KP_MEMRECALL :: 209;
    SDL_SCANCODE_KP_MEMCLEAR :: 210;
    SDL_SCANCODE_KP_MEMADD :: 211;
    SDL_SCANCODE_KP_MEMSUBTRACT :: 212;
    SDL_SCANCODE_KP_MEMMULTIPLY :: 213;
    SDL_SCANCODE_KP_MEMDIVIDE :: 214;
    SDL_SCANCODE_KP_PLUSMINUS :: 215;
    SDL_SCANCODE_KP_CLEAR :: 216;
    SDL_SCANCODE_KP_CLEARENTRY :: 217;
    SDL_SCANCODE_KP_BINARY :: 218;
    SDL_SCANCODE_KP_OCTAL :: 219;
    SDL_SCANCODE_KP_DECIMAL :: 220;
    SDL_SCANCODE_KP_HEXADECIMAL :: 221;

    SDL_SCANCODE_LCTRL :: 224;
    SDL_SCANCODE_LSHIFT :: 225;
    SDL_SCANCODE_LALT :: 226;
    SDL_SCANCODE_LGUI :: 227;
    SDL_SCANCODE_RCTRL :: 228;
    SDL_SCANCODE_RSHIFT :: 229;
    SDL_SCANCODE_RALT :: 230;
    SDL_SCANCODE_RGUI :: 231;

    SDL_SCANCODE_MODE :: 257;

    SDL_SCANCODE_AUDIONEXT :: 258;
    SDL_SCANCODE_AUDIOPREV :: 259;
    SDL_SCANCODE_AUDIOSTOP :: 260;
    SDL_SCANCODE_AUDIOPLAY :: 261;
    SDL_SCANCODE_AUDIOMUTE :: 262;
    SDL_SCANCODE_MEDIASELECT :: 263;
    SDL_SCANCODE_WWW :: 264;
    SDL_SCANCODE_MAIL :: 265;
    SDL_SCANCODE_CALCULATOR :: 266;
    SDL_SCANCODE_COMPUTER :: 267;
    SDL_SCANCODE_AC_SEARCH :: 268;
    SDL_SCANCODE_AC_HOME :: 269;
    SDL_SCANCODE_AC_BACK :: 270;
    SDL_SCANCODE_AC_FORWARD :: 271;
    SDL_SCANCODE_AC_STOP :: 272;
    SDL_SCANCODE_AC_REFRESH :: 273;
    SDL_SCANCODE_AC_BOOKMARKS :: 274;

    SDL_SCANCODE_BRIGHTNESSDOWN :: 275;
    SDL_SCANCODE_BRIGHTNESSUP :: 276;
    SDL_SCANCODE_DISPLAYSWITCH :: 277;
    SDL_SCANCODE_KBDILLUMTOGGLE :: 278;
    SDL_SCANCODE_KBDILLUMDOWN :: 279;
    SDL_SCANCODE_KBDILLUMUP :: 280;
    SDL_SCANCODE_EJECT :: 281;
    SDL_SCANCODE_SLEEP :: 282;

    SDL_SCANCODE_APP1 :: 283;
    SDL_SCANCODE_APP2 :: 284;

    SDL_NUM_SCANCODES :: 512;
}

SDL_Surface :: struct {
    flags: u32;
    format: *SDL_PixelFormat;
    w, h: s32;
    pitch: s32;
    pixels: *void;
    userdata: *void;
    locked: s32;
    lock_data: *void;
    clip_rect: SDL_Rect;
    map: *SDL_BlitMap;
    refcount: s32;
}

using SDL_SystemCursor :: enum s32 {
    ARROW;
    IBEAM;
    WAIT;
    CROSSHAIR;
    WAITARROW;
    SIZENWSE;
    SIZENESW;
    SIZEWE;
    SIZENS;
    SIZEALL;
    NO;
    HAND;
    SDL_NUM_SYSTEM_CURSORS;
}

using SDL_SYSWM_TYPE :: enum s32 {
    SDL_SYSWM_UNKNOWN;
    SDL_SYSWM_WINDOWS;
    SDL_SYSWM_X11;
    SDL_SYSWM_DIRECTFB;
    SDL_SYSWM_COCOA;
    SDL_SYSWM_UIKIT;
}

SDL_SysWMEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;

}

SDL_SysWMinfo :: struct {
    version: SDL_version;
    subsystem: SDL_SYSWM_TYPE;
    info: union {
        #if OS == .WINDOWS {
            win: struct {

                window: *void;
            };
        } else #if OS == .LINUX {
            x11: struct {
                display: *Display;
                window: Window;
            };
        } else #if OS == .MACOS {
            cocoa: struct {

                window: *void;
            };
        }
    };
}

SDL_TextEditingEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    windowID: u32;
    text: [SDL_TEXTEDITINGEVENT_TEXT_SIZE] u8;
    start: s32;
    length: s32;
}

SDL_TextInputEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    windowID: u32;
    text: [SDL_TEXTINPUTEVENT_TEXT_SIZE] s8;
}

SDL_Texture :: struct {}

SDL_TextureAccess :: enum u32 {
    STATIC :: 0;
    STREAMING :: 1;
    TARGET :: 2;
}

SDL_TextureModulate :: enum u32 {
    NONE :: 0;
    COLOR :: 1;
    ALPHA :: 2;
}

SDL_TouchFingerEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;

}

SDL_UserEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;

}

SDL_version :: struct {
    major: u8;
    minor: u8;
    patch: u8;
}

SDL_WindowEvent :: struct {
    type: SDL_EventType;
    timestamp: u32;
    windowID: u32;
    event: SDL_WindowEventID;
    padding1: u8;
    padding2: u8;
    padding3: u8;
    data1: s32;
    data2: s32;
}

using SDL_WindowEventID :: enum u8 {
    SDL_WINDOWEVENT_NONE;
    SDL_WINDOWEVENT_SHOWN;
    SDL_WINDOWEVENT_HIDDEN;
    SDL_WINDOWEVENT_EXPOSED;
    SDL_WINDOWEVENT_MOVED;
    SDL_WINDOWEVENT_RESIZED;
    SDL_WINDOWEVENT_SIZE_CHANGED;
    SDL_WINDOWEVENT_MINIMIZED;
    SDL_WINDOWEVENT_MAXIMIZED;
    SDL_WINDOWEVENT_RESTORED;
    SDL_WINDOWEVENT_ENTER;
    SDL_WINDOWEVENT_LEAVE;
    SDL_WINDOWEVENT_FOCUS_GAINED;
    SDL_WINDOWEVENT_FOCUS_LOST;
    SDL_WINDOWEVENT_CLOSE;
    SDL_WINDOWEVENT_TAKE_FOCUS;
    SDL_WINDOWEVENT_HIT_TEST;
}

using SDL_WindowFlags :: enum_flags u32 {
    SDL_WINDOW_FULLSCREEN :: 0x00000001;
    SDL_WINDOW_OPENGL :: 0x00000002;
    SDL_WINDOW_SHOWN :: 0x00000004;
    SDL_WINDOW_HIDDEN :: 0x00000008;
    SDL_WINDOW_BORDERLESS :: 0x00000010;
    SDL_WINDOW_RESIZABLE :: 0x00000020;
    SDL_WINDOW_MINIMIZED :: 0x00000040;
    SDL_WINDOW_MAXIMIZED :: 0x00000080;
    SDL_WINDOW_INPUT_GRABBED :: 0x00000100;
    SDL_WINDOW_INPUT_FOCUS :: 0x00000200;
    SDL_WINDOW_MOUSE_FOCUS :: 0x00000400;
    SDL_WINDOW_FULLSCREEN_DESKTOP :: ((xx SDL_WINDOW_FULLSCREEN) | 0x00001000);
    SDL_WINDOW_FOREIGN :: 0x00000800;
    SDL_WINDOW_ALLOW_HIGHDPI :: 0x00002000;
    SDL_WINDOW_MOUSE_CAPTURE :: 0x00004000;
    SDL_WINDOW_ALWAYS_ON_TOP :: 0x00008000;
    SDL_WINDOW_SKIP_TASKBAR :: 0x00010000;
    SDL_WINDOW_UTILITY :: 0x00020000;
    SDL_WINDOW_TOOLTIP :: 0x00040000;
    SDL_WINDOW_POPUP_MENU :: 0x00080000;
    SDL_WINDOW_VULKAN :: 0x10000000;
}

// Callback and opaque types

SDL_AudioCallback :: #type (a0: *void, a1: *u8, a2: s32) #c_call;

SDL_AudioFilter :: #type (a0: *SDL_AudioCVT, a1: SDL_AudioFormat) #c_call;

SDL_blit :: #type (src: *SDL_Surface, srcrect: *SDL_Rect, dst: *SDL_Surface, dstrect: *SDL_Rect) -> s32 #c_call;

SDL_HintCallback :: #type (a0: *void, a1: *u8, a2: *u8, a3: *u8) #c_call;

SDL_HitTest :: #type (win: *SDL_Window, area: *SDL_Point, data: *void) -> SDL_HitTestResult #c_call;

// Aliases and constants

AUDIO_F32 :: AUDIO_F32LSB;

AUDIO_F32LSB :: 0x8120;

AUDIO_F32MSB :: 0x9120;

AUDIO_F32SYS :: AUDIO_F32LSB;

AUDIO_S16 :: AUDIO_S16LSB;

AUDIO_S16LSB :: 0x8010;

AUDIO_S16MSB :: 0x9010;

AUDIO_S16SYS :: AUDIO_S16LSB;

AUDIO_S32 :: AUDIO_S32LSB;

AUDIO_S32LSB :: 0x8020;

AUDIO_S32MSB :: 0x9020;

AUDIO_S32SYS :: AUDIO_S32LSB;

AUDIO_S8 :: 0x8008;

AUDIO_U16 :: AUDIO_U16LSB;

AUDIO_U16LSB :: 0x0010;

AUDIO_U16MSB :: 0x1010;

AUDIO_U16SYS :: AUDIO_U16LSB;

AUDIO_U8 :: 0x0008;

SDL_AUDIO_ALLOW_ANY_CHANGE :: (SDL_AUDIO_ALLOW_FREQUENCY_CHANGE | SDL_AUDIO_ALLOW_FORMAT_CHANGE | SDL_AUDIO_ALLOW_CHANNELS_CHANGE | SDL_AUDIO_ALLOW_SAMPLES_CHANGE);

SDL_AUDIO_ALLOW_CHANNELS_CHANGE :: 0x00000004;

SDL_AUDIO_ALLOW_FORMAT_CHANGE :: 0x00000002;

SDL_AUDIO_ALLOW_FREQUENCY_CHANGE :: 0x00000001;

SDL_AUDIO_ALLOW_SAMPLES_CHANGE :: 0x00000008;

SDL_AUDIO_MASK_BITSIZE :: (0xFF);

SDL_AUDIO_MASK_DATATYPE :: (1 << 8);

SDL_AUDIO_MASK_ENDIAN :: (1 << 12);

SDL_AUDIO_MASK_SIGNED :: (1 << 15);

SDL_AudioDeviceID :: u32;

SDL_AudioFormat :: u16;

SDL_BUTTON_LEFT :: 1;

SDL_BUTTON_MIDDLE :: 2;

SDL_BUTTON_RIGHT :: 3;

SDL_BUTTON_X1 :: 4;

SDL_BUTTON_X2 :: 5;

SDL_Cursor :: void;

SDL_GLContext :: *void;

SDL_HAT_CENTERED :: 0x00;

SDL_HAT_DOWN :: 0x04;

SDL_HAT_LEFT :: 0x08;

SDL_HAT_LEFTDOWN :: (SDL_HAT_LEFT | SDL_HAT_DOWN);

SDL_HAT_LEFTUP :: (SDL_HAT_LEFT | SDL_HAT_UP);

SDL_HAT_RIGHT :: 0x02;

SDL_HAT_RIGHTDOWN :: (SDL_HAT_RIGHT | SDL_HAT_DOWN);

SDL_HAT_RIGHTUP :: (SDL_HAT_RIGHT | SDL_HAT_UP);

SDL_HAT_UP :: 0x01;

SDL_HINT_ACCELEROMETER_AS_JOYSTICK :: "SDL_ACCELEROMETER_AS_JOYSTICK";

SDL_HINT_ALLOW_TOPMOST :: "SDL_ALLOW_TOPMOST";

SDL_HINT_ANDROID_APK_EXPANSION_MAIN_FILE_VERSION :: "SDL_ANDROID_APK_EXPANSION_MAIN_FILE_VERSION";

SDL_HINT_ANDROID_APK_EXPANSION_PATCH_FILE_VERSION :: "SDL_ANDROID_APK_EXPANSION_PATCH_FILE_VERSION";

SDL_HINT_ANDROID_SEPARATE_MOUSE_AND_TOUCH :: "SDL_ANDROID_SEPARATE_MOUSE_AND_TOUCH";

SDL_HINT_ANDROID_TRAP_BACK_BUTTON :: "SDL_ANDROID_TRAP_BACK_BUTTON";

SDL_HINT_APPLE_TV_CONTROLLER_UI_EVENTS :: "SDL_APPLE_TV_CONTROLLER_UI_EVENTS";

SDL_HINT_APPLE_TV_REMOTE_ALLOW_ROTATION :: "SDL_APPLE_TV_REMOTE_ALLOW_ROTATION";

SDL_HINT_AUDIO_CATEGORY :: "SDL_AUDIO_CATEGORY";

SDL_HINT_AUDIO_RESAMPLING_MODE :: "SDL_AUDIO_RESAMPLING_MODE";

SDL_HINT_BMP_SAVE_LEGACY_FORMAT :: "SDL_BMP_SAVE_LEGACY_FORMAT";

SDL_HINT_EMSCRIPTEN_KEYBOARD_ELEMENT :: "SDL_EMSCRIPTEN_KEYBOARD_ELEMENT";

SDL_HINT_ENABLE_STEAM_CONTROLLERS :: "SDL_ENABLE_STEAM_CONTROLLERS";

SDL_HINT_FRAMEBUFFER_ACCELERATION :: "SDL_FRAMEBUFFER_ACCELERATION";

SDL_HINT_GAMECONTROLLER_IGNORE_DEVICES :: "SDL_GAMECONTROLLER_IGNORE_DEVICES";

SDL_HINT_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT :: "SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT";

SDL_HINT_GAMECONTROLLERCONFIG :: "SDL_GAMECONTROLLERCONFIG";

SDL_HINT_GRAB_KEYBOARD :: "SDL_GRAB_KEYBOARD";

SDL_HINT_IDLE_TIMER_DISABLED :: "SDL_IOS_IDLE_TIMER_DISABLED";

SDL_HINT_IME_INTERNAL_EDITING :: "SDL_IME_INTERNAL_EDITING";

SDL_HINT_IOS_HIDE_HOME_INDICATOR :: "SDL_IOS_HIDE_HOME_INDICATOR";

SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS :: "SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS";

SDL_HINT_JOYSTICK_HIDAPI :: "SDL_JOYSTICK_HIDAPI";

SDL_HINT_JOYSTICK_HIDAPI_PS4 :: "SDL_JOYSTICK_HIDAPI_PS4";

SDL_HINT_JOYSTICK_HIDAPI_PS4_RUMBLE :: "SDL_JOYSTICK_HIDAPI_PS4_RUMBLE";

SDL_HINT_JOYSTICK_HIDAPI_STEAM :: "SDL_JOYSTICK_HIDAPI_STEAM";

SDL_HINT_JOYSTICK_HIDAPI_SWITCH :: "SDL_JOYSTICK_HIDAPI_SWITCH";

SDL_HINT_JOYSTICK_HIDAPI_XBOX :: "SDL_JOYSTICK_HIDAPI_XBOX";

SDL_HINT_MAC_BACKGROUND_APP :: "SDL_MAC_BACKGROUND_APP";

SDL_HINT_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK :: "SDL_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK";

SDL_HINT_MOUSE_DOUBLE_CLICK_RADIUS :: "SDL_MOUSE_DOUBLE_CLICK_RADIUS";

SDL_HINT_MOUSE_DOUBLE_CLICK_TIME :: "SDL_MOUSE_DOUBLE_CLICK_TIME";

SDL_HINT_MOUSE_FOCUS_CLICKTHROUGH :: "SDL_MOUSE_FOCUS_CLICKTHROUGH";

SDL_HINT_MOUSE_NORMAL_SPEED_SCALE :: "SDL_MOUSE_NORMAL_SPEED_SCALE";

SDL_HINT_MOUSE_RELATIVE_MODE_WARP :: "SDL_MOUSE_RELATIVE_MODE_WARP";

SDL_HINT_MOUSE_RELATIVE_SPEED_SCALE :: "SDL_MOUSE_RELATIVE_SPEED_SCALE";

SDL_HINT_NO_SIGNAL_HANDLERS :: "SDL_NO_SIGNAL_HANDLERS";

SDL_HINT_OPENGL_ES_DRIVER :: "SDL_OPENGL_ES_DRIVER";

SDL_HINT_ORIENTATIONS :: "SDL_IOS_ORIENTATIONS";

SDL_HINT_QTWAYLAND_CONTENT_ORIENTATION :: "SDL_QTWAYLAND_CONTENT_ORIENTATION";

SDL_HINT_QTWAYLAND_WINDOW_FLAGS :: "SDL_QTWAYLAND_WINDOW_FLAGS";

SDL_HINT_RENDER_DIRECT3D11_DEBUG :: "SDL_RENDER_DIRECT3D11_DEBUG";

SDL_HINT_RENDER_DIRECT3D_THREADSAFE :: "SDL_RENDER_DIRECT3D_THREADSAFE";

SDL_HINT_RENDER_DRIVER :: "SDL_RENDER_DRIVER";

SDL_HINT_RENDER_LOGICAL_SIZE_MODE :: "SDL_RENDER_LOGICAL_SIZE_MODE";

SDL_HINT_RENDER_OPENGL_SHADERS :: "SDL_RENDER_OPENGL_SHADERS";

SDL_HINT_RENDER_SCALE_QUALITY :: "SDL_RENDER_SCALE_QUALITY";

SDL_HINT_RENDER_VSYNC :: "SDL_RENDER_VSYNC";

SDL_HINT_RETURN_KEY_HIDES_IME :: "SDL_RETURN_KEY_HIDES_IME";

SDL_HINT_RPI_VIDEO_LAYER :: "SDL_RPI_VIDEO_LAYER";

SDL_HINT_THREAD_STACK_SIZE :: "SDL_THREAD_STACK_SIZE";

SDL_HINT_TIMER_RESOLUTION :: "SDL_TIMER_RESOLUTION";

SDL_HINT_TOUCH_MOUSE_EVENTS :: "SDL_TOUCH_MOUSE_EVENTS";

SDL_HINT_TV_REMOTE_AS_JOYSTICK :: "SDL_TV_REMOTE_AS_JOYSTICK";

SDL_HINT_VIDEO_ALLOW_SCREENSAVER :: "SDL_VIDEO_ALLOW_SCREENSAVER";

SDL_HINT_VIDEO_DOUBLE_BUFFER :: "SDL_VIDEO_DOUBLE_BUFFER";

SDL_HINT_VIDEO_HIGHDPI_DISABLED :: "SDL_VIDEO_HIGHDPI_DISABLED";

SDL_HINT_VIDEO_MAC_FULLSCREEN_SPACES :: "SDL_VIDEO_MAC_FULLSCREEN_SPACES";

SDL_HINT_VIDEO_MINIMIZE_ON_FOCUS_LOSS :: "SDL_VIDEO_MINIMIZE_ON_FOCUS_LOSS";

SDL_HINT_VIDEO_WIN_D3DCOMPILER :: "SDL_VIDEO_WIN_D3DCOMPILER";

SDL_HINT_VIDEO_WINDOW_SHARE_PIXEL_FORMAT :: "SDL_VIDEO_WINDOW_SHARE_PIXEL_FORMAT";

SDL_HINT_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR :: "SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR";

SDL_HINT_VIDEO_X11_NET_WM_PING :: "SDL_VIDEO_X11_NET_WM_PING";

SDL_HINT_VIDEO_X11_XINERAMA :: "SDL_VIDEO_X11_XINERAMA";

SDL_HINT_VIDEO_X11_XRANDR :: "SDL_VIDEO_X11_XRANDR";

SDL_HINT_VIDEO_X11_XVIDMODE :: "SDL_VIDEO_X11_XVIDMODE";

SDL_HINT_WINDOW_FRAME_USABLE_WHILE_CURSOR_HIDDEN :: "SDL_WINDOW_FRAME_USABLE_WHILE_CURSOR_HIDDEN";

SDL_HINT_WINDOWS_DISABLE_THREAD_NAMING :: "SDL_WINDOWS_DISABLE_THREAD_NAMING";

SDL_HINT_WINDOWS_ENABLE_MESSAGELOOP :: "SDL_WINDOWS_ENABLE_MESSAGELOOP";

SDL_HINT_WINDOWS_INTRESOURCE_ICON :: "SDL_WINDOWS_INTRESOURCE_ICON";

SDL_HINT_WINDOWS_INTRESOURCE_ICON_SMALL :: "SDL_WINDOWS_INTRESOURCE_ICON_SMALL";

SDL_HINT_WINDOWS_NO_CLOSE_ON_ALT_F4 :: "SDL_WINDOWS_NO_CLOSE_ON_ALT_F4";

SDL_HINT_WINRT_HANDLE_BACK_BUTTON :: "SDL_WINRT_HANDLE_BACK_BUTTON";

SDL_HINT_WINRT_PRIVACY_POLICY_LABEL :: "SDL_WINRT_PRIVACY_POLICY_LABEL";

SDL_HINT_WINRT_PRIVACY_POLICY_URL :: "SDL_WINRT_PRIVACY_POLICY_URL";

SDL_HINT_XINPUT_ENABLED :: "SDL_XINPUT_ENABLED";

SDL_HINT_XINPUT_USE_OLD_JOYSTICK_MAPPING :: "SDL_XINPUT_USE_OLD_JOYSTICK_MAPPING";

SDL_Joystick :: void;

SDL_JOYSTICK_AXIS_MAX :: 32767;

SDL_JOYSTICK_AXIS_MIN :: -32768;

SDL_JoystickID :: s32;

SDL_MAJOR_VERSION :: 2;

SDL_MINOR_VERSION :: 0;

SDL_PATCHLEVEL :: 1;

SDL_PRESSED :: 1;

SDL_RELEASED :: 0;

SDL_TEXTEDITINGEVENT_TEXT_SIZE :: 32;

SDL_TEXTINPUTEVENT_TEXT_SIZE :: 32;

SDL_Window :: void;

// Native storage

SDL_WINDOWPOS_UNDEFINED_MASK: s32 : 0x1FFF0000;

SDL_WINDOWPOS_UNDEFINED: s32 : 0x1FFF0000;

SDL_WINDOWPOS_CENTERED_MASK: s32 : 0x2FFF0000;

SDL_WINDOWPOS_CENTERED: s32 : 0x2FFF0000;

// Native entry points

SDL_AddHintCallback :: (name: *u8, callback: SDL_HintCallback, userdata: *void) #foreign SDL2;

SDL_AudioInit :: (driver_name: *u8) -> s32 #foreign SDL2;

SDL_AudioQuit :: () #foreign SDL2;

SDL_AudioStreamAvailable :: (stream: *SDL_AudioStream) -> s32 #foreign SDL2;

SDL_AudioStreamClear :: (stream: *SDL_AudioStream) #foreign SDL2;

SDL_AudioStreamFlush :: (stream: *SDL_AudioStream) -> s32 #foreign SDL2;

SDL_AudioStreamGet :: (stream: *SDL_AudioStream, buf: *void, len: s32) -> s32 #foreign SDL2;

SDL_AudioStreamPut :: (stream: *SDL_AudioStream, buf: *void, len: s32) -> s32 #foreign SDL2;

SDL_BuildAudioCVT :: (cvt: *SDL_AudioCVT, src_format: SDL_AudioFormat, src_channels: u8, src_rate: s32, dst_format: SDL_AudioFormat, dst_channels: u8, dst_rate: s32) -> s32 #foreign SDL2;

SDL_CaptureMouse :: (enabled: SDL_bool) -> s32 #foreign SDL2;

SDL_ClearError :: () #foreign SDL2;

SDL_ClearHints :: () #foreign SDL2;

SDL_ClearQueuedAudio :: (dev: SDL_AudioDeviceID) #foreign SDL2;

SDL_CloseAudio :: () #foreign SDL2;

SDL_CloseAudioDevice :: (dev: SDL_AudioDeviceID) #foreign SDL2;

SDL_ComposeCustomBlendMode :: (srcColorFactor: SDL_BlendFactor, dstColorFactor: SDL_BlendFactor, colorOperation: SDL_BlendOperation, srcAlphaFactor: SDL_BlendFactor, dstAlphaFactor: SDL_BlendFactor, alphaOperation: SDL_BlendOperation) -> SDL_BlendMode #foreign SDL2;

SDL_ConvertAudio :: (cvt: *SDL_AudioCVT) -> s32 #foreign SDL2;

SDL_CreateColorCursor :: (surface: *SDL_Surface, hot_x: s32, hot_y: s32) -> *SDL_Cursor #foreign SDL2;

SDL_CreateCursor :: (data: *u8, mask: *u8, w: s32, h: s32, hot_x: s32, hot_y: s32) -> *SDL_Cursor #foreign SDL2;

SDL_CreateRenderer :: (window: *SDL_Window, index: s32, flags: SDL_RendererFlags) -> *SDL_Renderer #foreign SDL2;

SDL_CreateRGBSurfaceFrom :: (pixels: *void, width: int, height: int, depth: int, pitch: int, rmask: u32, gmask: u32, bmask: u32, amask: u32) -> *SDL_Surface #foreign SDL2;

SDL_CreateRGBSurfaceWithFormatFrom :: (pixels: *void, width: int, height: int, depth: int, pitch: int, format: u32) -> *SDL_Surface #foreign SDL2;

SDL_CreateSoftwareRenderer :: (surface: *SDL_Surface) -> *SDL_Renderer #foreign SDL2;

SDL_CreateSystemCursor :: (id: SDL_SystemCursor) -> *SDL_Cursor #foreign SDL2;

SDL_CreateTexture :: (renderer: *SDL_Renderer, format: u32, access: s32, w: s32, h: s32) -> *SDL_Texture #foreign SDL2;

SDL_CreateTextureFromSurface :: (renderer: *SDL_Renderer, surface: *SDL_Surface) -> *SDL_Texture #foreign SDL2;

SDL_CreateWindow :: (title: *u8, x: s32, y: s32, w: s32, h: s32, flags: SDL_WindowFlags) -> *SDL_Window #foreign SDL2;

SDL_CreateWindowAndRenderer :: (width: s32, height: s32, window_flags: u32, window: **SDL_Window, renderer: **SDL_Renderer) -> s32 #foreign SDL2;

SDL_CreateWindowFrom :: (data: *void) -> *SDL_Window #foreign SDL2;

SDL_Delay :: (ms: u32) #foreign SDL2;

SDL_DelHintCallback :: (name: *u8, callback: SDL_HintCallback, userdata: *void) #foreign SDL2;

SDL_DequeueAudio :: (dev: SDL_AudioDeviceID, data: *void, len: u32) -> u32 #foreign SDL2;

SDL_DestroyRenderer :: (renderer: *SDL_Renderer) #foreign SDL2;

SDL_DestroyTexture :: (texture: *SDL_Texture) #foreign SDL2;

SDL_DestroyWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_DisableScreenSaver :: () #foreign SDL2;

SDL_EnableScreenSaver :: () #foreign SDL2;

SDL_EnclosePoints :: (points: *SDL_Point, count: s32, clip: *SDL_Rect, result: *SDL_Rect) -> SDL_bool #foreign SDL2;

SDL_FlushEvent :: (type: u32) #foreign SDL2;

SDL_FlushEvents :: (minType: u32, maxType: u32) #foreign SDL2;

SDL_FreeAudioStream :: (stream: *SDL_AudioStream) #foreign SDL2;

SDL_FreeCursor :: (cursor: *SDL_Cursor) #foreign SDL2;

SDL_FreeSurface :: (surface: *SDL_Surface) #foreign SDL2;

SDL_FreeWAV :: (audio_buf: *u8) #foreign SDL2;

SDL_GameControllerAddMapping :: (mappingString: *s8) -> s32 #foreign SDL2;

SDL_GameControllerClose :: (gamecontroller: *SDL_GameController) -> void #foreign SDL2;

SDL_GameControllerEventState :: (state: s32) -> s32 #foreign SDL2;

SDL_GameControllerFromInstanceID :: (joyid: SDL_JoystickID) -> *SDL_GameController #foreign SDL2;

SDL_GameControllerGetAttached :: (gamecontroller: *SDL_GameController) -> SDL_bool #foreign SDL2;

SDL_GameControllerGetAxis :: (gamecontroller: *SDL_GameController, axis: SDL_GameControllerAxis) -> s16 #foreign SDL2;

SDL_GameControllerGetAxisFromString :: (pchString: *s8) -> SDL_GameControllerAxis #foreign SDL2;

SDL_GameControllerGetBindForAxis :: (gamecontroller: *SDL_GameController, axis: SDL_GameControllerAxis) -> SDL_GameControllerButtonBind #foreign SDL2;

SDL_GameControllerGetBindForButton :: (gamecontroller: *SDL_GameController, button: SDL_GameControllerButton) -> SDL_GameControllerButtonBind #foreign SDL2;

SDL_GameControllerGetButton :: (gamecontroller: *SDL_GameController, button: SDL_GameControllerButton) -> u8 #foreign SDL2;

SDL_GameControllerGetButtonFromString :: (pchString: *s8) -> SDL_GameControllerButton #foreign SDL2;

SDL_GameControllerGetJoystick :: (gamecontroller: *SDL_GameController) -> *SDL_Joystick #foreign SDL2;

SDL_GameControllerGetProduct :: (gamecontroller: *SDL_GameController) -> u16 #foreign SDL2;

SDL_GameControllerGetProductVersion :: (gamecontroller: *SDL_GameController) -> u16 #foreign SDL2;

SDL_GameControllerGetStringForAxis :: (axis: SDL_GameControllerAxis) -> *s8 #foreign SDL2;

SDL_GameControllerGetStringForButton :: (button: SDL_GameControllerButton) -> *s8 #foreign SDL2;

SDL_GameControllerGetVendor :: (gamecontroller: *SDL_GameController) -> u16 #foreign SDL2;

SDL_GameControllerMapping :: (gamecontroller: *SDL_GameController) -> *s8 #foreign SDL2;

SDL_GameControllerMappingForGUID :: (guid: SDL_JoystickGUID) -> *s8 #foreign SDL2;

SDL_GameControllerMappingForIndex :: (mapping_index: s32) -> *s8 #foreign SDL2;

SDL_GameControllerName :: (gamecontroller: *SDL_GameController) -> *s8 #foreign SDL2;

SDL_GameControllerNameForIndex :: (joystick_index: s32) -> *s8 #foreign SDL2;

SDL_GameControllerNumMappings :: () -> s32 #foreign SDL2;

SDL_GameControllerOpen :: (joystick_index: s32) -> *SDL_GameController #foreign SDL2;

SDL_GameControllerUpdate :: () -> void #foreign SDL2;

SDL_GetAudioDeviceName :: (index: s32, iscapture: s32) -> *u8 #foreign SDL2;

SDL_GetAudioDeviceStatus :: (dev: SDL_AudioDeviceID) -> SDL_AudioStatus #foreign SDL2;

SDL_GetAudioDriver :: (index: s32) -> *u8 #foreign SDL2;

SDL_GetAudioStatus :: () -> SDL_AudioStatus #foreign SDL2;

SDL_GetClipboardText :: () -> *u8 #foreign SDL2;

SDL_GetClosestDisplayMode :: (displayIndex: s32, mode: *SDL_DisplayMode, closest: *SDL_DisplayMode) -> *SDL_DisplayMode #foreign SDL2;

SDL_GetCurrentAudioDriver :: () -> *u8 #foreign SDL2;

SDL_GetCurrentDisplayMode :: (displayIndex: s32, mode: *SDL_DisplayMode) -> s32 #foreign SDL2;

SDL_GetCurrentVideoDriver :: () -> *u8 #foreign SDL2;

SDL_GetCursor :: () -> *SDL_Cursor #foreign SDL2;

SDL_GetDefaultCursor :: () -> *SDL_Cursor #foreign SDL2;

SDL_GetDesktopDisplayMode :: (displayIndex: s32, mode: *SDL_DisplayMode) -> s32 #foreign SDL2;

SDL_GetDisplayBounds :: (displayIndex: s32, rect: *SDL_Rect) -> s32 #foreign SDL2;

SDL_GetDisplayDPI :: (displayIndex: s32, ddpi: *float, hdpi: *float, vdpi: *float) -> s32 #foreign SDL2;

SDL_GetDisplayMode :: (displayIndex: s32, modeIndex: s32, mode: *SDL_DisplayMode) -> s32 #foreign SDL2;

SDL_GetDisplayName :: (displayIndex: s32) -> *u8 #foreign SDL2;

SDL_GetError :: () -> *u8 #foreign SDL2;

SDL_GetGlobalMouseState :: (x: *s32, y: *s32) -> u32 #foreign SDL2;

SDL_GetGrabbedWindow :: () -> *SDL_Window #foreign SDL2;

SDL_GetHint :: (name: *u8) -> *u8 #foreign SDL2;

SDL_GetHintBoolean :: (name: *u8, default_value: SDL_bool) -> SDL_bool #foreign SDL2;

SDL_GetKeyboardFocus :: () -> *SDL_Window #foreign SDL2;

SDL_GetKeyboardState :: (numkeys: *s32) -> *u8 #foreign SDL2;

SDL_GetKeyFromName :: (name: *u8) -> SDL_Keycode #foreign SDL2;

SDL_GetKeyFromScancode :: (scancode: SDL_Scancode) -> SDL_Keycode #foreign SDL2;

SDL_GetKeyName :: (key: SDL_Keycode) -> *u8 #foreign SDL2;

SDL_GetModState :: () -> SDL_Keymod #foreign SDL2;

SDL_GetMouseFocus :: () -> *SDL_Window #foreign SDL2;

SDL_GetMouseState :: (x: *s32, y: *s32) -> u32 #foreign SDL2;

SDL_GetNumAudioDevices :: (iscapture: s32) -> s32 #foreign SDL2;

SDL_GetNumAudioDrivers :: () -> s32 #foreign SDL2;

SDL_GetNumDisplayModes :: (displayIndex: s32) -> s32 #foreign SDL2;

SDL_GetNumRenderDrivers :: () -> s32 #foreign SDL2;

SDL_GetNumVideoDisplays :: () -> s32 #foreign SDL2;

SDL_GetNumVideoDrivers :: () -> s32 #foreign SDL2;

SDL_GetPerformanceCounter :: () -> u64 #foreign SDL2;

SDL_GetPerformanceFrequency :: () -> u64 #foreign SDL2;

SDL_GetPlatform :: () -> *u8 #foreign SDL2;

SDL_GetQueuedAudioSize :: (dev: SDL_AudioDeviceID) -> u32 #foreign SDL2;

SDL_GetRelativeMouseMode :: () -> SDL_bool #foreign SDL2;

SDL_GetRelativeMouseState :: (x: *s32, y: *s32) -> u32 #foreign SDL2;

SDL_GetRenderDrawBlendMode :: (renderer: *SDL_Renderer, blendMode: *SDL_BlendMode) -> s32 #foreign SDL2;

SDL_GetRenderDrawColor :: (renderer: *SDL_Renderer, r: *u8, g: *u8, b: *u8, a: *u8) -> s32 #foreign SDL2;

SDL_GetRenderDriverInfo :: (index: s32, info: *SDL_RendererInfo) -> s32 #foreign SDL2;

SDL_GetRenderer :: (window: *SDL_Window) -> *SDL_Renderer #foreign SDL2;

SDL_GetRendererInfo :: (renderer: *SDL_Renderer, info: *SDL_RendererInfo) -> s32 #foreign SDL2;

SDL_GetRendererOutputSize :: (renderer: *SDL_Renderer, w: *s32, h: *s32) -> s32 #foreign SDL2;

SDL_GetRenderTarget :: (renderer: *SDL_Renderer) -> *SDL_Texture #foreign SDL2;

SDL_GetRevision :: () -> *u8 #foreign SDL2;

SDL_GetRevisionNumber :: () -> s32 #foreign SDL2;

SDL_GetScancodeFromKey :: (key: SDL_Keycode) -> SDL_Scancode #foreign SDL2;

SDL_GetScancodeFromName :: (name: *u8) -> SDL_Scancode #foreign SDL2;

SDL_GetScancodeName :: (scancode: SDL_Scancode) -> *u8 #foreign SDL2;

SDL_GetTextureAlphaMod :: (texture: *SDL_Texture, alpha: *u8) -> s32 #foreign SDL2;

SDL_GetTextureBlendMode :: (texture: *SDL_Texture, blendMode: *SDL_BlendMode) -> s32 #foreign SDL2;

SDL_GetTextureColorMod :: (texture: *SDL_Texture, r: *u8, g: *u8, b: *u8) -> s32 #foreign SDL2;

SDL_GetTicks :: () -> u32 #foreign SDL2;

SDL_GetVersion :: (ver: *SDL_version) #foreign SDL2;

SDL_GetVideoDriver :: (index: s32) -> *u8 #foreign SDL2;

SDL_GetWindowBrightness :: (window: *SDL_Window) -> float #foreign SDL2;

SDL_GetWindowData :: (window: *SDL_Window, name: *u8) -> *void #foreign SDL2;

SDL_GetWindowDisplayIndex :: (window: *SDL_Window) -> s32 #foreign SDL2;

SDL_GetWindowDisplayMode :: (window: *SDL_Window, mode: *SDL_DisplayMode) -> s32 #foreign SDL2;

SDL_GetWindowFlags :: (window: *SDL_Window) -> SDL_WindowFlags #foreign SDL2;

SDL_GetWindowFromID :: (id: u32) -> *SDL_Window #foreign SDL2;

SDL_GetWindowGammaRamp :: (window: *SDL_Window, red: *u16, green: *u16, blue: *u16) -> s32 #foreign SDL2;

SDL_GetWindowGrab :: (window: *SDL_Window) -> SDL_bool #foreign SDL2;

SDL_GetWindowID :: (window: *SDL_Window) -> u32 #foreign SDL2;

SDL_GetWindowMaximumSize :: (window: *SDL_Window, w: *s32, h: *s32) #foreign SDL2;

SDL_GetWindowMinimumSize :: (window: *SDL_Window, w: *s32, h: *s32) #foreign SDL2;

SDL_GetWindowPixelFormat :: (window: *SDL_Window) -> u32 #foreign SDL2;

SDL_GetWindowPosition :: (window: *SDL_Window, x: *s32, y: *s32) #foreign SDL2;

SDL_GetWindowSize :: (window: *SDL_Window, w: *s32, h: *s32) #foreign SDL2;

SDL_GetWindowSurface :: (window: *SDL_Window) -> *SDL_Surface #foreign SDL2;

SDL_GetWindowTitle :: (window: *SDL_Window) -> *u8 #foreign SDL2;

SDL_GL_BindTexture :: (texture: *SDL_Texture, texw: *float32, texh: *float32) -> s32 #foreign SDL2;

SDL_GL_CreateContext :: (window: *SDL_Window) -> SDL_GLContext #foreign SDL2;

SDL_GL_DeleteContext :: (ctx: SDL_GLContext) #foreign SDL2;

SDL_GL_ExtensionSupported :: (extension: *u8) -> SDL_bool #foreign SDL2;

SDL_GL_GetAttribute :: (attr: SDL_GLattr, value: *s32) -> s32 #foreign SDL2;

SDL_GL_GetCurrentContext :: () -> SDL_GLContext #foreign SDL2;

SDL_GL_GetCurrentWindow :: () -> *SDL_Window #foreign SDL2;

SDL_GL_GetDrawableSize :: (window: *SDL_Window, w: *s32, h: *s32) #foreign SDL2;

SDL_GL_GetProcAddress :: (proc: *u8) -> *void #foreign SDL2;

SDL_GL_GetSwapInterval :: () -> s32 #foreign SDL2;

SDL_GL_LoadLibrary :: (path: *u8) -> s32 #foreign SDL2;

SDL_GL_MakeCurrent :: (window: *SDL_Window, ctx: SDL_GLContext) -> s32 #foreign SDL2;

SDL_GL_ResetAttributes :: () #foreign SDL2;

SDL_GL_SetAttribute :: (attr: SDL_GLattr, value: s32) -> s32 #foreign SDL2;

SDL_GL_SetSwapInterval :: (interval: s32) -> s32 #foreign SDL2;

SDL_GL_SwapWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_GL_UnbindTexture :: (texture: *SDL_Texture) -> s32 #foreign SDL2;

SDL_GL_UnloadLibrary :: () #foreign SDL2;

SDL_HasClipboardText :: () -> SDL_bool #foreign SDL2;

SDL_HasEvent :: (type: u32) -> SDL_bool #foreign SDL2;

SDL_HasEvents :: (minType: u32, maxType: u32) -> SDL_bool #foreign SDL2;

SDL_HasIntersection :: (A: *SDL_Rect, B: *SDL_Rect) -> SDL_bool #foreign SDL2;

SDL_HasScreenKeyboardSupport :: () -> SDL_bool #foreign SDL2;

SDL_HideWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_Init :: (flags: SDL_INIT_FLAGS) -> s32 #foreign SDL2;

SDL_InitSubSystem :: (flags: SDL_INIT_FLAGS) -> s32 #foreign SDL2;

SDL_IntersectRect :: (A: *SDL_Rect, B: *SDL_Rect, result: *SDL_Rect) -> SDL_bool #foreign SDL2;

SDL_IntersectRectAndLine :: (rect: *SDL_Rect, X1: *s32, Y1: *s32, X2: *s32, Y2: *s32) -> SDL_bool #foreign SDL2;

SDL_IsGameController :: (joystick_index: s32) -> SDL_bool #foreign SDL2;

SDL_IsScreenKeyboardShown :: (window: *SDL_Window) -> SDL_bool #foreign SDL2;

SDL_IsScreenSaverEnabled :: () -> SDL_bool #foreign SDL2;

SDL_IsTextInputActive :: () -> SDL_bool #foreign SDL2;

SDL_JoystickClose :: (joystick: *SDL_Joystick) #foreign SDL2;

SDL_JoystickCurrentPowerLevel :: (joystick: *SDL_Joystick) -> SDL_JoystickPowerLevel #foreign SDL2;

SDL_JoystickEventState :: (state: s32) -> s32 #foreign SDL2;

SDL_JoystickFromInstanceID :: (joyid: SDL_JoystickID) -> *SDL_Joystick #foreign SDL2;

SDL_JoystickGetAttached :: (joystick: *SDL_Joystick) -> SDL_bool #foreign SDL2;

SDL_JoystickGetAxis :: (joystick: *SDL_Joystick, axis: s32) -> s16 #foreign SDL2;

SDL_JoystickGetBall :: (joystick: *SDL_Joystick, ball: s32, dx: *s32, dy: *s32) -> s32 #foreign SDL2;

SDL_JoystickGetButton :: (joystick: *SDL_Joystick, button: s32) -> u8 #foreign SDL2;

SDL_JoystickGetDeviceGUID :: (device_index: s32) -> SDL_JoystickGUID #foreign SDL2;

SDL_JoystickGetGUID :: (joystick: *SDL_Joystick) -> SDL_JoystickGUID #foreign SDL2;

SDL_JoystickGetGUIDFromString :: (pchGUID: *s8) -> SDL_JoystickGUID #foreign SDL2;

SDL_JoystickGetGUIDString :: (guid: SDL_JoystickGUID, pszGUID: *s8, cbGUID: s32) #foreign SDL2;

SDL_JoystickGetHat :: (joystick: *SDL_Joystick, hat: s32) -> u8 #foreign SDL2;

SDL_JoystickInstanceID :: (joystick: *SDL_Joystick) -> SDL_JoystickID #foreign SDL2;

SDL_JoystickName :: (joystick: *SDL_Joystick) -> *s8 #foreign SDL2;

SDL_JoystickNameForIndex :: (device_index: s32) -> *s8 #foreign SDL2;

SDL_JoystickNumAxes :: (joystick: *SDL_Joystick) -> s32 #foreign SDL2;

SDL_JoystickNumBalls :: (joystick: *SDL_Joystick) -> s32 #foreign SDL2;

SDL_JoystickNumButtons :: (joystick: *SDL_Joystick) -> s32 #foreign SDL2;

SDL_JoystickNumHats :: (joystick: *SDL_Joystick) -> s32 #foreign SDL2;

SDL_JoystickOpen :: (device_index: s32) -> *SDL_Joystick #foreign SDL2;

SDL_JoystickUpdate :: () #foreign SDL2;

SDL_LockAudio :: () #foreign SDL2;

SDL_LockAudioDevice :: (dev: SDL_AudioDeviceID) #foreign SDL2;

SDL_LockTexture :: (texture: *SDL_Texture, rect: *SDL_Rect, pixels: **void, pitch: *s32) -> s32 #foreign SDL2;

SDL_MaximizeWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_MinimizeWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_MixAudio :: (dst: *u8, src: *u8, len: u32, volume: s32) #foreign SDL2;

SDL_MixAudioFormat :: (dst: *u8, src: *u8, format: SDL_AudioFormat, len: u32, volume: s32) #foreign SDL2;

SDL_NewAudioStream :: (src_format: SDL_AudioFormat, src_channels: u8, src_rate: s32, dst_format: SDL_AudioFormat, dst_channels: u8, dst_rate: s32) -> *SDL_AudioStream #foreign SDL2;

SDL_NumJoysticks :: () -> s32 #foreign SDL2;

SDL_OpenAudio :: (desired: *SDL_AudioSpec, obtained: *SDL_AudioSpec) -> s32 #foreign SDL2;

SDL_OpenAudioDevice :: (device: *u8, iscapture: s32, desired: *SDL_AudioSpec, obtained: *SDL_AudioSpec, allowed_changes: s32) -> SDL_AudioDeviceID #foreign SDL2;

SDL_PauseAudio :: (pause_on: s32) #foreign SDL2;

SDL_PauseAudioDevice :: (dev: SDL_AudioDeviceID, pause_on: s32) #foreign SDL2;

SDL_PeepEvents :: (events: *SDL_Event, numevents: s32, action: SDL_eventaction, minType: u32, maxType: u32) -> s32 #foreign SDL2;

SDL_PollEvent :: (event: *SDL_Event) -> s32 #foreign SDL2;

SDL_PumpEvents :: () #foreign SDL2;

SDL_PushEvent :: (event: *SDL_Event) -> s32 #foreign SDL2;

SDL_QueryTexture :: (texture: *SDL_Texture, format: *u32, access: *s32, w: *s32, h: *s32) -> s32 #foreign SDL2;

SDL_QueueAudio :: (dev: SDL_AudioDeviceID, data: *void, len: u32) -> s32 #foreign SDL2;

SDL_Quit :: () #foreign SDL2;

SDL_QuitSubSystem :: (flags: SDL_INIT_FLAGS) #foreign SDL2;

SDL_RaiseWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_RenderClear :: (renderer: *SDL_Renderer) -> s32 #foreign SDL2;

SDL_RenderCopy :: (renderer: *SDL_Renderer, texture: *SDL_Texture, srcrect: *SDL_Rect, dstrect: *SDL_Rect) -> s32 #foreign SDL2;

SDL_RenderCopyEx :: (renderer: *SDL_Renderer, texture: *SDL_Texture, srcrect: *SDL_Rect, dstrect: *SDL_Rect, angle: float64, center: *SDL_Point, flip: SDL_RendererFlip) -> s32 #foreign SDL2;

SDL_RenderDrawLine :: (renderer: *SDL_Renderer, x1: s32, y1: s32, x2: s32, y2: s32) -> s32 #foreign SDL2;

SDL_RenderDrawLines :: (renderer: *SDL_Renderer, points: *SDL_Point, count: s32) -> s32 #foreign SDL2;

SDL_RenderDrawPoint :: (renderer: *SDL_Renderer, x: s32, y: s32) -> s32 #foreign SDL2;

SDL_RenderDrawPoints :: (renderer: *SDL_Renderer, points: *SDL_Point, count: s32) -> s32 #foreign SDL2;

SDL_RenderDrawRect :: (renderer: *SDL_Renderer, rect: *SDL_Rect) -> s32 #foreign SDL2;

SDL_RenderDrawRects :: (renderer: *SDL_Renderer, rects: *SDL_Rect, count: s32) -> s32 #foreign SDL2;

SDL_RenderFillRect :: (renderer: *SDL_Renderer, rect: *SDL_Rect) -> s32 #foreign SDL2;

SDL_RenderFillRects :: (renderer: *SDL_Renderer, rects: *SDL_Rect, count: s32) -> s32 #foreign SDL2;

SDL_RenderGetClipRect :: (renderer: *SDL_Renderer, rect: *SDL_Rect) #foreign SDL2;

SDL_RenderGetIntegerScale :: (renderer: *SDL_Renderer) -> SDL_bool #foreign SDL2;

SDL_RenderGetLogicalSize :: (renderer: *SDL_Renderer, w: *s32, h: *s32) #foreign SDL2;

#if OS == .MACOS {
    SDL_RenderGetMetalCommandEncoder :: (renderer: *SDL_Renderer) -> *void #foreign SDL2;
}

#if OS == .MACOS {
    SDL_RenderGetMetalLayer :: (renderer: *SDL_Renderer) -> *void #foreign SDL2;
}

SDL_RenderGetScale :: (renderer: *SDL_Renderer, scaleX: *float32, scaleY: *float32) #foreign SDL2;

SDL_RenderGetViewport :: (renderer: *SDL_Renderer, rect: *SDL_Rect) #foreign SDL2;

SDL_RenderIsClipEnabled :: (renderer: *SDL_Renderer) -> SDL_bool #foreign SDL2;

SDL_RenderPresent :: (renderer: *SDL_Renderer) #foreign SDL2;

SDL_RenderReadPixels :: (renderer: *SDL_Renderer, rect: *SDL_Rect, format: u32, pixels: *void, pitch: s32) -> s32 #foreign SDL2;

SDL_RenderSetClipRect :: (renderer: *SDL_Renderer, rect: *SDL_Rect) -> s32 #foreign SDL2;

SDL_RenderSetIntegerScale :: (renderer: *SDL_Renderer, enable: SDL_bool) -> s32 #foreign SDL2;

SDL_RenderSetLogicalSize :: (renderer: *SDL_Renderer, w: s32, h: s32) -> s32 #foreign SDL2;

SDL_RenderSetScale :: (renderer: *SDL_Renderer, scaleX: float32, scaleY: float32) -> s32 #foreign SDL2;

SDL_RenderSetViewport :: (renderer: *SDL_Renderer, rect: *SDL_Rect) -> s32 #foreign SDL2;

SDL_RenderTargetSupported :: (renderer: *SDL_Renderer) -> SDL_bool #foreign SDL2;

SDL_RestoreWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_RWFromFile :: (path: *u8, mode: *u8) -> *SDL_RWops #foreign SDL2;

SDL_SetClipboardText :: (text: *u8) -> s32 #foreign SDL2;

SDL_SetCursor :: (cursor: *SDL_Cursor) #foreign SDL2;

SDL_SetHint :: (name: *u8, value: *u8) -> SDL_bool #foreign SDL2;

SDL_SetHintWithPriority :: (name: *u8, value: *u8, priority: SDL_HintPriority) -> SDL_bool #foreign SDL2;

SDL_SetModState :: (modstate: SDL_Keymod) #foreign SDL2;

SDL_SetRelativeMouseMode :: (enabled: SDL_bool) -> s32 #foreign SDL2;

SDL_SetRenderDrawBlendMode :: (renderer: *SDL_Renderer, blendMode: SDL_BlendMode) -> s32 #foreign SDL2;

SDL_SetRenderDrawColor :: (renderer: *SDL_Renderer, r: u8, g: u8, b: u8, a: u8) -> s32 #foreign SDL2;

SDL_SetRenderTarget :: (renderer: *SDL_Renderer, texture: *SDL_Texture) -> s32 #foreign SDL2;

SDL_SetTextInputRect :: (rect: *SDL_Rect) #foreign SDL2;

SDL_SetTextureAlphaMod :: (texture: *SDL_Texture, alpha: u8) -> s32 #foreign SDL2;

SDL_SetTextureBlendMode :: (texture: *SDL_Texture, blendMode: SDL_BlendMode) -> s32 #foreign SDL2;

SDL_SetTextureColorMod :: (texture: *SDL_Texture, r: u8, g: u8, b: u8) -> s32 #foreign SDL2;

SDL_SetWindowBordered :: (window: *SDL_Window, bordered: SDL_bool) #foreign SDL2;

SDL_SetWindowBrightness :: (window: *SDL_Window, brightness: float) -> s32 #foreign SDL2;

SDL_SetWindowData :: (window: *SDL_Window, name: *u8, userdata: *void) -> *void #foreign SDL2;

SDL_SetWindowDisplayMode :: (window: *SDL_Window, mode: *SDL_DisplayMode) -> s32 #foreign SDL2;

SDL_SetWindowFullscreen :: (window: *SDL_Window, flags: u32) -> s32 #foreign SDL2;

SDL_SetWindowGammaRamp :: (window: *SDL_Window, red: *u16, green: *u16, blue: *u16) -> s32 #foreign SDL2;

SDL_SetWindowGrab :: (window: *SDL_Window, grabbed: SDL_bool) #foreign SDL2;

SDL_SetWindowHitTest :: (window: *SDL_Window, callback: SDL_HitTest, callback_data: *void) -> s32 #foreign SDL2;

SDL_SetWindowIcon :: (window: *SDL_Window, icon: *SDL_Surface) #foreign SDL2;

SDL_SetWindowMaximumSize :: (window: *SDL_Window, w: s32, h: s32) #foreign SDL2;

SDL_SetWindowMinimumSize :: (window: *SDL_Window, w: s32, h: s32) #foreign SDL2;

SDL_SetWindowPosition :: (window: *SDL_Window, x: s32, y: s32) #foreign SDL2;

SDL_SetWindowResizable :: (window: *SDL_Window, resizable: SDL_bool) #foreign SDL2;

SDL_SetWindowSize :: (window: *SDL_Window, w: s32, h: s32) #foreign SDL2;

SDL_SetWindowTitle :: (window: *SDL_Window, title: *u8) #foreign SDL2;

SDL_ShowCursor :: (toggle: s32) -> s32 #foreign SDL2;

SDL_ShowSimpleMessageBox :: (flags: u32, title: *u8, msg: *u8, parent: *SDL_Window) -> int #foreign SDL2;

SDL_ShowWindow :: (window: *SDL_Window) #foreign SDL2;

SDL_StartTextInput :: () #foreign SDL2;

SDL_StopTextInput :: () #foreign SDL2;

SDL_UnionRect :: (A: *SDL_Rect, B: *SDL_Rect, result: *SDL_Rect) #foreign SDL2;

SDL_UnlockAudio :: () #foreign SDL2;

SDL_UnlockAudioDevice :: (dev: SDL_AudioDeviceID) #foreign SDL2;

SDL_UnlockTexture :: (texture: *SDL_Texture) #foreign SDL2;

SDL_UpdateTexture :: (texture: *SDL_Texture, rect: *SDL_Rect, pixels: *void, pitch: s32) -> s32 #foreign SDL2;

SDL_UpdateWindowSurface :: (window: *SDL_Window) -> s32 #foreign SDL2;

SDL_UpdateWindowSurfaceRects :: (window: *SDL_Window, rects: *SDL_Rect, numrects: s32) -> s32 #foreign SDL2;

SDL_UpdateYUVTexture :: (texture: *SDL_Texture, rect: *SDL_Rect, Yplane: *u8, Ypitch: s32, Uplane: *u8, Upitch: s32, Vplane: *u8, Vpitch: s32) -> s32 #foreign SDL2;

SDL_VideoInit :: (driver_name: *u8) -> s32 #foreign SDL2;

SDL_VideoQuit :: () #foreign SDL2;

SDL_WaitEvent :: (event: *SDL_Event) -> s32 #foreign SDL2;

SDL_WaitEventTimeout :: (event: *SDL_Event, timeout: s32) -> s32 #foreign SDL2;

SDL_WarpMouseGlobal :: (x: s32, y: s32) -> s32 #foreign SDL2;

SDL_WarpMouseInWindow :: (window: *SDL_Window, x: s32, y: s32) #foreign SDL2;

SDL_WasInit :: (flags: SDL_INIT_FLAGS) -> SDL_INIT_FLAGS #foreign SDL2;

// Convenience operations

SDL_AUDIO_BITSIZE :: (x: SDL_AudioFormat) -> u16 {
    return x & SDL_AUDIO_MASK_BITSIZE;
}

SDL_AUDIO_ISBIGENDIAN :: (x: SDL_AudioFormat) -> bool {
    return (x & SDL_AUDIO_MASK_ENDIAN) != 0;
}

SDL_AUDIO_ISFLOAT :: (x: SDL_AudioFormat) -> bool {
    return (x & SDL_AUDIO_MASK_DATATYPE) != 0;
}

SDL_AUDIO_ISINT :: (x: SDL_AudioFormat) -> bool {
    return !SDL_AUDIO_ISFLOAT(x);
}

SDL_AUDIO_ISLITTLEENDIAN :: (x: SDL_AudioFormat) -> bool {
    return !SDL_AUDIO_ISBIGENDIAN(x);
}

SDL_AUDIO_ISSIGNED :: (x: SDL_AudioFormat) -> bool {
    return (x & SDL_AUDIO_MASK_SIGNED) != 0;
}

SDL_AUDIO_ISUNSIGNED :: (x: SDL_AudioFormat) -> bool {
    return !SDL_AUDIO_ISSIGNED(x);
}

SDL_BUTTON :: (x: int) -> int {
    return 1 << (x - 1);
}

SDL_LoadBMP :: inline (path: *u8) -> *SDL_Surface {
    return SDL_LoadBMP_RW(SDL_RWFromFile(path, "rb"), 1);
}

SDL_PointInRect :: (p: *SDL_Point, r: *SDL_Rect) -> SDL_bool {
    return cast(SDL_bool) (p.x >= r.x && p.x < r.x + r.w && p.y >= r.y && p.y < r.y + r.h);
}

SDL_RectEmpty :: (r: *SDL_Rect) -> SDL_bool {
    return cast(SDL_bool) (!r || r.w <= 0 || r.h <= 0);
}

SDL_RectEquals :: (a: *SDL_Rect, b: *SDL_Rect) -> SDL_bool {
    return cast(SDL_bool) (a != null && b != null && a.x == b.x && a.y == b.y && a.w == b.w && a.h == b.h);
}

SDL_SCANCODE_TO_KEYCODE :: inline (scancode: SDL_Scancode) -> s32 {
    return cast(s32) scancode | (1 << 30);
}

SDL_VERSION :: (x: *$T) {
    x.major = SDL_MAJOR_VERSION;
    x.minor = SDL_MINOR_VERSION;
    x.patch = SDL_PATCHLEVEL;
}

SDL_VERSIONNUM :: (X: s32, Y: s32, Z: s32) -> s32 {
    return X * 1000 + Y * 100 + Z;
}

SDL_WINDOWPOS_CENTERED_DISPLAY :: (x: s32) -> s32 {
    return SDL_WINDOWPOS_CENTERED_MASK | x;
}

SDL_WINDOWPOS_UNDEFINED_DISPLAY :: (x: s32) -> s32 {
    return SDL_WINDOWPOS_UNDEFINED_MASK | x;
}
