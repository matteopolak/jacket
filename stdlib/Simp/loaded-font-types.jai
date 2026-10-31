// A font file read into memory with FreeType. `load_font` makes these for you.
Loaded_Font :: struct {
    name: string;
    face: FT_Face;
    data: [] u8;
}
