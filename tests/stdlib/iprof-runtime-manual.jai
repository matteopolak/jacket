// Iprof runtime in MANUAL_MODE: hand-placed zones, per-frame update, report generation.
#import "Basic";
#import "Math";
Iprof :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = true);

work :: (n: int) -> int {
    Iprof.zone_begin(1);
    defer Iprof.zone_end();
    sum := 0;
    for 0..n - 1 sum += it * it;
    return sum;
}

main :: () {
    // Manual mode: the user fills in the zone table.
    Iprof.zones[0].name = "(global)";
    Iprof.zones[1].name = "work";
    Iprof.zones[1].hash = 12345;
    Iprof.zones.count; // fixed-size array in manual mode

    Iprof.init_runtime();

    total := 0;
    for frame: 0..9 {
        total += work(10000);
        Iprof.update(true);
    }
    assert(total > 0);

    report := Iprof.create_report();
    found := false;
    for report.records {
        if it.name == "work" {
            found = true;
            assert(it.calls > 0);
        }
    }
    assert(found);

    Iprof.set_report_mode(.CALL_COUNT);
    report = Iprof.create_report();
    assert(report.records.count > 0);
    Iprof.set_report_mode(.CALL_GRAPH);
    Iprof.select();
    report = Iprof.create_report();

    Iprof.set_report_mode(.SELF_TIME);
    report = Iprof.create_report();
    Iprof.log_text_report(report);

    // Drawing goes through callbacks; count what a renderer would receive.
    config: Iprof.Config;
    config.draw_text = (x: float, y: float, str: string, color: Vector4) {
        draws += 1;
    };
    config.text_width = (s: string) -> float {
        return cast(float) s.count * 8;
    };
    config.draw_rectangle = (x0: float, y0: float, x1: float, y1: float, color: Vector4) {
        draws += 1;
    };
    config.draw_line = (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color: Vector4) {
        draws += 1;
    };
    config.graph_begin = (x: float, y: float, w: float, h: float) {};
    config.graph_end = (x: float, y: float, w: float, h: float) {};
    Iprof.draw(10, 10, 400, 200, *config);
    Iprof.draw_graph(10, 400, 300, 100, *config);
    assert(draws > 0);

    print("ok\n");
}

draws := 0;
