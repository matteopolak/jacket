use super::*;

#[test]
fn lines_of_lf_text() {
    let text = "ab\ncd\n";
    let index = LineIndex::new(text);
    assert_eq!(index.line_count(), 3);
    assert_eq!(index.line_range(0), Some((0, 2)));
    assert_eq!(index.line_range(1), Some((3, 5)));
    assert_eq!(index.line_range(2), Some((6, 6)));
    assert_eq!(index.line_range(3), None);
    assert_eq!(index.line_start(4), 3);
    assert_eq!(index.line_end(3), 5);
}

#[test]
fn crlf_line_ends_exclude_the_carriage_return() {
    let text = "ab\r\ncd\r\n\r\nef\r";
    let index = LineIndex::new(text);
    let lines: Vec<&str> = index.lines().map(|(a, b)| &text[a..b]).collect();
    assert_eq!(lines, ["ab", "cd", "", "ef"]);
    // On the `\r` itself the position is the line's end, not past it.
    assert_eq!(index.utf16_position(text, 2), Some((0, 2)));
    assert_eq!(index.utf16_position(text, 3), Some((0, 2)));
    assert_eq!(index.line_start(5), 4);
    assert_eq!(index.line_end(5), 6);
}

#[test]
fn empty_text_has_one_empty_line() {
    let index = LineIndex::new("");
    assert_eq!(index.line_count(), 1);
    assert_eq!(index.line_range(0), Some((0, 0)));
    assert_eq!(index.line_col("", 0), (1, 1));
    assert_eq!(index.utf16_position("", 0), Some((0, 0)));
    assert_eq!(index.offset_utf16("", 0, 5), 0);
    assert_eq!(index.offset_utf16("", 3, 0), 0);
}

#[test]
fn offsets_at_and_past_the_end() {
    let text = "ab\ncd";
    let index = LineIndex::new(text);
    assert_eq!(index.line_col(text, 5), (2, 3));
    assert_eq!(index.line_col(text, 99), (2, 3));
    assert_eq!(index.utf16_position(text, 5), Some((1, 2)));
    assert_eq!(index.utf16_position(text, 6), None);
    assert_eq!(index.line_end(99), 5);
    let ends_in_newline = "ab\n";
    let index = LineIndex::new(ends_in_newline);
    assert_eq!(index.line_col(ends_in_newline, 3), (2, 1));
    assert_eq!(index.utf16_position(ends_in_newline, 3), Some((1, 0)));
    assert_eq!(index.offset_utf16(ends_in_newline, 1, 0), 3);
    // A line past the last one names the end of the text.
    assert_eq!(index.offset_utf16(ends_in_newline, 2, 0), 3);
    assert_eq!(index.offset_utf16(ends_in_newline, u32::MAX, u32::MAX), 3);
}

#[test]
fn columns_in_characters_bytes_and_utf16() {
    // 'é' is 2 bytes and 1 unit; the emoji is 4 bytes and 2 units.
    let text = "x\né😀z";
    let index = LineIndex::new(text);
    let z = text.find('z').unwrap();
    assert_eq!(index.line_col(text, z), (2, 3));
    assert_eq!(index.line_byte_col(z), (2, 7));
    assert_eq!(index.utf16_position(text, z), Some((1, 3)));
    assert_eq!(index.offset_utf16(text, 1, 3), z);
    // The middle of the emoji, in bytes and in UTF-16 units.
    assert_eq!(index.utf16_position(text, z - 2), None);
    // A column between the halves of a pair names the start of the character.
    assert_eq!(index.offset_utf16(text, 1, 2), z - 4);
    assert_eq!(index.offset_utf16(text, 1, 1), z - 4);
    assert_eq!(index.offset_utf16(text, 1, 0), 2);
    // Offsets inside a character are moved back for the character column.
    assert_eq!(index.line_col(text, z - 2), (2, 2));
    // Past the line's end names the end.
    assert_eq!(index.offset_utf16(text, 1, 40), text.len());
}

#[test]
fn line_start_of_free_function() {
    let text = "a\nbc\r\nd";
    assert_eq!(line_start(text, 0), 0);
    assert_eq!(line_start(text, 3), 2);
    assert_eq!(line_start(text, 7), 6);
    assert_eq!(line_start(text, 100), 6);
    assert_eq!(line_start("é", 1), 0);
}

#[test]
fn source_file_columns_count_bytes() {
    let file = SourceFile::new("a.jai".into(), "é\nxy\r\nz".into());
    assert_eq!(file.line_col(2), (1, 3));
    assert_eq!(file.line_col(3), (2, 1));
    assert_eq!(file.line_text(2), "xy");
    assert_eq!(file.line_text(3), "z");
    assert_eq!(file.offset_of(2, 2), 4);
    assert_eq!(file.offset_of(9, 9), file.text.len() as u32);
}

