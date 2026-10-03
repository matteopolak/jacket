#scope_module
// Native OpenType GSUB structures, read from the caller's face through FreeType.
// Common-format and single/alternate substitution layouts follow the public
// Microsoft OpenType specification; offsets are checked before each access.
Gsub_Storage :: struct {
    data: [] u8;
}

gsub_storage: [..] Gsub_Storage;

Extension_Kind :: struct {
    table: *Font_Lookup_Table;
    kind: s32;
}

extension_kinds: [..] Extension_Kind;

discover_font_features :: (font: *Dynamic_Font) -> bool {
    if (font.private1.face_flags & FT_FACE_FLAG_SFNT) == 0 return true;

    length: FT_ULong;
    status := FT_Load_Sfnt_Table(font.private1, 0x47535542, 0, null, *length);

    if status == 0x8e return true; // FT_Err_Table_Missing: an actual absent optional table.

    if status != 0 || length < 10 || length > 0xffffffff {
        report_font_error("native GSUB table query failed", status);
        return false;
    }

    data: [] u8 = .{ cast(int) length, cast(*u8) Basic.alloc(cast(int) length) };

    if !data.data {
        report_font_error("GSUB table storage allocation failed", 0);
        return false;
    }

    status = FT_Load_Sfnt_Table(font.private1, 0x47535542, 0, data.data, *length);
    if status != 0 || length < 10 || length > cast(FT_ULong) data.count {
        Basic.free(data.data);
        report_font_error("native GSUB table load failed", status);
        return false;
    }

    data.count = cast(int) length;
    assert(gsub_u16(data, 0) == 1, "unsupported GSUB major version");
    feature_list := cast(int) gsub_u16(data, 6);
    count := gsub_u16(data, feature_list);

    for index: 0..cast(int) count - 1 {
        record := feature_list + 2 + index * 6;
        offset := feature_list + gsub_u16(data, record + 4);
        feature: Font_Feature;
        feature.feature_tag = gsub_u32(data, record);
        feature.feature_tables_pointer = data.data + offset;
        feature.lookup_lists = gsub_words(data, offset + 4, gsub_u16(data, offset + 2));
        Basic.array_add(*font.features, feature);
    }

    Basic.array_add(*gsub_storage, Gsub_Storage.{ data });

    for *feature: font.features {
        if feature.feature_tag == 0x736d6370 && !font.feature_smallcaps font.feature_smallcaps = feature;
        if feature.feature_tag == 0x6c6e756d && !font.feature_lining_numbers font.feature_lining_numbers = feature;
        if feature.feature_tag == 0x6c696761 && !font.feature_ligatures font.feature_ligatures = feature;
    }

    return true;
}

expand_font_feature :: (feature: *Font_Feature) {
    if feature.expanded return;

    data: [] u8;

    for storage: gsub_storage {
        if feature.feature_tables_pointer >= storage.data.data && feature.feature_tables_pointer < storage.data.data + storage.data.count {
            data = storage.data;
            break;
        }
    }

    assert(data.data != null, "unexpanded feature has no retained native GSUB table");
    list := cast(int) gsub_u16(data, 8);
    count := gsub_u16(data, list);

    feature.lookup_tables = Basic.NewArray(feature.lookup_lists.count, Font_Lookup_Table);
    for lookup, index: feature.lookup_lists {
        assert(lookup < count, "GSUB feature references an invalid lookup index");
        offset := list + gsub_u16(data, list + 2 + cast(int) lookup * 2);
        table := *feature.lookup_tables[index];
        table.lookup_type = gsub_u16(data, offset);
        table.lookup_flags = gsub_u16(data, offset + 2);
        sub_count := gsub_u16(data, offset + 4);

        assert((table.lookup_flags & 0x00e0) == 0, "GSUB lookup has reserved flag bits");

        table.subtables = Basic.NewArray(sub_count, Font_Lookup_Subtable);

        for sub_index: 0..cast(int) sub_count - 1 {
            sub := offset + gsub_u16(data, offset + 6 + sub_index * 2);
            kind := table.lookup_type;

            if kind == 7 {
                assert(gsub_u16(data, sub) == 1, "invalid GSUB extension format");
                kind = gsub_u16(data, sub + 2);
                sub += gsub_u32(data, sub + 4);
                Basic.array_add(*extension_kinds, Extension_Kind.{ table, kind });
            }

            assert(kind == 1 || kind == 3, "this font feature requires contextual/multiple substitutions outside the current single-glyph path");
            result := *table.subtables[sub_index];
            result.format = gsub_u16(data, sub);
            result.coverage_table = gsub_coverage(data, sub + gsub_u16(data, sub + 2));

            if kind == 1 {
                if result.format == 1 {
                    delta := cast(s32) gsub_u16(data, sub + 4);
                    if delta >= 32768 delta -= 65536;
                    result.delta_glyph_id = delta;
                } else {
                    assert(result.format == 2, "invalid GSUB single-substitution format");
                    result.glyphs = gsub_words(data, sub + 6, gsub_u16(data, sub + 4));
                }
            } else {
                assert(result.format == 1, "invalid GSUB alternate-substitution format");
                alternatives := gsub_u16(data, sub + 4);

                result.sequence_table = Basic.NewArray(alternatives, Font_Sequence_Table_Entry);

                for alt: 0..cast(int) alternatives - 1 {
                    set := sub + gsub_u16(data, sub + 6 + alt * 2);
                    result.sequence_table[alt].glyphs = gsub_words(data, set + 2, gsub_u16(data, set));
                }
            }
        }
    }

    feature.expanded = true;
}

