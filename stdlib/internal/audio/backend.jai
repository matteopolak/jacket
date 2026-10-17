#if OS == .LINUX MY_SAMPLING_RATE :: 48000;
#if OS == .MACOS || OS == .WINDOWS OUTPUT_SAMPLING_RATE :: 44100;
#if OS == .MACOS FILL_BUFFER_SIZE_IN_SAMPLES :: OUTPUT_SAMPLING_RATE / 2;
#if OS == .WINDOWS FILL_BUFFER_SIZE_IN_SAMPLES :: 24000;

backend_get_devices :: () -> [] Output_Device {
    lock_producer();
    defer finish_backend_call();
    push_allocator(context.default_allocator);
    candidates: [..] Output_Device;
    index: s32;

    while true {
        name: [1024] u8;
        status := audio_output_device(index, name.data, cast(s32) name.count);

        if status < 0 {
            device_error = status;
            device_notice |= 1;
            array_free(candidates);
            return output_devices;
        }

        if status == 0 break;

        device: Output_Device;
        label := to_string(name);

        for stored: device_names if stored == label {
            device.name = stored;
            break;
        }

        if device.name.count == 0 {
            device.name = copy_string(label);
            array_add(*device_names, device.name);
        }

        device.device_index = index;
        array_add(*candidates, device);
        index += 1;
    }

    same := candidates.count == output_devices.count;
    if same for i: 0..candidates.count - 1 if candidates[i].name != output_devices[i].name same = false;

    if same {
        array_free(candidates);
        return output_devices;
    }

    // Retain prior returned snapshots: callers may still hold their slices.
    array_add(*device_snapshots, candidates);
    output_devices = candidates;

    return output_devices;
}

backend_init :: (config: Sound_Player_Config) -> Backend_Properties {
    lock_producer();
    defer finish_backend_call();

    if output_handle {
        existing := properties;
        if device_lost existing.initted = false;
        return existing;
    }

    result: Backend_Properties;
    if !is_finite(config.seconds_to_fill_ahead) || config.seconds_to_fill_ahead <= 0 return result;

    channels := audio_output_preferred_channels();
    if channels != 2 && channels != 6 && channels != 8 return result;

    error: s32;

    output_handle = audio_output_open(native_rate, channels, *error);
    if !output_handle return result;

    result.initted = true;
    result.num_channels = audio_output_channels(output_handle);
    result.output_sampling_rate = audio_output_rate(output_handle);
    result.channel_names = .{ result.num_channels, channel_names.data };
    initialize_panning_vectors();
    if result.num_channels == 6 arcs = arcs_6ch;
    else if result.num_channels == 8 arcs = arcs_8ch;
    else arcs = .{};

    properties = result;
    backend_config = config;
    next_device_poll = 0;
    device_paused = false;
    device_lost = false;
    restart_after_fill = false;
    capacity := min(audio_output_capacity(output_handle), cast(s64) 8 * staging.count);
    if capacity < result.num_channels * 2 {
        close_output_device();
        return .{};
    }

    frames := min(cast(float64) capacity / (result.num_channels * 2), cast(float64) config.seconds_to_fill_ahead * result.output_sampling_rate);

    minimum_bytes = max(cast(s64) 1, cast(s64) frames) * result.num_channels * 2;

    return result;
}

#if OS == .WINDOWS {
    backend_count_buffered_bytes :: () -> (currently_buffered: s64, minimum_prebuffered: s64) {
        return count_output_bytes();
    }
} else #if OS == .MACOS {
    backend_count_buffered_bytes :: () -> (currently_buffered: s64, minimum_prebuffered_bytes: s64) {
        return count_output_bytes();
    }
} else #if OS == .LINUX {
    backend_count_buffered_bytes :: () -> (buffered_bytes: s64, minimum_prebuffered_bytes: s64) {
        return count_output_bytes();
    }
}

