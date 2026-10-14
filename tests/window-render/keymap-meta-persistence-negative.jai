#import "Basic";
K :: #import "Keymap";

press :: () {
    assert(false, "persistence must not execute binding callbacks");
}

main :: () {
    map: K.Keymap;

    K.add_proc(*map, "press", press);
    success, error := K.add_mapping(*map, .F1, .{ cmd_meta_pressed = true }, true, "press", true);

    assert(success && error == .NO_ERROR);
    builder: String_Builder;

    K.print_to_builder(*builder, *map);
}
