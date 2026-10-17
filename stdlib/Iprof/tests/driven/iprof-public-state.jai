#import "Basic";
I :: #import "Iprof"(USE_GRAPHICS = false);
Alias :: #import "Iprof"(USE_GRAPHICS = false);
Other :: #import "Iprof"(USE_GRAPHICS = false, DO_HISTORY = false);

G :: #import "Iprof"(IMPORT_MODE = .CLIENT, USE_GRAPHICS = true);

main :: () {
    assert(I.NUM_THROWAWAY_UPDATES == 3);
    assert(I.INT_ZERO_THRESHHOLD == 0.25 && I.FRAME_TIME_INITIAL == 0.001);
    scale: G.Graph_Scaling;
    location: I.GraphLocation;

    assert(G.Graph_Scaling.SCALE_MAX == 16);
    assert(type_of(scale.output_factor) == float64 && type_of(scale.target) == s64);
    assert(type_of(location.x0) == float && type_of(location.sy) == float);
    assert(I.current_graph_window_selected_sample == -1 && I.displayed_quantity == .SELF_TIME);
    assert(!I.paused && !I.update_cursor && I.zone_infos.count == 0);
    assert(*I.cursor == *Alias.cursor && *I.frame_time == *Alias.frame_time);
    I.cursor = 19;
    assert(Alias.cursor == 19 && Other.cursor == 0);
    assert(*I.cursor != *Other.cursor);
    I.frame_time.values[1] = 0.125;
    assert(Alias.get_value(*Alias.frame_time) == 0.125);
    assert(Other.get_value(*Other.frame_time) == 0);
    I.set_report_mode(.CALL_GRAPH);
    assert(Alias.displayed_quantity == .CALL_GRAPH && Other.displayed_quantity == .SELF_TIME);
    I.set_report_mode(.HIERARCHICAL_TIME);
    assert(Alias.displayed_quantity == .HIERARCHICAL_TIME);
    I.set_report_mode(.CALL_COUNT);
    assert(Alias.displayed_quantity == .CALL_COUNT);
    I.set_report_mode(.SELF_TIME);
    assert(Alias.displayed_quantity == .SELF_TIME);
    I.paused = true;
    assert(Alias.paused && !Other.paused);
    info: I.Zone_Info;
    info.name = "original public state zone";
    array_add(*I.zone_infos, info);
    assert(Alias.zone_infos.count == 1 && Alias.zone_infos[0].name == info.name);
    array_free(I.zone_infos);
    print("ok\n");
}
