// Simp: original-API names added for source compatibility (fonts, coordinate system, immediate vertices).
#import "Basic";
#import "Math";
#import "File";
Simp :: #import "Simp";

main :: () {
    // Pure helpers.
    assert(Simp.isdigit(#char "7") && !Simp.isdigit(#char "x"));
    assert(Simp.glyph_id_to_hash_key(5) == -5);
    ok := string.["a", "\xc3\xa9", "\xe2\x82\xac", "\xf0\x9f\x98\x80"];
    for ok assert(Simp.isLegalUTF8(it.data, it.count));
    assert(!Simp.isLegalUTF8("\xc0\x80".data, 2));  // overlong
    assert(!Simp.isLegalUTF8("\xe2\x82".data, 2));  // truncated
    assert(Simp.get_glyph_width_in_pixels(null) == 0);

    // Type shapes.
    coords: Simp.Simp_Coordinate_System = .LEFT_HANDED;
    assert(coords != .RIGHT_HANDED);
    loaded: Simp.Loaded_Font;
    feature: Simp.Font_Feature;
    table: Simp.Font_Coverage_Table;
    assert(loaded.name.count == 0 && feature.expanded == false && table.format == 0);

    assert(Simp.find_loaded_font("no/such/font.ttf") == null);
    assert(Simp.make_loaded_font("no/such/font.ttf") == null);

    // Immediate vertices are plain triangle corners; buffering needs no GPU.
    Simp.immediate_begin();
    for 0..2 Simp.immediate_vertex(.{cast(float) it, 0, 0}, .{1, 1, 1, 1});

    // Real font, if the system has one.
    candidates := string.["/System/Library/Fonts/Supplemental/Arial.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"];
    for candidates {
        if !file_exists(it) continue;
        path, base := split_path(it);
        font := Simp.load_font(path, base, 16);
        assert(font != null);
        assert(Simp.find_loaded_font(tprint("%/%", path, base)) != null);
        assert(Simp.get_character_width_in_pixels(font, "W".data, 0) > 0);
        assert(Simp.get_baseline_height(font) == 4);
        break;
    }
    print("ok\n");
}

split_path :: (full: string) -> string, string {
    cut := full.count - 1;
    while cut >= 0 && full[cut] != #char "/" cut -= 1;
    if cut < 0 return "", full;
    return string.{cut, full.data}, string.{full.count - cut - 1, full.data + cut + 1};
}
