// The format version `serialize` writes in its `[1]` header line.
SECTION_VERSION_NUMBER :: 1;

// One line of a section: something a character says, or a stage direction.
Subtitles_Line :: struct {
    type: enum u8 {
        UNINITIALIZED :: 0;
        SPOKEN :: 1;
        DIRECTION :: 2;
    }
    text: string;
}

// A named cue (an animation or sound, say) with an optional parameter, tied to an instant or to
// explicit start and end times. A time of -1 is unset.
Subtitles_Event :: struct {
    name: string;
    parameter: string;
    instant_index: s32;
    time_begin: float64 = -1;
    time_end: float64 = -1;
}

// A moment of dialogue: one speaker, the lines they say, and when (-1 when untimed). A speaker of
// `_` in the source reads back as an empty `speaker`.
Subtitles_Instant :: struct {
    lines: [..] Subtitles_Line;
    speaker: string;
    time: float64 = -1;
}

// A named scene or conversation: its instants and events in order.
Subtitles_Section :: struct {
    name: string;
    embed: string;
    instants: [..] Subtitles_Instant;
    events: [..] Subtitles_Event;
    flags: enum_flags u32 {
        FINALIZED :: 1;
    }
}

// A parsed subtitle file. Free it with `free`.
Subtitles :: struct {
    data: string;
    data_owned: bool;
    sections_by_name: H.Table(string, *Subtitles_Section);
    all_sections: [..] Subtitles_Section;
}
