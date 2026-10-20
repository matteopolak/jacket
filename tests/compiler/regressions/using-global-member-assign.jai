// Writing a member of a global struct through `using` is not writing an argument: the global's
// address is never compared with the argument slots (focus's color picker, whose struct has a
// `#type` field with parameter names that repeat the members).
// rules: proc.12
#import "Basic";
#import "Math";

dialog: struct {
    value: float;
    point: Vector2;

    rgba: Vector4;
    hsl: Vector3;

    on_change: #type (rgba: Vector4, hsl: Vector3, finalize: bool);
    data: *void;
}

set_color :: (color: Vector4, finalize := false) {
    using dialog;
    changed := rgba != color;

    rgba = color;
    hsl = color.xyz;
    assert(changed && !finalize);
}

main :: () {
    set_color(.{ 1, 2, 3, 4 });
    assert(dialog.rgba == .{ 1, 2, 3, 4 } && dialog.hsl == .{ 1, 2, 3 });
    print("ok\n");
}
