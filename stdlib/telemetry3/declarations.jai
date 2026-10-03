// Generated from docs/specs/stdlib/modules/telemetry3.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file

#if ENABLED && OS == .WINDOWS {
    tm_error :: enum u32 {
        TM_OK :: 0;
        TMERR_DISABLED :: 1;
        TMERR_INVALID_PARAM :: 2;
        TMERR_NULL_API :: 3;
        TMERR_OUT_OF_RESOURCES :: 4;
        TMERR_UNINITIALIZED :: 5;
        TMERR_BAD_HOSTNAME :: 6;
        TMERR_COULD_NOT_CONNECT :: 7;
        TMERR_UNKNOWN_NETWORK :: 8;
        TMERR_ALREADY_SHUTDOWN :: 9;
        TMERR_ARENA_TOO_SMALL :: 10;
        TMERR_BAD_HANDSHAKE :: 11;
        TMERR_UNALIGNED :: 12;
        TMERR_NETWORK_NOT_INITIALIZED :: 13;
        TMERR_BAD_VERSION :: 14;
        TMERR_BAD_TIMER :: 15;
        TMERR_ALREADY_OPENED :: 16;
        TMERR_ALREADY_INITIALIZED :: 17;
        TMERR_FILE_OPEN_FAILED :: 18;
        TMERR_INIT_NETWORKING_FAILED :: 19;
        TMERR_THREAD_CREATE_FAILED :: 20;
        TMERR_WRONG_PLATFORM :: 21;
        TMERR_UNKNOWN :: 0xFFFF;
    }

    tm_compression :: enum u32 {
        TMCOMPRESS_LZB :: 0;
        TMCOMPRESS_LZ4_DEPRECATED :: 1;
        TMCOMPRESS_NONE :: 2;
        TMCOMPRESS_UNKNOWN :: 0x7FFFFFFF;
    }

    tm_zone_flags :: enum u32 {
        TMZF_NONE :: 0;
        TMZF_STALL :: 1 << 1;
        TMZF_IDLE :: 1 << 2;
        TMZF_CONTEXT_SWITCH :: 1 << 3;
        TMZF_PLOT_TIME :: 1 << 4;
        TMZF_PLOT_TIME_EXPERIMENTAL :: TMZF_PLOT_TIME;
        TMZF_HAS_ACCUMULATION :: 1 << 5;
        TMZF_ACCUMULATION :: 1 << 6;
        TMZF_LOCK_HOLD :: 1 << 7;
        TMZF_LOCK_WAIT :: 1 << 8;
        TMZF_RECURSIVE :: 1 << 9;
        TMZF_NO_DURATION_LABEL :: 1 << 10;
        TMZF_KERNEL_SAMPLING_ZONE :: 1 << 11;
    }

    tm_open_flags :: enum u32 {
        TMOF_INIT_NETWORKING :: 1 << 0;
        TMOF_CAPTURE_CONTEXT_SWITCHES :: 1 << 1;
        TMOF_SAVE_PLOTS_CSV :: 1 << 3;
        TMOF_SAVE_ZONES_CSV :: 1 << 4;
        TMOF_SAVE_TIME_SPANS_CSV :: 1 << 5;
        TMOF_SAVE_MESSAGES_CSV :: 1 << 6;
        TMOF_GENERATE_HTML_REPORT :: 1 << 7;
        TMOF_BREAK_ON_MISMATCHED_ZONE :: 1 << 8;
        TMOF_SEND_ENTER_ZONE_EXPERIMENTAL :: 1 << 9;
        TMOF_SAVE_ALL_CSV :: TMOF_SAVE_PLOTS_CSV | TMOF_SAVE_ZONES_CSV | TMOF_SAVE_TIME_SPANS_CSV | TMOF_SAVE_MESSAGES_CSV;
    }

    tm_connection_type :: enum u32 {
        TMCT_TCP :: 0;
        TMCT_IPC :: 1;
        TMCT_FILE :: 2;
        TMCT_USER_PROVIDED :: 3;
    }

    tm_plot_draw_type :: enum u32 {
        TM_PLOT_DRAW_LINE :: 0;
        TM_PLOT_DRAW_POINTS :: 1;
        TM_PLOT_DRAW_STEPPED_XY :: 2;
        TM_PLOT_DRAW_STEPPED_YX :: 3;
        TM_PLOT_DRAW_COLUMN :: 4;
        TM_PLOT_DRAW_LOLLIPOP :: 5;
    }

    tm_plot_units :: enum u32 {
        TM_PLOT_UNITS_REAL :: 0;
        TM_PLOT_UNITS_MEMORY :: 1;
        TM_PLOT_UNITS_HEX :: 2;
        TM_PLOT_UNITS_INTEGER :: 3;
        TM_PLOT_UNITS_PERCENTAGE_COMPUTED :: 4;
        TM_PLOT_UNITS_PERCENTAGE_DIRECT :: 5;
        TM_PLOT_UNITS_TIME :: 6;
        TM_PLOT_UNITS_TIME_MS :: 7;
        TM_PLOT_UNITS_TIME_US :: 8;
        TM_PLOT_UNITS_TIME_CLOCKS :: 9;
        TM_PLOT_UNITS_TIME_INTERVAL :: 10;
    }

    tm_library_type :: enum u32 {
        TM_RELEASE :: 0;
        TM_DEBUG :: 1;
    }

    TM_API_NYI :: #type () -> void #c_call;

    tm_gpu_zone :: struct {
        name_id: u64;
        file_id: u64;
        line: u32;
        depth: u32;
    }

    tm_api :: struct #type_info_none {
        major_version: u32;
        minor_version: u32;
        revision_number: u32;
        build_number: u32;
        tmCheckVersion: (major_version: u32, minor_version: u32, revision_number: u32, build_number: u32) -> tm_error #c_call;
        tmInitialize: (arena_size: u32, arena_address: *u8) -> tm_error #c_call;
        tmOpen: (ctx: u32, program_name: *u8, build_info: *u8, host_or_filename: *u8,
                  connection_type: tm_connection_type, port: u16, open_flags: tm_open_flags, timeout_ms: s32) -> tm_error #c_call;
        tmClose: (ctx: u32) -> void #c_call;
        tmShutdown: () -> void #c_call;
        tmGetThreadHandle: () -> void #c_call;
        tmTick: (mask: u64) -> void #c_call;
        _tmRegisterFileCallbacks: TM_API_NYI;
        tmFlush: () -> void #c_call;
        tmMessage_Core: (mask: u64, flags: u32, filename: *u8, filename_id: *u64, line_number: u32, message: *u8, message_id: *u64) -> void #c_call;
        tmMessageV_Core: (mask: u64, flags: u32, filename: *u8, filename_id: *u64, line_number: u32, fmt: *u8, args: ..Any) -> void #c_call;
        tmString: (static_string: *u8) -> u64 #c_call;
        tmStaticString: (string_id: *u64, static_string: *u8) -> u64 #c_call;
        tmPrintf: (fmt: *u8, args: ..Any) -> u64 #c_call;
        _tmPrintfV: TM_API_NYI;
        tmCallStack: (mask: u64) -> u64 #c_call;
        tmZoneColor: (red: float, green: float, blue: float) -> void #c_call;
        _tmZoneWaitingFor: TM_API_NYI;
        tmEnterZone_Core: (mask: u64, flags: u32, file: *u8, file_id: *u64, line: u32, zone_name: *u8, zone_name_id: *u64) -> void #c_call;
        tmEnterZoneV_Core: (mask: u64, flags: u32, file: *u8, file_id: *u64, line: u32, zone_name: *u8, args: ..Any) -> void #c_call;
        tmLeaveZone: (mask: u64) -> void #c_call;
        _tmRenameZone_Core: TM_API_NYI;
        _tmRenameZoneV_Core: TM_API_NYI;
        _tmZoneAtTime_Core: TM_API_NYI;
        _tmZoneAtTimeV_Core: TM_API_NYI;
        _tmEmitAccumulationZone_Core: TM_API_NYI;
        _tmEmitAccumulationZoneV_Core: TM_API_NYI;
        _tmSetZoneFlag: TM_API_NYI;
        tmSetCaptureMask: (mask: u64) -> void #c_call;
        tmGetCaptureMask: () -> u64 #c_call;
        _tmGetSessionName: TM_API_NYI;
        _tmBeginTimeSpan_Core: TM_API_NYI;
        _tmBeginTimeSpanV_Core: TM_API_NYI;
        _tmEndTimeSpan_Core: TM_API_NYI;
        _tmSetTimeSpanName_Core: TM_API_NYI;
        _tmSetTimeSpanNameV_Core: TM_API_NYI;
        _tmStartWaitForLock_Core: TM_API_NYI;
        _tmStartWaitForLockV_Core: TM_API_NYI;
        _tmEndWaitForLock_Core: TM_API_NYI;
        _tmAcquiredLock_Core: TM_API_NYI;
        _tmAcquiredLockV_Core: TM_API_NYI;
        _tmReleasedLock_Core: TM_API_NYI;
        tmPlot_Core: (mask: u64, units: tm_plot_units, draw_type: tm_plot_draw_type, plot_value: float64, plot_name: *u8, plot_name_id: *u64) -> void #c_call;
        tmPlotAt_Core: (mask: u64, ts: u64, units: tm_plot_units, draw_type: tm_plot_draw_type, plot_value: float64, plot_name: *u8, plot_name_id: *u64) -> void #c_call;
        tmSetPlotInfo: (plot_name: u64, draw_type: tm_plot_draw_type, units: tm_plot_units) -> void #c_call;
        _tmAlloc_Core: TM_API_NYI;
        _tmAllocV_Core: TM_API_NYI;
        _tmFree_Core: TM_API_NYI;
        tmThreadName: (mask: u64, thread_id: u32, fmt: *u8, args: ..Any) -> void #c_call;
        tmEndThread: (mask: u64, thread_id: u32) -> void #c_call;
        tmTrackOrder: (mask: u64, track_id: u32, track_order: u64) -> void #c_call;
        tmTrackColor: (mask: u64, track_id: u32, r: float32, g: float32, b: float32) -> void #c_call;
        tmHash: (str: *u8, len: s32) -> u64 #c_call;
        tmHashString: (str: *u8) -> u64 #c_call;
        tmHash32: (str: *u8, len: s32) -> u32 #c_call;
        _tmSetMaxThreadCount: TM_API_NYI;
        _tmGetMaxThreadCount: TM_API_NYI;
        _tmSetMaxTimeSpanTrackCount: TM_API_NYI;
        _tmGetMaxTimeSpanTrackCount: TM_API_NYI;
        _tmSetSendBufferSize: TM_API_NYI;
        _tmGetSendBufferSize: TM_API_NYI;
        _tmSetSendBufferCount: TM_API_NYI;
        _tmGetSendBufferCount: TM_API_NYI;
        _tmGetMemoryFootprint: TM_API_NYI;
        tmRunning: () -> u32 #c_call;
        _tmSetCompression: TM_API_NYI;
        _tmSetZoneFilterThreshold: TM_API_NYI;
        tmNewTrackID: () -> u32 #c_call;
        tmTrackName: (mask: u64, track_id: u32, fmt: *u8) -> void #c_call;
        _tmThreadTrack: TM_API_NYI;
        _tmSwitchToFiber: TM_API_NYI;
        _tmEndFiber: TM_API_NYI;
        tmEnterZoneFast_Core: (mask: u64, flags: u32, file_id: u64, line: u32, zone_id: u64) -> void #c_call;
        tmGetCurrentThreadId: () -> u64 #c_call;
        _tmZoneColorSticky: TM_API_NYI;
        _tmSwitchToFiberOnTrack: TM_API_NYI;
        _tmGetZoneDepth: TM_API_NYI;
        tmSecondsToTicks: (seconds: float64) -> u64 #c_call;
        _tmBeginColoredTimeSpan_Core: TM_API_NYI;
        _tmBeginColoredTimeSpanV_Core: TM_API_NYI;
        _tmNewTimeSpanTrackID: TM_API_NYI;
        _tmScreenshot: TM_API_NYI;
        _tmTickAtTime: TM_API_NYI;
        _tmContextSwitchAtTime: TM_API_NYI;
        _tmTimeSpanAtTime_Core: TM_API_NYI;
        _tmTimeSpanAtTimeV_Core: TM_API_NYI;
        _tmMessageAtTime_Core: TM_API_NYI;
        _tmMessageAtTimeV_Core: TM_API_NYI;
        _tmMemZoneAtTime: TM_API_NYI;
        _tmAllocAtTime_Core: TM_API_NYI;
        _tmAllocAtTimeV_Core: TM_API_NYI;
        _tmFreeAtTime_Core: TM_API_NYI;
        tmProfileThread: (mask: u64, thread_id: u32, flags: u32) -> void #c_call;
        tmSetSamplingInterval: (microseconds: u32) -> void #c_call;
        tmGpuZone: (zone: *tm_gpu_zone, track: u32,
                     gpu_enter: u64, gpu_leave: u64, cpu_time: u64, fast_time: u64,
                     gpu_begin: u64, cpu_begin: u64, fast_begin: u64,
                     gpu_freq: u64, cpu_freq: u64) -> void #c_call;
        _tmBitmapRGB: TM_API_NYI;
        _tmBitmapPlanes: TM_API_NYI;
        _tmBitmapStart: TM_API_NYI;
        _tmBitmapEnd: TM_API_NYI;
        _tmBitmapSendRGB: TM_API_NYI;
        _tmBitmapSendPlane: TM_API_NYI;
        _tmSetThreadStackSize: TM_API_NYI;
        _tmGetThreadStackSize: TM_API_NYI;
        _tmSetMaxTweakCount: TM_API_NYI;
        _tmGetMaxTweakCount: TM_API_NYI;
        _tmTweakVar_Core: TM_API_NYI;
        _tmRegisterTweakCallback: TM_API_NYI;
        _tmGetZoneIDPtr: TM_API_NYI;
        _tmDependsOnPtr: TM_API_NYI;
        _tmAddDependencyPtr: TM_API_NYI;
        connected: s32;
        error: tm_error;
    }
}

