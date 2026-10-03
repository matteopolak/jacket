// Authored receiver/reference fixture; this is not engine acceptance.
Vec2 :: struct { x:float32; y:float32; }
prepare :: () -> *void #foreign "jai_imgui_prepare";
finish :: () -> s32 #foreign "jai_imgui_finish";
add_rect :: (this:*void,p_min:*Vec2,p_max:*Vec2,col:u32,rounding:float32=0.0,flags:s32=0) #cpp_method #foreign "_ZN10ImDrawList13AddRectFilledERK6ImVec2S2_jfi";
main :: () -> int {
    draw_list := prepare();
    minimum:Vec2 = .{10.0,10.0};
    maximum:Vec2 = .{52.0,52.0};
    add_rect(draw_list,*minimum,*maximum,4294967295);
    return finish();
}