/// A short text made of the characters that matter: both line endings, a 2-byte character
/// and an astral one.
fn random_text(seed: &mut u64) -> String {
    let mut next = || {
        *seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (*seed >> 33) as usize
    };
    let alphabet = ['a', 'b', ' ', '\n', '\r', '\n', 'é', '😀', '\u{2028}'];
    let len = next() % 30;
    (0..len)
        .map(|_| alphabet[next() % alphabet.len()])
        .collect()
}

/// The language server's original line table, kept as the reference the shared one must
/// agree with.
fn reference(text: &str) -> (Vec<usize>, Vec<usize>) {
    let mut starts = vec![0];
    let mut ends = vec![];
    for (at, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            ends.push(if at > 0 && text.as_bytes()[at - 1] == b'\r' {
                at - 1
            } else {
                at
            });
            starts.push(at + 1);
        }
    }
    ends.push(text.len());
    (starts, ends)
}

#[test]
fn utf16_positions_agree_with_the_original_on_random_text() {
    let mut seed = 7;
    for _ in 0..2000 {
        let text = random_text(&mut seed);
        let index = LineIndex::new(&text);
        let (starts, ends) = reference(&text);
        for byte in 0..=text.len() {
            if !text.is_char_boundary(byte) {
                assert_eq!(index.utf16_position(&text, byte), None);
                continue;
            }
            let line = starts.partition_point(|s| *s <= byte) - 1;
            // A `\r` ending the text is part of the last line in the original; the shared
            // index leaves it out like every other line ending.
            let end = if line == ends.len() - 1 && text.ends_with('\r') && text.len() > starts[line]
            {
                ends[line] - 1
            } else {
                ends[line]
            };
            let want = (
                line as u32,
                text[starts[line]..byte.min(end)].encode_utf16().count() as u32,
            );
            assert_eq!(
                index.utf16_position(&text, byte),
                Some(want),
                "{text:?} at {byte}"
            );
            // And back, unless the offset sat inside a line ending.
            if byte <= end {
                assert_eq!(
                    index.offset_utf16(&text, want.0, want.1),
                    byte,
                    "{text:?} at {byte}"
                );
            }
        }
    }
}

#[test]
fn char_columns_agree_with_counting_from_the_line_start() {
    let mut seed = 99;
    for _ in 0..2000 {
        let text = random_text(&mut seed);
        let index = LineIndex::new(&text);
        for byte in 0..=text.len() + 2 {
            let clamped = text.floor_char_boundary(byte.min(text.len()));
            let line = text[..clamped].matches('\n').count() + 1;
            let start = text[..clamped].rfind('\n').map_or(0, |n| n + 1);
            let col = text[start..clamped].chars().count() + 1;
            assert_eq!(
                index.line_col(&text, byte),
                (line, col),
                "{text:?} at {byte}"
            );
            assert_eq!(line_start(&text, clamped), start);
            assert_eq!(index.line_start(clamped), start);
            assert_eq!(
                line_end(&text, clamped),
                index.line_end(clamped),
                "{text:?} at {clamped}"
            );
        }
    }
}

#[test]
fn naming_the_typechecked_message_is_tracked_per_build() {
    let named = |usage: &TypecheckedUse| usage.get() & TYPECHECKED_NAMED != 0;
    let (first, second) = (TypecheckedUse::default(), TypecheckedUse::default());
    let mut a = SourceMap::default();
    let mut b = SourceMap::default();
    a.share_typechecked_use(first.clone());
    b.share_typechecked_use(second.clone());
    a.add("/app/main.jai", "m: *Message_Typechecked;".into());
    assert!(named(&first));
    assert!(!named(&second));
    b.add("/app/other.jai", "x :: 1;".into());
    assert!(!named(&second));
    // The record definition and its delivery code do not count.
    b.add(
        "/lib/Compiler/contract-records.jai",
        "Message_Typechecked :: struct {}".into(),
    );
    assert!(!named(&second));
    // A map that named it before joining a build brings that along.
    let mut c = SourceMap::default();
    c.add("/app/late.jai", "Message_Typechecked".into());
    c.share_typechecked_use(second.clone());
    assert!(named(&second));
}

#[test]
fn updating_after_random_edits_matches_a_full_rebuild() {
    let mut seed = 31;
    for _ in 0..300 {
        let mut text = random_text(&mut seed);
        let mut index = LineIndex::new(&text);
        for _ in 0..20 {
            let piece = random_text(&mut seed);
            let at = |seed: &mut u64| {
                *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                text.floor_char_boundary((*seed >> 33) as usize % (text.len() + 1))
            };
            let start = at(&mut seed);
            let end = at(&mut seed).max(start);
            text.replace_range(start..end, &piece);
            index.update(&text, start);
            assert_eq!(
                index,
                LineIndex::new(&text),
                "{text:?} after an edit at {start}"
            );
        }
    }
}