#scope_export

// Aliases and constants

#if !ENABLED || OS != .WINDOWS {
    ENABLE_TELEMETRY :: 0;
}

#if ENABLED && OS == .WINDOWS {
    ENABLE_TELEMETRY :: 1;
}

#if ENABLED && OS == .WINDOWS {
    TM_API_BUILD_NUMBER :: 51766;
}

#if ENABLED && OS == .WINDOWS {
    TM_API_MAJOR_VERSION :: 2022;
}

#if ENABLED && OS == .WINDOWS {
    TM_API_MINOR_VERSION :: 5;
}

#if ENABLED && OS == .WINDOWS {
    TM_API_REVISION_NUMBER :: 23;
}

#if ENABLED && OS == .WINDOWS {
    TM_API_VERSION :: "2022.05.23.51766";
}

#if ENABLED && OS == .WINDOWS {
    TM_BUILD_DATE :: "2022.09.06.53968";
}

#if ENABLED && OS == .WINDOWS {
    TM_BUILD_DAY :: 6;
}

#if ENABLED && OS == .WINDOWS {
    TM_BUILD_MONTH :: 9;
}

#if ENABLED && OS == .WINDOWS {
    TM_BUILD_SECOND :: 53968;
}

#if ENABLED && OS == .WINDOWS {
    TM_BUILD_YEAR :: 2022;
}

