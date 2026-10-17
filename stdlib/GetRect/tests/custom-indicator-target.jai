// Target-availability reducer: custom rendering types do not replace Input.Event.
#import "Basic";
G :: #import "GetRect"(Type_Indicator = Renderer);

Renderer :: struct {
    Font :: struct {
        em_width: int;
        character_height: int;
        max_ascender: int;
        max_descender: int;
        y_offset_for_centering: int;
    }
    Font_Effects :: enum_flags u16 {
        OUTLINE;
    }
    Texture :: struct {
        caller_resource: u64;
    }
    Window_Type :: u64;
}

main :: () {
    rect := G.Rect.{ 0, 0, 10, 10 };
    assert(G.is_inside(5, 5, rect));
}
