#import "Basic";
M :: #import "Gamepad/SDL_Input_Map";

main :: () {
    assert(M.input_name_to_input_target("leftx") == .LEFTX);
    assert(M.input_name_to_input_target("no-such-input") == .UNINITIALIZED);
    guid := M.get_guid_string(3, 0x1234, 0x5678, 0x101);

    assert(guid == "03000000341200007856000001010000");
    map, success := M.parse_gamepad_mapping_string("03000000341200007856000001010000,Worker Test Pad,a:b2,leftx:a0,dpup:h0.1,platform:Mac OS X,");

    assert(success);
    assert(map.name == "Worker Test Pad" && map.platform == "Mac OS X");
    assert(map.mappings.count == 3);
    assert(map.mappings[0].target == .A && map.mappings[0].type == .BUTTON && map.mappings[0].index == 2);
    assert(map.mappings[1].type == .AXIS && map.mappings[1].index == 0);
    assert(map.mappings[2].type == .HAT && map.mappings[2].subindex == 1);
    M.deinit(map);
    malformed := string.[
        "", "guid-only", "guid,name,unknown:b1", "guid,name,a:b-1",
        "guid,name,a:b2147483648", "guid,name,a:h0", "guid,name,a:b1junk",
        "guid,name,leftx:+a1", "guid,name,leftx:a1~", "guid,name,a:"
    ];
    for text: malformed {
        failed, ok := M.parse_gamepad_mapping_string(text);
        assert(!ok);
        assert(failed.guid.data == null && failed.name.data == null);
        assert(failed.mappings.data == null);
    }

    print("ok\n");
}