apply_font_feature :: (feature: *Font_Feature, glyph: u32) -> u32 {
    if !feature return glyph;

    persistent := font_context();

    push_context, defer_pop persistent;
    expand_font_feature(feature);
    result := glyph;

    for *table: feature.lookup_tables {
        // OpenType lookup flags filter other glyphs in sequence matching, not
        // the current glyph. Single/alternate lookups have no other inputs.

        kind := table.lookup_type;
        if kind == 7 for entry: extension_kinds if entry.table == table {
            kind = entry.kind;
            break;
        }

        for sub: table.subtables {
            index := coverage_index(sub.coverage_table, result);
            if index < 0 continue;

            if kind == 1 {
                if sub.format == 1 {
                    adjusted := cast(s64) result + sub.delta_glyph_id;
                    adjusted %= 65536;
                    if adjusted < 0 adjusted += 65536;
                    result = cast(u32) adjusted;
                } else {
                    assert(index < sub.glyphs.count, "GSUB coverage exceeds substitution array");
                    result = sub.glyphs[index];
                }
            } else {
                assert(kind == 3 && index < sub.sequence_table.count && sub.sequence_table[index].glyphs.count > 0);
                result = sub.sequence_table[index].glyphs[0];
            }

            break;
        }
    }

    return result;
}

coverage_index :: (coverage: *Font_Coverage_Table, glyph: u32) -> int {
    if coverage.format == 1 {
        for candidate, index: coverage.glyphs if candidate == glyph return index;
    } else {
        assert(coverage.format == 2, "invalid GSUB coverage format");
        for range: coverage.range_records if cast(s64) glyph >= range.start && cast(s64) glyph <= range.end return range.start_coverage_index + cast(int) glyph - range.start;
    }

    return -1;
}

gsub_coverage :: (data: [] u8, offset: int) -> *Font_Coverage_Table {
    result := Basic.New(Font_Coverage_Table);
    result.format = gsub_u16(data, offset);
    count := gsub_u16(data, offset + 2);
    if result.format == 1 result.glyphs = gsub_words(data, offset + 4, count);
    else {
        assert(result.format == 2, "invalid GSUB coverage format");
        result.range_records = Basic.NewArray(count, Font_Range_Record);

        for index: 0..cast(int) count - 1 {
            record := offset + 4 + index * 6;
            result.range_records[index] = .{ gsub_u16(data, record), gsub_u16(data, record + 2), gsub_u16(data, record + 4) };
        }
    }

    return result;
}

gsub_words :: (data: [] u8, offset: int, count: u16) -> [] u16 {
    result := Basic.NewArray(count, u16);
    for index: 0..cast(int) count - 1 result[index] = gsub_u16(data, offset + index * 2);
    return result;
}

gsub_u16 :: (data: [] u8, offset: int) -> u16 {
    assert(offset >= 0 && offset + 2 <= data.count, "truncated GSUB word");
    high: u16 = data[offset];
    low: u16 = data[offset + 1];
    return (high << 8) | low;
}

gsub_u32 :: (data: [] u8, offset: int) -> u32 {
    high: u32 = gsub_u16(data, offset);
    low: u32 = gsub_u16(data, offset + 2);
    return (high << 16) | low;
}