#if ENABLED && OS == .WINDOWS {
    TM_SDK_BUILD_NUMBER :: 147;
}

#if ENABLED && OS == .WINDOWS {
    TM_SDK_MAJOR_VERSION :: 3;
}

#if ENABLED && OS == .WINDOWS {
    TM_SDK_MINOR_VERSION :: 5;
}

#if ENABLED && OS == .WINDOWS {
    TM_SDK_REVISION_NUMBER :: 0;
}

#if ENABLED && OS == .WINDOWS {
    TM_SDK_VERSION :: "3.5.0.147";
}

#if ENABLED && OS == .WINDOWS {
    TmBuildDate :: 2022_09_06_53968;
}

#if ENABLED && OS == .WINDOWS {
    TmBuildVersion :: 03_5_000_147;
}

#if ENABLED && OS == .WINDOWS {
    W :: #import "Windows";
}

// Native storage

#if ENABLED && OS == .WINDOWS {
    tm_program_name := "";
}

#if ENABLED && OS == .WINDOWS {
    tm_build_info := "";
}

#if ENABLED && OS == .WINDOWS {
    g_tm_api: *tm_api = null;
}

#if ENABLED && OS == .WINDOWS {
    g_tm_api0: *tm_api = null;
}

// Convenience operations

#if !ENABLED || OS != .WINDOWS {
    telemetry_add_gpuzone :: (name: string, frame_number: u32, level: u32, gpu_enter: u64, gpu_leave: u64) #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_enabled :: () -> bool #expand {
        return false;
    }
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_end_gpuzone :: () #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_enterzone :: (name: string, loc := #caller_location, is_dynamic := false) #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_init :: (program_name: string, build_info: string, output_file := "") #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_leavezone :: () #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_message :: (message: string) #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_plot :: (plot_name: string, plot_value: float64) #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_setthreadname :: (name: string) #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_shutdown :: () #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_start :: (frames := 0) #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_start_gpuzone :: (frame_number: u32, gpu_freq: u64, cpu_freq: u64, gpu_begin: u64, cpu_begin: u64, fast_begin: u64) #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_stop :: () #expand {}
}

#if !ENABLED || OS != .WINDOWS {
    telemetry_tick :: () #expand {}
}
