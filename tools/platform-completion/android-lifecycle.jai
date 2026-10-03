android_main :: (app: *Android_App) #c_call {
    if app == null || app.activity == null || _sdk_android_app != null return;
    // NativeActivity glue invokes this entry on its own thread. Establish a full
    // Jai context before invoking allocators, loggers or application callbacks.
    initializer_of(#Context)(*_sdk_android_context);
    push_context _sdk_android_context {
        if !Android_Basic.init_temporary_storage(*_sdk_android_storage, 65536) return;
        defer Android_Basic.deinit_temporary_storage(*_sdk_android_storage, free_original = true);
        context.temporary_storage = *_sdk_android_storage;
        context.android_app = app;
        _sdk_android_context = context;
        _sdk_android_app = app;
        app.onAppCmd = my_android_handle_command;
        app.onInputEvent = my_android_handle_input;
        previous_pump := Input.input_event_source;
        previous_pointer := Input.input_pointer_source;
        Input.input_event_source = _sdk_android_input_pump;
        Input.input_pointer_source = null;
        defer {
            Input.input_event_source = previous_pump;
            Input.input_pointer_source = previous_pointer;
            _sdk_android_app = null;
        }
        while app.destroyRequested == 0 && app.window == null {
            if !_sdk_android_poll(app, -1) return;
        }
        if app.destroyRequested == 0 DEFINE_ANDROID_MAIN_AND_CALL_THIS();
        if app.destroyRequested == 0 {
            ANativeActivity_finish(app.activity);
            while app.destroyRequested == 0 {
                if !_sdk_android_poll(app, -1) break;
            }
        }
    }
}
my_android_handle_command :: (app: *Android_App, cmd: s32) #c_call {
    if app == null || app != _sdk_android_app return;
    push_context _sdk_android_context {
        window := cast(s64) (cast(u64) app.window);
        if cmd == 6 Input.set_input_focus(window, true);
        else if cmd == 7 || cmd == 13 || cmd == 14 || cmd == 2 {
            Input.set_input_focus(window, false);
            _sdk_android_pointer_valid = false;
            Input.input_pointer_source = null;
        }
        else if cmd == 1 || cmd == 3 || cmd == 5 || cmd == 8 {
            if app.window != null Input.queue_window_resize(window, ANativeWindow_getWidth(app.window), ANativeWindow_getHeight(app.window));
        } else if cmd == 15 {
            Input.set_input_focus(window, false);
            quit: Input.Event; quit.type = .QUIT;
            Input.queue_input_event(window, quit);
        }
    }
}
my_android_handle_input :: (app: *Android_App, event: *AInputEvent) -> s32 #c_call {
    if app == null || app != _sdk_android_app || event == null return 0;
    push_context _sdk_android_context {
        window := cast(s64) (cast(u64) app.window);
        event_type := cast(s32) AInputEvent_getType(event);
        if event_type == 1 {
            action := AKeyEvent_getAction(event);
            if action != 0 && action != 1 return 0;
            native_code := AKeyEvent_getKeyCode(event);
            key := _sdk_android_key(native_code);
            if key == .UNKNOWN return 0;
            meta := AKeyEvent_getMetaState(event);
            translated: Input.Event;
            translated.type = .KEYBOARD;
            translated.key_code = key;
            translated.key_pressed = action == 0;
            translated.repeat = AKeyEvent_getRepeatCount(event) > 0;
            translated.shift_pressed = (meta & 1) != 0;
            translated.alt_pressed = (meta & 2) != 0;
            translated.ctrl_pressed = (meta & 0x1000) != 0;
            translated.cmd_meta_pressed = (meta & 0x10000) != 0;
            Input.queue_input_event(window, translated);
            if action == 0 && !translated.ctrl_pressed && !translated.cmd_meta_pressed {
                codepoint := _sdk_android_unicode(app, event, meta);
                // Android marks dead-key accents with the high bit. Composed IME
                // text belongs to an application InputConnection, not this event.
                if codepoint >= 32 && codepoint <= 0x10ffff && !(codepoint >= 0xd800 && codepoint <= 0xdfff) {
                    text: Input.Event; text.type = .TEXT_INPUT;
                    text.utf32 = codepoint; text.text_input_count = 1;
                    Input.queue_input_event(window, text);
                }
            }
            return 1;
        }
        if event_type != 2 return 0;
        buttons := cast(u32) AMotionEvent_getButtonState(event);
        for index: 0..2 {
            mask := cast(u32) 1 << index;
            if (buttons & mask) == (_sdk_android_mouse_buttons & mask) continue;
            button: Input.Event; button.type = .KEYBOARD;
            if index == 0 button.key_code = .MOUSE_BUTTON_LEFT;
            else if index == 1 button.key_code = .MOUSE_BUTTON_RIGHT;
            else button.key_code = .MOUSE_BUTTON_MIDDLE;
            button.key_pressed = (buttons & mask) != 0;
            Input.queue_input_event(window, button);
        }
        _sdk_android_mouse_buttons = buttons;
        action := AMotionEvent_getAction(event);
        kind := action & 255;
        pointers := AMotionEvent_getPointerCount(event);
        if pointers == 0 return 0;
        if kind == 8 {
            scroll := AMotionEvent_getAxisValue(event, 9, 0);
            wheel: Input.Event; wheel.type = .MOUSE_WHEEL;
            wheel.typical_wheel_delta = 120; wheel.wheel_delta = cast(s32) (scroll * 120);
            Input.queue_input_event(window, wheel);
            return 1;
        }
        selected := cast(u64) ((action >> 8) & 255);
        if (kind == 5 || kind == 6) && selected >= pointers return 0;
        index: u64 = 0;
        while index < pointers {
            if kind == 0 || kind == 1 || kind == 5 || kind == 6 {
                if (kind == 5 || kind == 6) && index != selected { index += 1; continue; }
                if (kind == 0 || kind == 1) && index != 0 { index += 1; continue; }
            } else if kind != 2 && kind != 3 && kind != 7 return 0;
            touch: Input.Event; touch.type = .TOUCH;
            touch.touch_index = AMotionEvent_getPointerId(event, index);
            if kind == 0 || kind == 5 touch.touch_type = .PRESSED;
            else if kind == 1 || kind == 6 || kind == 3 touch.touch_type = .RELEASED;
            else touch.touch_type = .MOVED;
            Input.queue_input_event(window, touch);
            if index == 0 {
                x := AMotionEvent_getX(event, index); y := AMotionEvent_getY(event, index);
                if _sdk_android_pointer_valid Input.queue_mouse_delta(cast(int) (x - _sdk_android_x), cast(int) (y - _sdk_android_y));
                _sdk_android_x = x; _sdk_android_y = y; _sdk_android_pointer_valid = true;
                Input.input_pointer_source = _sdk_android_pointer;
            }
            index += 1;
        }
        return 1;
    }
}
// PRIVATE HELPERS
Android_Basic :: #import "Basic";
Android_Runtime :: #import "Runtime_Support" (DEFINE_SYSTEM_ENTRY_POINT=false, DEFINE_INITIALIZATION=false, ENABLE_BACKTRACE_ON_CRASH=false);
#if DEFINE_ANDROID_MAIN_AND_CALL_THIS {
    _sdk_android_context: #Context;
    _sdk_android_storage: Android_Runtime.Temporary_Storage;
    _sdk_android_mouse_buttons: u32;
    _sdk_android_app: *Android_App;
    _sdk_android_x: float; _sdk_android_y: float;
    _sdk_android_pointer_valid := false;
    _sdk_android_poll :: (app: *Android_App, timeout: s32) -> bool {
        data: *void;
        ident := ALooper_pollOnce(timeout, null, null, *data);
        if ident == -4 return false;
        if (ident == 1 || ident == 2) && data != null {
            source := cast(*Android_Poll_Source) data;
            // Native-app-glue owns and updates app state around this callback.
            if source.process != null source.process(app, source);
        }
        return true;
    }
    _sdk_android_input_pump :: () {
        app := _sdk_android_app;
        if app == null return;
        for iteration: 0..255 {
            if app.destroyRequested != 0 break;
            data: *void;
            ident := ALooper_pollOnce(0, null, null, *data);
            if ident == -3 || ident == -4 break;
            if (ident == 1 || ident == 2) && data != null {
                source := cast(*Android_Poll_Source) data;
                if source.process != null source.process(app, source);
            }
        }
    }
    _sdk_android_pointer :: (window: s64, client_area: bool) -> float, float {
        Android_Basic.assert(_sdk_android_app != null && _sdk_android_pointer_valid, "No Android pointer event has arrived");
        Android_Basic.assert(window == 0 || window == cast(s64) (cast(u64) _sdk_android_app.window), "Pointer belongs to a different native window");
        return _sdk_android_x, _sdk_android_y;
    }
    _sdk_android_key :: (key: s32) -> Input.Key_Code #no_context {
        if key >= 7 && key <= 16 return cast(Input.Key_Code) (48 + key - 7);
        if key >= 29 && key <= 54 return cast(Input.Key_Code) (65 + key - 29);
        if key >= 131 && key <= 142 return cast(Input.Key_Code) (cast(s32) Input.Key_Code.F1 + key - 131);
        if key == 4 || key == 111 return .ESCAPE;
        if key == 19 return .ARROW_UP;
        if key == 20 return .ARROW_DOWN;
        if key == 21 return .ARROW_LEFT;
        if key == 22 return .ARROW_RIGHT;
        if key == 61 return .TAB;
        if key == 62 return .SPACEBAR;
        if key == 66 return .ENTER;
        if key == 67 return .BACKSPACE;
        if key == 112 return .DELETE;
        if key == 92 return .PAGE_UP;
        if key == 93 return .PAGE_DOWN;
        if key == 122 return .HOME;
        if key == 123 return .END;
        if key == 124 return .INSERT;
        if key == 57 || key == 58 return .ALT;
        if key == 59 || key == 60 return .SHIFT;
        if key == 113 || key == 114 return .CTRL;
        if key == 117 || key == 118 return .CMD;
        if key == 55 return cast(Input.Key_Code) 44;
        if key == 56 return cast(Input.Key_Code) 46;
        if key == 68 return cast(Input.Key_Code) 96;
        if key == 69 return cast(Input.Key_Code) 45;
        if key == 70 return cast(Input.Key_Code) 61;
        if key == 71 return cast(Input.Key_Code) 91;
        if key == 72 return cast(Input.Key_Code) 93;
        if key == 73 return cast(Input.Key_Code) 92;
        if key == 74 return cast(Input.Key_Code) 59;
        if key == 75 return cast(Input.Key_Code) 39;
        if key == 76 return cast(Input.Key_Code) 47;
        return .UNKNOWN;
    }
    _sdk_android_unicode :: (app: *Android_App, event: *AInputEvent, meta: s32) -> u32 {
        vm := app.activity.vm;
        if vm == null return 0;
        env: *JNIEnv;
        attached := false;
        status := vm.*.GetEnv(vm, cast(**void) *env, JNI_VERSION_1_6);
        if status == JNI_EDETACHED {
            if vm.*.AttachCurrentThread(vm, *env, null) != JNI_OK return 0;
            attached = true;
        } else if status != JNI_OK return 0;
        defer if attached vm.*.DetachCurrentThread(vm);
        if env == null || env.*.ExceptionCheck(env) != 0 return 0;
        key_class := env.*.FindClass(env, cast(*u8) "android/view/KeyEvent\0");
        if _sdk_android_jni_failed(env) || key_class == null return 0;
        defer env.*.DeleteLocalRef(env, key_class);
        constructor := env.*.GetMethodID(env, key_class, cast(*u8) "<init>\0", cast(*u8) "(JJIIIIIIII)V\0");
        if _sdk_android_jni_failed(env) || constructor == null return 0;
        args: [10] jvalue;
        args[0].j = AKeyEvent_getDownTime(event) / 1000000;
        args[1].j = AKeyEvent_getEventTime(event) / 1000000;
        args[2].i = AKeyEvent_getAction(event);
        args[3].i = AKeyEvent_getKeyCode(event);
        args[4].i = AKeyEvent_getRepeatCount(event);
        args[5].i = meta;
        args[6].i = AInputEvent_getDeviceId(event);
        args[7].i = AKeyEvent_getScanCode(event);
        args[8].i = AKeyEvent_getFlags(event);
        args[9].i = AInputEvent_getSource(event);
        java_key := env.*.NewObjectA(env, key_class, constructor, *args[0]);
        if _sdk_android_jni_failed(env) || java_key == null return 0;
        defer env.*.DeleteLocalRef(env, java_key);
        unicode_method := env.*.GetMethodID(env, key_class, cast(*u8) "getUnicodeChar\0", cast(*u8) "(I)I\0");
        if _sdk_android_jni_failed(env) || unicode_method == null return 0;
        unicode_args: [1] jvalue; unicode_args[0].i = meta;
        scalar := env.*.CallIntMethodA(env, java_key, unicode_method, *unicode_args[0]);
        if _sdk_android_jni_failed(env) return 0;
        return cast(u32) scalar;
    }
    _sdk_android_jni_failed :: (env: *JNIEnv) -> bool #no_context {
        if env.*.ExceptionCheck(env) == 0 return false;
        env.*.ExceptionClear(env);
        return true;
    }
}
