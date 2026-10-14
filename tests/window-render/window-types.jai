#import "Basic";
W :: #import "Window_Creation";

main :: () {
    #if OS == .WINDOWS {
        #assert size_of(W.Saved_Window_Info) == 24;
        #assert offset_of(W.Saved_Window_Info.style) == 16;
        #assert offset_of(W.Saved_Window_Info.ex_style) == 20;
        #assert W.WINDOWED_STYLE == 0x00cf0000;
        #assert W.FULLSCREEN_STYLE == 0x90000000;
    } else #if OS == .LINUX {
        #assert size_of(W.Saved_Window_Info) == 0;
    }

    print("ok\n");
}
