// Dialogue scripts for games: sections of timed speech, stage directions and events. Load text
// with `load_subtitles_from_memory` or `load_subtitles_from_file`, write it with `serialize`, and
// turn sections into a reading page (`format_sections_as_html`) or a recording sheet
// (`format_sections_as_csv`).
Subtitles :: Impl.Subtitles;
Subtitles_Section :: Impl.Subtitles_Section;
Subtitles_Line :: Impl.Subtitles_Line;
Subtitles_Instant :: Impl.Subtitles_Instant;
Subtitles_Event :: Impl.Subtitles_Event;
SECTION_VERSION_NUMBER :: Impl.SECTION_VERSION_NUMBER;
load_subtitles_from_file :: Impl.load_subtitles_from_file;
load_subtitles_from_memory :: Impl.load_subtitles_from_memory;
serialize :: Impl.serialize;
free :: Impl.free;
get_sections_for_characters :: Impl.get_sections_for_characters;
format_sections_as_html :: Impl.format_sections_as_html;
format_section_as_html :: Impl.format_section_as_html;
csv_line_number: int;
csv_line_index_within_section: int;

csv_quote :: Impl.csv_quote;

// Appends a recording script as CSV: a header row, then one row per speaker turn with its
// dialogue, directions and the wav file names to record into.
format_sections_as_csv :: (builder: *B.String_Builder, sections: [] *Subtitles_Section, name_map: *H.Table(string, string) = null) {
    Impl.format_sections_as_csv(builder, sections, name_map, *csv_line_number, *csv_line_index_within_section);
}

// Appends one section's CSV rows. See `format_sections_as_csv`.
format_section_as_csv :: (builder: *B.String_Builder, section: *Subtitles_Section, name_map: *H.Table(string, string) = null) {
    Impl.format_section_as_csv(builder, section, name_map, *csv_line_number, *csv_line_index_within_section);
}

// Writes the CSV row for one speaker's gathered lines, if any of them has text.
flush_csv_lines :: (builder: *B.String_Builder, section_name: string, name_map: *H.Table(string, string), _speaker_name: string, lines: [] string, directions: [] string) {
    Impl.flush_csv_lines(builder, section_name, name_map, _speaker_name, lines, directions, *csv_line_number, *csv_line_index_within_section);
}

#scope_file
B :: #import "Basic";
H :: #import "Hash_Table";
Impl :: #import "internal/subtitles";