backend_lock_fill_regions :: (bytes_to_lock: s64) -> Fill_Region_Result, bool {
    lock_producer();
    defer unlock_producer();
    result: Fill_Region_Result;
    if !output_handle || device_lost return result, false;
    if region_locked || bytes_to_lock <= 0 || bytes_to_lock > staging.count || bytes_to_lock % (properties.num_channels * 2) != 0 return result, false;

    region_locked = true;
    locked_bytes = bytes_to_lock;
    result.buffer0 = .{ bytes_to_lock, staging.data };

    return result, true;
}

backend_release_fill_regions :: (result: Fill_Region_Result) {
    lock_producer();
    defer finish_backend_call();
    assert(output_handle && region_locked);
    assert(result.buffer0.data == staging.data && result.buffer0.count == locked_bytes && result.buffer1.count == 0, "invalid audio fill region release");

    if device_lost {
        region_locked = false;
        locked_bytes = 0;
        lose_output_device(device_error);
        return;
    }

    status := audio_output_submit(output_handle, result.buffer0.data, cast(s32) result.buffer0.count);

    region_locked = false;
    locked_bytes = 0;

    if status != 0 {
        lose_output_device(status);
        return;
    }

    if restart_after_fill {
        restart_after_fill = false;
        status = audio_output_play(output_handle);
        if status != 0 lose_output_device(status);
    }
}

backend_play :: () {
    lock_producer();
    defer finish_backend_call();
    assert(output_handle != null, "audio output is not initialized");
    status := audio_output_play(output_handle);

    if status != 0 lose_output_device(status);
}

backend_needs_async_update_from_main_thread :: () -> bool {
    #if OS == .LINUX return false;
    return true;
}

backend_shutdown :: () {
    lock_producer();
    defer finish_backend_call();
    close_output_device();
}

#scope_module
close_output_device :: () -> bool {
    status: s32;
    if output_handle status = audio_output_close(output_handle);

    if status != 0 {
        properties.initted = false;
        device_lost = true;
        device_error = status;
        device_notice |= 1;

        return false;
    }

    output_handle = null;
    properties = .{};
    minimum_bytes = 0;
    region_locked = false;
    locked_bytes = 0;
    restart_after_fill = false;
    device_paused = false;
    device_lost = false;
    device_notice = 0;

    return true;
} @Releases

#scope_export

#if OS == .WINDOWS || OS == .MACOS {
    get_bytes_between :: (front: $T, rear: T, buffer_len: T) -> T {
        assert(buffer_len > 0 && front >= 0 && rear >= 0 && front < buffer_len && rear < buffer_len);
        if front >= rear return front - rear;
        return buffer_len - rear + front;
    }
}

#scope_module
report_backend_notice :: () {
    lock_producer();
    notice := device_notice;
    error := device_error;

    device_notice = 0;
    unlock_producer();
    if (notice & 1) != 0 context.logger(tprint("Sound_Player: output unavailable (%); retrying\n", error), context.logger_data, Log_Info.{ common_flags = .ERROR });
    if (notice & 2) != 0 context.logger("Sound_Player: output restored\n", context.logger_data, Log_Info.{});
}

backend_can_fill :: () -> bool {
    return output_handle != null && !device_lost && !device_paused && !region_locked;
}

backend_current_rate :: () -> s64 {
    if !output_handle || device_lost return 0;
    return properties.output_sampling_rate;
}

output_channel_count :: () -> s32 {
    if properties.num_channels == 0 return 2;
    return properties.num_channels;
}

audio_wall_time :: () -> float64 {
    time := audio_output_clock();
    assert(time >= 0, "audio monotonic clock failed");
    return time;
}

start_output_if_possible :: () -> bool {
    return output_handle != null && audio_output_play(output_handle) == 0;
}

output_sampling_rate :: () -> s64 {
    return properties.output_sampling_rate;
}

#scope_file
native :: #library "jai_audio_output";

