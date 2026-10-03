// Independently authored contract for the reviewed ImGui 1.90.4 docking source.
// Link only a fresh archive produced by tools/native_sdk_proofs.py.
create_context :: (atlas: *void) -> *void #foreign "_ZN5ImGui13CreateContextEP11ImFontAtlas";
destroy_context :: (ctx: *void) #foreign "_ZN5ImGui14DestroyContextEP12ImGuiContext";
get_version :: () -> *u8 #foreign "_ZN5ImGui10GetVersionEv";

main :: () -> int {
    ctx := create_context(null);
    if ctx == null return 1;
    version := get_version();
    if version == null return 2;
    if version[0] != 49 || version[1] != 46 || version[2] != 57 return 3;
    if version[3] != 48 || version[4] != 46 || version[5] != 52 || version[6] != 0 return 4;
    destroy_context(ctx);
    return 42;
}
