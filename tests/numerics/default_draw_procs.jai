// ui_init() without callbacks installs the Simp-backed drawing callbacks; explicit ones replace them.
#import "Basic";
G :: #import "GetRect";
L :: #import "GetRect_LeftHanded";

main :: () {
    assert(G.draw_procs.immediate_quad == null);
    G.ui_init();
    assert(G.draw_procs.immediate_quad != null && G.draw_procs.immediate_triangle != null);
    assert(G.draw_procs.get_font_at_size != null && G.draw_procs.texture_load_from_memory != null);
    assert(G.draw_procs.get_mouse_pointer_position != null && G.draw_procs.immediate_flush != null);

    L.ui_init();
    assert(L.draw_procs.immediate_quad != null);

    procs: G.Draw_Procs;

    G.ui_init(procs = *procs);
    assert(G.draw_procs.immediate_quad == null);
    print("default draw procs ok\n");
}
