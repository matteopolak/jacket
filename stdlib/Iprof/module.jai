#module_parameters(IMPORT_MODE := Import_Mode.METAPROGRAM, MANUAL_MODE := false, USE_GRAPHICS := true, DO_HISTORY := true);

#scope_module
Profile_Context :: #import "internal/developer-tools/profiling/context";
#scope_export

Import_Mode :: enum u8 {
    METAPROGRAM :: 0;
    CLIENT :: 1;
}

NUM_VALUES :: 4;
NUM_TITLE :: 2;
NUM_HEADER :: NUM_VALUES + 1;
NUM_FRAME_SLOTS :: 128;

Report_Mode :: enum u8 {
    SELF_TIME;
    HIERARCHICAL_TIME;
    CALL_COUNT;
    CALL_GRAPH;
}

Report :: struct {
    title: string;
    title_right: string;
    header: [NUM_HEADER] string;
    hilit_header: s32 = 0;
    records: [..] Report_Row;
    hilight: s64;
}

Report_Row :: struct {
    indent: s64;
    name: string;
    number: s64;
    prefix: u8;
    value_flag: s64;
    self_time: float64;
    hier_time: float64;
    calls: float64;
    #overlay (self_time) array_aliased_values: [3] float64;
    heat: float64;
    zone: *Zone;
}

Zone_Info :: struct {
    name: string;
    load: *Iprof_Compiler.Message_File;
    line: s64;
}

#if DO_HISTORY Zone_History :: struct {
    values: [NUM_FRAME_SLOTS] float32;
}

Zone :: struct {
    name: string;
    hash: u64;
    filename: string;
    full_path: string;
    line: s64;
    highlevel: *Report_Row;
    #if DO_HISTORY {
        history_self: Zone_History;
        history_hier: Zone_History;
        history_calls: Zone_History;
    }
    visited := false;
    initted := false;
}

#load "../internal/developer-tools/iprof-report.jai";
#load "../internal/developer-tools/iprof-history-values.jai";
#load "../internal/developer-tools/iprof-public-state.jai";
#load "../internal/developer-tools/iprof-stack-inspection.jai";
#load "../internal/developer-tools/profiling/runtime-clock.jai";
#load "../internal/developer-tools/profiling/runtime-zones.jai";
#load "../internal/developer-tools/profiling/client-setup.jai";

#load "../internal/developer-tools/profiling/runtime-traversal.jai";
#load "../internal/developer-tools/profiling/frame-history.jai";
#load "../internal/developer-tools/profiling/frame-report.jai";
#load "../internal/developer-tools/profiling/automatic-zones.jai";
#if IMPORT_MODE == .METAPROGRAM #load "../internal/developer-tools/profiling/instrumentation.jai";
#if USE_GRAPHICS #load "../internal/developer-tools/profiling/graphics.jai";

#scope_file
Iprof_Compiler :: #import "Compiler";
