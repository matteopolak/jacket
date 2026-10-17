#import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
H :: #import "Extensions/Http";

plain :: () -> string {
    return "plain";
}

by_id :: (id: H.Path("id", int)) -> string {
    return tprint("%", id.value);
}

by_other :: (other: H.Path("other", int)) -> string {
    return tprint("%", other.value);
}

tail :: (path: H.Rest("path")) -> string {
    return path.value;
}

around :: (request: *H.Request, next: H.Next) -> H.Response {
    return next(request);
}

main :: () {
    #assert size_of(H.Group("")) == size_of(H.Group("/admin"));
    #assert align_of(H.Group("")) == align_of(H.Group("/admin/v1"));
    allocations_before_this_call_will_not_be_considered_leaks();

    for 0..3 {
        app: H.App;

        H.get(*app, "/top/fixed", plain);
        H.get(*app, "/{id}", by_id);
        H.get(*app, "/{path...}", tail);
        H.get(*app, "/items/{id}", by_id);
        H.get(*app, "/files/{path...}", tail);
        first := H.group(*app, "/admin", around);

        H.get(first, "/items/{id}", by_id);
        second := H.group(first, "/v1", around);

        H.get(second, "/items/{id}", by_id);
        H.get(first, "/items/{other}", by_other);
        assert(app.setup_errors.count == 1);
        assert(app.router.routes.count == 7);
        H.free_app(*app);
        assert(app.router.routes.count == 0 && app.groups.count == 0);
        assert(app.router.root.param == null && app.router.root.rest == null);
    }

    report := make_leak_report(.{ scan_global_memory_to_remove_leaks = false, scan_leaks_to_combine = false });
    count: s64;
    bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.count;
        bytes += summary.bytes;
    }

    print("leaks % bytes %\n", count, bytes);
    deinit(*report);
    assert(count == 0 && bytes == 0);
    print("ok\n");
}