#if OS == .MACOS {
    #library,system,link_always "AudioToolbox";
    #library,system,link_always "CoreAudio";
    #library,system,link_always "CoreFoundation";
    #library,system,link_always "AppKit";
    #library,system,link_always "CoreGraphics";
}

#if OS == .WINDOWS {
    #library,system,link_always "winmm";
    #library,system,link_always "user32";
}

#if OS == .LINUX #library,system,link_always "asound";
audio_output_open :: (rate: s32, channels: s32, error: *s32) -> *void #foreign native;
audio_output_close :: (output: *void) -> s32 #foreign native;
audio_output_device :: (index: s32, name: *u8, capacity: s32) -> s32 #foreign native;
audio_output_buffered :: (output: *void) -> s64 #foreign native;
audio_output_submit :: (output: *void, bytes: *u8, count: s32) -> s32 #foreign native;
audio_output_play :: (output: *void) -> s32 #foreign native;
audio_output_clock :: () -> float64 #foreign native;
audio_output_rate :: (output: *void) -> s32 #foreign native;
audio_output_channels :: (output: *void) -> s32 #foreign native;
audio_output_preferred_channels :: () -> s32 #foreign native;
audio_output_capacity :: (output: *void) -> s64 #foreign native;
audio_output_poll :: (output: *void, allow_background: s32) -> s32 #foreign native;
audio_output_last_error :: (output: *void) -> s32 #foreign native;

#if OS == .LINUX native_rate: s32 : MY_SAMPLING_RATE;
else native_rate: s32 : OUTPUT_SAMPLING_RATE;
output_handle: *void;
properties: Backend_Properties;
minimum_bytes: s64;
channel_names := string.["front left", "front right", "center", "subwoofer", "rear left", "rear right", "extend left", "extend right"];
staging: [96000] u8;
region_locked := false;
locked_bytes: s64;
output_devices: [..] Output_Device;
device_names: [..] string;
device_snapshots: [..][] Output_Device;
backend_config: Sound_Player_Config;
next_device_poll: float64;
device_paused := false;
device_lost := false;
restart_after_fill := false;
device_notice: s32;
device_error: s32;

lose_output_device :: (error: s32) {
    device_error = error;
    if !device_lost device_notice |= 1;

    if region_locked {
        device_lost = true;
        return;
    }

    if output_handle {
        status := audio_output_close(output_handle);
        if status != 0 {
            device_lost = true;
            device_error = status;
            next_device_poll = audio_wall_time() + 0.25;
            return;
        }

        output_handle = null;
    }

    device_lost = false;
    next_device_poll = audio_wall_time() + 0.25;
    restart_after_fill = false;
}

service_output_device :: () {
    if !properties.initted || region_locked return;

    now := audio_wall_time();
    if now < next_device_poll return;

    next_device_poll = now + 0.25;

    if device_lost {
        lose_output_device(device_error);
        if output_handle return;
    }

    if !output_handle {
        next_device_poll = now + 0.25;
        error: s32;

        output_handle = audio_output_open(cast(s32) properties.output_sampling_rate, properties.num_channels, *error);
        if !output_handle {
            device_error = error;
            return;
        }

        restart_after_fill = true;
        device_notice |= 2;
    }

    state := audio_output_poll(output_handle, cast(s32) backend_config.audible_when_window_is_not_in_focus);

    device_paused = state == 1;
    if state == 2 lose_output_device(audio_output_last_error(output_handle));
}

count_output_bytes :: () -> s64, s64 {
    lock_producer();
    defer finish_backend_call();
    service_output_device();
    if device_lost return 0, minimum_bytes;
    if !output_handle return 0, minimum_bytes;

    bytes := audio_output_buffered(output_handle);
    if bytes < 0 {
        lose_output_device(cast(s32) bytes);
        return 0, minimum_bytes;
    }

    return bytes, minimum_bytes;
}
