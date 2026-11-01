// Pure timeline layout and per-zone statistics; UI-independent and nanosecond-based.
#import "Basic";
#import "String";
Trace :: #import "Extensions/Iprof_Trace";

Sort_Column :: enum {
    NAME;
    CALLS;
    SELF;
    TOTAL;
    AVERAGE;
    MAXIMUM;
}

Stat :: struct {
    zone: s64;
    calls: s64;
    self_ns, total, maximum: u64;
    average: float64;
}

Flame_Box :: struct {
    span: s64;
    depth: s64;
    x, width: float64;
}

// Boxes are fractions of the frame width, with one row per call depth.
flame_layout :: (frame: *Trace.Frame) -> [] Flame_Box {
    result := NewArray(frame.spans.count, Flame_Box);

    for span, i: frame.spans {
        depth := ifx span.parent >= 0 then result[span.parent].depth + 1 else 0;

        result[i] = .{ span = i, depth = depth };

        if frame.duration > 0 {
            result[i].x = cast(float64) span.start / cast(float64) frame.duration;
            result[i].width = cast(float64) (span.end - span.start) / cast(float64) frame.duration;
        }
    }

    return result;
}

// Inclusive totals count every invocation, including recursion. Self subtracts direct children.
statistics :: (trace: *Trace.Trace, frame: s64, search: string = "", column: Sort_Column = .TOTAL, descending := true, include_empty := false) -> [] Stat {
    all := NewArray(trace.names.count, Stat);

    defer array_free(all);
    for *all it.zone = it_index;

    if frame >= 0 && frame < trace.frames.count {
        current := *trace.frames[frame];
        for current.spans {
            elapsed := it.end - it.start;
            stat := *all[it.zone];
            stat.calls += 1;
            stat.total += elapsed;
            stat.self_ns += elapsed;
            stat.maximum = max(stat.maximum, elapsed);
        }

        for current.spans if it.parent >= 0 {
            parent := current.spans[it.parent];
            all[parent.zone].self_ns -= it.end - it.start;
        }
    }

    result: [..] Stat;

    for *all if it.calls > 0 it.average = cast(float64) it.total / cast(float64) it.calls;

    for stat: all if (stat.calls > 0 || include_empty) && (search.count == 0 || contains(trace.names[stat.zone], search)) {
        array_add(*result, stat);
    }

    // Insertion sort keeps ties deterministic; a trace has few distinct zones.
    for i: 1..result.count - 1 {
        j := i;
        while j > 0 && before(trace, result[j], result[j - 1], column, descending) {
            swap(*result[j], *result[j - 1]);
            j -= 1;
        }
    }

    return result;
}

// Fractional width of each frame bar; heights are relative to the longest frame.
frame_height :: (trace: *Trace.Trace, index: s64) -> float64 {
    maximum: u64 = 1;
    for trace.frames maximum = max(maximum, it.duration);
    return cast(float64) trace.frames[index].duration / cast(float64) maximum;
}

#scope_file
before :: (trace: *Trace.Trace, a: Stat, b: Stat, column: Sort_Column, descending: bool) -> bool {
    if column == .NAME {
        order := compare(trace.names[a.zone], trace.names[b.zone]);
        return ifx descending then order > 0 else order < 0;
    }

    av, bv: float64;

    if column == .CALLS {
        av = cast(float64) a.calls;
        bv = cast(float64) b.calls;
    } else if column == .SELF {
        av = cast(float64) a.self_ns;
        bv = cast(float64) b.self_ns;
    } else if column == .TOTAL {
        av = cast(float64) a.total;
        bv = cast(float64) b.total;
    } else if column == .AVERAGE {
        av = a.average;
        bv = b.average;
    } else {
        av = cast(float64) a.maximum;
        bv = cast(float64) b.maximum;
    }

    return ifx descending then av > bv else av < bv;
}
