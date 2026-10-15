#import "Basic";
P :: #import "POSIX";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true);
M :: #import "Math";
No_History :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, DO_HISTORY = false);

texts, rectangles, lines, begins, ends: s64;
diagnostics: s64;

capture :: (text: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR diagnostics += 1;
}

text :: (x: float, y: float, str: string, color: M.Vector4) {
    assert(str.count > 0);
    texts += 1;
}

width :: (s: string) -> float {
    return cast(float) s.count * 8;
}

rectangle :: (x0: float, y0: float, x1: float, y1: float, color: M.Vector4) {
    assert(x1 >= x0 && y1 >= y0);
    rectangles += 1;
}

line :: (p0: M.Vector2, p1: M.Vector2, p2: M.Vector2, p3: M.Vector2, color: M.Vector4) {
    lines += 1;
}

begin :: (x: float, y: float, width: float, height: float) {
    begins += 1;
}

end :: (x: float, y: float, width: float, height: float) {
    ends += 1;
}

main :: () {
    I.zones[0].name = "measured graph";
    I.zones[0].initted = true;
    I.Prof_init_highlevel();

    for 0..3 {
        I.zone_begin(0);
        I.zone_end();
        I.update(true);
    }

    config: I.Config;

    assert(config.line_spacing == 8 && config.font_character_height == 8 && config.zone_time_precision_digits == 2);

    push_context {
        context.logger = capture;
        assert(config.text_width("unset width") == 0);
    }

    assert(diagnostics == 1);
    config.draw_text = text;
    config.text_width = width;
    config.draw_rectangle = rectangle;
    config.draw_line = line;
    config.graph_begin = begin;
    config.graph_end = end;
    I.draw(0, 0, 600, 300, *config);
    I.draw_graph(0, 0, 600, 300, *config);
    assert(texts > 0 && rectangles > 0 && lines > 0 && begins == 1 && ends == 1);
    self := I.get_current_graph_scaling();

    assert(self.current == 0 && self.output_factor == 1);
    self.target = 2;
    duration: P.timespec;
    duration.tv_nsec = 5_000_000;
    assert(P.nanosleep(*duration, null) == 0);
    I.draw_graph(0, 0, 600, 300, *config);
    assert(self.current > 0 && self.current < 2 && self.output_factor > 0.25 && self.output_factor < 1);

    I.set_report_mode(.CALL_COUNT);
    hierarchical := I.get_current_graph_scaling();

    assert(self != hierarchical);
    I.current_graph_window_selected_sample = 1;
    I.zone_under_cursor = *I.zones[0];
    I.graph_select_sample();
    assert(I.displayed_quantity == .CALL_GRAPH && I.display_frame >= 0 && I.display_frame < 128);
    No_History.zones[0].name = "current data without graph history";
    No_History.zones[0].initted = true;
    No_History.Prof_init_highlevel();
    No_History.zone_begin(0);
    No_History.zone_end();
    No_History.update(true);
    other: No_History.Config;
    other.draw_text = text;
    other.text_width = width;
    other.draw_rectangle = rectangle;
    other.draw_line = line;
    other.graph_begin = begin;
    other.graph_end = end;
    prior_lines := lines;
    prior_texts := texts;

    No_History.draw_graph(0, 0, 600, 300, *other);
    assert(lines == prior_lines && texts == prior_texts + 1);
    No_History.draw(0, 0, 600, 300, *other);
    assert(texts > prior_texts + 1);
    print("ok\n");
}
