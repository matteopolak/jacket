#if USE_GRAPHICS Graph_Scaling :: struct {
    target: s64;
    current: float;
    ease: float;
    output_factor: float64;
    SCALE_MAX :: 16;
}

GraphLocation :: struct {
    x0, y0: float;
    sx, sy: float;
}

NUM_THROWAWAY_UPDATES :: 3;
INT_ZERO_THRESHHOLD :: 0.25;
FRAME_TIME_INITIAL :: 0.001;

// Canonical module storage, shared by clients and loaded implementation files.
current_graph_window_selected_sample := -1;
cursor: s64;
display_frame: s64;
displayed_quantity := Report_Mode.SELF_TIME;
frame_time: History_Scalar;
graph_bigness_t_target: float;
history_index: s64;
injected_final_profiling_code := false;
instrumented_entry_point := false;
last_integer_timestamp: s64;
last_update_time: float64;
next_zone_under_cursor: *Zone;
num_active_zones: s64;
paused := false;
precomputed_factors: [NUM_SMOOTHING_SLOTS] float64;
prof_sum: s64;
speedstep_warning: bool;
times_to_reach_90_percent: [NUM_SMOOTHING_SLOTS] float64;
update_cursor := false;
update_index: s64;
zone_infos: [..] Zone_Info;
zone_under_cursor: *Zone;

set_report_mode :: (desired: Report_Mode) {
    displayed_quantity = desired;
}
