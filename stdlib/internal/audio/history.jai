advance_history_cursor :: () {
    history_cursor = (history_cursor + 1) % history_intervals.count;
    history_intervals[history_cursor] = .{};
    history_partial_frames = 0;
}

#scope_module
record_history_sample :: (channel: s64, sample: float) {
    interval := *history_intervals[history_cursor];

    if history_partial_frames == 0 {
        interval.minimum[channel] = sample;
        interval.maximum[channel] = sample;
    } else {
        interval.minimum[channel] = min(interval.minimum[channel], sample);
        interval.maximum[channel] = max(interval.maximum[channel], sample);
    }
}

finish_history_frame :: () {
    history_partial_frames += 1;
    if history_partial_frames == 1024 advance_history_cursor();
}

#scope_file
History_Interval :: struct {
    minimum: [OUTPUT_CHANNELS_MAX] float;
    maximum: [OUTPUT_CHANNELS_MAX] float;
}

history_intervals: [64] History_Interval;
history_cursor: s64;
history_partial_frames: s64;
