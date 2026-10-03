SECTION_VERSION_NUMBER :: 1;

Subtitles_Line :: struct {
    type: enum u8 {
        UNINITIALIZED :: 0;
        SPOKEN :: 1;
        DIRECTION :: 2;
    }
    text: string;
}

Subtitles_Event :: struct {
    name: string;
    parameter: string;
    instant_index: s32;
    time_begin: float64 = -1;
    time_end: float64 = -1;
}

Subtitles_Instant :: struct {
    lines: [..] Subtitles_Line;
    speaker: string;
    time: float64 = -1;
}

Subtitles_Section :: struct {
    name: string;
    embed: string;
    instants: [..] Subtitles_Instant;
    events: [..] Subtitles_Event;
    flags: enum_flags u32 {
        FINALIZED :: 1;
    }
}

Subtitles :: struct {
    data: string;
    data_owned: bool;
    sections_by_name: H.Table(string, *Subtitles_Section);
    all_sections: [..] Subtitles_Section;
}
