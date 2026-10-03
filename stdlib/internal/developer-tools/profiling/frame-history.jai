SPEEDSTEP_DETECTION_RATIO: float64 = 0.08;

Prof_init_highlevel :: () {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    if profile_highlevel_ready return;

    init_runtime();
    push_allocator(profile_owner);
    times_to_reach_90_percent = .[0.1, 0.8, 2.5];
    eternity_set(*frame_time, FRAME_TIME_INITIAL);

    profile_highlevel_ready = true;
}

update :: (using s: *History_Scalar, new_value: float64, k_array: [] float64) {
    assert(s != null && k_array.count == NUM_SMOOTHING_SLOTS);

    for k, i: k_array {
        assert(k >= 0 && k <= 1, "Iprof: smoothing factor outside [0,1]");
        s.variances[i] = k * s.variances[i] + (1 - k) * new_value * new_value;
        s.values[i] = k * s.values[i] + (1 - k) * new_value;
    }

    #if DO_HISTORY s.history[history_index] = cast(float) new_value;
}

eternity_set :: (using s: *History_Scalar, new_value: float64) {
    assert(s != null);
    for *value: s.values value.* = new_value;
    for *moment: s.variances moment.* = new_value * new_value;
    #if DO_HISTORY s.history[history_index] = cast(float) new_value;
}

clear_stack :: (using c: *Zone_Profiling_Data) {
    assert(c != null);
    c.total_self_ticks = 0;
    c.total_hier_ticks = 0;
    c.total_entry_count = 0;
}

sum_times :: (c: *Zone_Profiling_Data) {
    assert(c != null && c.total_self_ticks >= 0);
    prof_sum += c.total_self_ticks;
}

update_history :: (c: *Zone_Profiling_Data) {
    assert(c != null && c.zone != null);

    if !c.profile_tracker_data_record {
        push_allocator(profile_owner);
        c.profile_tracker_data_record = cast(*Profile_Tracker_Data_Record) Profile_Pool.get(*profile_pool, size_of(Profile_Tracker_Data_Record));
        assert(c.profile_tracker_data_record != null, "Iprof: tracker pool allocation failed");
        c.profile_tracker_data_record.* = .{};
    }

    tracker := c.profile_tracker_data_record;

    if update_index < NUM_THROWAWAY_UPDATES {
        eternity_set(*tracker.self_time, cast(float64) c.total_self_ticks * timestamps_to_seconds);
        eternity_set(*tracker.hierarchical_time, cast(float64) c.total_hier_ticks * timestamps_to_seconds);
        eternity_set(*tracker.entry_count, cast(float64) c.total_entry_count);
    } else {
        update(*tracker.self_time, cast(float64) c.total_self_ticks * timestamps_to_seconds, precomputed_factors);
        update(*tracker.hierarchical_time, cast(float64) c.total_hier_ticks * timestamps_to_seconds, precomputed_factors);
        update(*tracker.entry_count, cast(float64) c.total_entry_count, precomputed_factors);
    }

    tracker.max_recursion = max(tracker.max_recursion, cast(s64) c.recursion_depth);
}

propagate_stack :: (c: *Zone_Profiling_Data) {
    assert(c != null);
    ancestor := c;
    while ancestor && ancestor.zone {
        probe := c;
        duplicate := false;

        while probe != ancestor {
            if probe.zone == ancestor.zone {
                duplicate = true;
                break;
            }

            probe = probe.parent;
        }

        if !duplicate ancestor.total_hier_ticks += c.total_self_ticks;

        ancestor = ancestor.parent;
    }
}

update :: (record_data: bool) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    Prof_init_highlevel();
    now := profile_now();

    assert(now >= last_integer_timestamp && stack != null);
    stack.total_self_ticks += now - stack.t_self_start;
    stack.t_self_start = now;

    for *activation: profile_activations {
        if it_index >= profile_activation_count break;
        activation.node.total_hier_ticks += now - activation.hierarchical_start;
        activation.hierarchical_start = now;
    }

    for c: zone_hash if c c.total_hier_ticks = 0;

    traverse(propagate_stack);
    prof_sum = 0;
    traverse(sum_times);
    elapsed := cast(float64) (now - last_integer_timestamp) * profile_nominal_seconds;
    counter_delta := ifx profile_calibration_count == 0 then max(cast(s64) 1, prof_sum) else now - last_integer_timestamp;

    if elapsed > 0 {
        rate := cast(float64) counter_delta / elapsed;
        if profile_calibration_count < NUM_THROWAWAY_UPDATES eternity_set(*integer_timestamps_per_second, rate);
        else update(*integer_timestamps_per_second, rate, precomputed_factors);

        timestamps_to_seconds = 1 / get_value(*integer_timestamps_per_second);
        speedstep_warning = compute_heat(integer_timestamps_per_second.variances[1], integer_timestamps_per_second.values[1]) > SPEEDSTEP_DETECTION_RATIO;
    }

    profile_calibration_count += 1;
    last_integer_timestamp = now;
    last_update_time = cast(float64) now * profile_nominal_seconds;

    if record_data && !paused {
        for response, i: times_to_reach_90_percent {
            precomputed_factors[i] = ifx update_index < NUM_THROWAWAY_UPDATES then 0 else Profile_Math.pow(0.1, elapsed / response);
        }

        if update_index < NUM_THROWAWAY_UPDATES eternity_set(*frame_time, elapsed);
        else update(*frame_time, elapsed, precomputed_factors);

        traverse(update_history);
        profile_write_zone_history();
        update_index += 1;
        history_index = (history_index + 1) % NUM_FRAME_SLOTS;
    }

    traverse(clear_stack);
    clear_stack(*dummy);
}

#scope_module
#import "Basic";
Profile_Math :: #import "Math";

profile_highlevel_ready: bool;
profile_nominal_seconds: float64;
profile_calibration_count: s64;

profile_write_zone_history :: () {
    #if DO_HISTORY {
        for c: zone_hash if c && c.zone {
            c.zone.history_self.values[history_index] = 0;
            c.zone.history_hier.values[history_index] = 0;
            c.zone.history_calls.values[history_index] = 0;
        }

        for c: zone_hash if c && c.zone && c.profile_tracker_data_record {
            t := c.profile_tracker_data_record;
            c.zone.history_self.values[history_index] += t.self_time.history[history_index];
            c.zone.history_hier.values[history_index] += t.hierarchical_time.history[history_index];
            c.zone.history_calls.values[history_index] += t.entry_count.history[history_index];
        }
    }
}

profile_history_value :: (s: *History_Scalar) -> float64 {
    if display_frame == 0 return get_value(s);
    position := (history_index + NUM_FRAME_SLOTS - 1 - display_frame) % NUM_FRAME_SLOTS;
    return cast(float64) s.history[position];
}
