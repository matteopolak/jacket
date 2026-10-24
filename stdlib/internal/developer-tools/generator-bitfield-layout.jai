// Record layout. libclang reports each bitfield's bit offset and width in the target's own
// layout; the units below are rebuilt from those numbers so the Jai record occupies the same bytes.

#scope_module
#import "Basic";

GM_Layout_Entry :: struct {
    declaration: *Declaration;
    group: *Bitfield;
    layout: bool;
    offset: s64 = -1;
    size: s64;
    alignment: s64 = 1;
    last_byte: s64;
    widest: s64;
}

gm_bitfield_member :: (declaration: *Declaration) -> bool {
    return declaration != null && declaration.kind == .DECLARATION && declaration.bitfield_width >= 0 && declaration.bit_offset_in_parent >= 0 && declaration.storage_class != .Static && (declaration.decl_flags & .IS_METADATA) == 0 && declaration.type != null;
}

gm_round_up :: (value: s64, multiple: s64) -> s64 {
    if multiple <= 1 return value;

    return (value + multiple - 1) / multiple * multiple;
}

gm_storage_type :: (generator: *Generator, bytes: s64) -> *CType {
    if bytes == 1 return generator.type_def_u8;
    if bytes == 2 return generator.type_def_u16;
    if bytes == 4 return generator.type_def_u32;

    return generator.type_def_u64;
}

gm_unused_member_name :: (session: *GM_Sessions.Session, record: *Struct, base: string) -> string {
    for suffix: 0..record.declarations.count * 3 + 1 {
        candidate := ifx suffix == 0 then base else tprint("%_%", base, suffix);
        taken := false;

        for member: record.declarations if member && (member.name == candidate || member.output_name == candidate || member.outputted_jai_name == candidate) taken = true;

        if !taken return GM_Sessions.copy_text(session, candidate);
    }

    return GM_Sessions.copy_text(session, base);
}

// The unit that holds bytes lo..hi: the largest power-of-two integer that is aligned, covers the bytes
// and stays between the neighbouring fields. When none fits, a byte array covers exactly the bytes.
gm_choose_unit :: (lo: s64, hi: s64, widest: s64, limit_low: s64, limit_high: s64) -> base: s64, size: s64, integer: bool {
    count := hi - lo + 1;
    top: s64 = 1;

    while top < count top *= 2;

    top = max(top, widest);

    candidate: s64 = 8;

    while count <= 8 && candidate >= count {
        if candidate <= top {
            base := lo - lo % candidate;
            if base >= limit_low && base + candidate > hi && base + candidate <= limit_high return base, candidate, true;
        }

        candidate /= 2;
    }

    return lo, count, false;
}

// Replaces the record's bitfield members with one storage field per unit, in `info.modified_declarations`.
// Returns false, leaving the list untouched, when the record has no layout this can express.
gm_group_bitfields :: (session: *GM_Sessions.Session, record: *Struct, info: *Struct_Extra_Info) -> bool { // jailint: allow(cognitive_complexity)
    generator := gm_active_generator();
    options := gm_active_options();

    if !generator || !options || !record.type || record.type.size < 0 || record.type.alignment < 1 return false;

    total := record.type.size;
    alignment := record.type.alignment;
    is_union := (record.flags & .IS_UNION) != 0;
    entries: [..] GM_Layout_Entry;
    open := -1;
    window_end: s64;
    separated := false;
    known := true;

    defer array_free(entries);

    for declaration: record.declarations {
        if !declaration return false;

        if gm_bitfield_member(declaration) {
            width := declaration.bitfield_width;
            storage := strip_type(declaration.type, typedefs = true, enums = true);

            if !storage || (storage.size != 1 && storage.size != 2 && storage.size != 4 && storage.size != 8) || width > storage.size * 8 return false;

            if width == 0 {
                // A zero-width field only ends the unit; it has no storage and no name.
                separated = true;
                continue;
            }

            container := storage.size * 8;
            first_bit := declaration.bit_offset_in_parent;
            last_bit := first_bit + width - 1;
            window_low := first_bit / container * container;
            window_high := (last_bit / container + 1) * container;

            if open < 0 || is_union || separated || window_low >= window_end {
                group := GM_Sessions.new_owned(session, Bitfield);
                group.kind = .BITFIELD;
                group.location = declaration.location;
                group.parent = cast(*Declaration) record;
                group.flags |= .LAYOUT_UNIT;
                if options.generate_bitfield_accessors group.flags |= .GENERATE_HELPERS;

                entry: GM_Layout_Entry;
                entry.declaration = cast(*Declaration) group;
                entry.group = group;
                entry.layout = true;
                entry.offset = first_bit / 8;
                array_add(*entries, entry);
                open = entries.count - 1;
                window_end = 0;
            }

            unit := *entries[open];

            separated = false;
            window_end = max(window_end, window_high);
            unit.last_byte = max(unit.last_byte, last_bit / 8);
            unit.widest = max(unit.widest, storage.size);
            unit.offset = min(unit.offset, first_bit / 8);
            GM_Sessions.append_owned(session, *unit.group.members, declaration);
            continue;
        }

        entry: GM_Layout_Entry;
        entry.declaration = declaration;

        if declaration.kind == .DECLARATION && declaration.storage_class != .Static && (declaration.decl_flags & .IS_METADATA) == 0 {
            open = -1;
            separated = false;
            entry.layout = true;

            if !declaration.type || declaration.type.size < 0 || declaration.bit_offset_in_parent < 0 || declaration.bit_offset_in_parent % 8 != 0 {
                known = false;
            } else {
                natural, _ := gm_alignment(declaration.type, declaration);

                entry.offset = declaration.bit_offset_in_parent / 8;
                entry.size = declaration.type.size;
                entry.alignment = natural;
                if natural < 1 known = false;
            }
        } else if declaration.kind == .STRUCT && (!declaration.output_name || (cast(*Struct) declaration).flags & .IS_ANON_INLINE_TYPE) {
            represented := false;

            for member: record.declarations if member && member.kind == .DECLARATION && try_find_struct_in_type(member.type) == cast(*Struct) declaration represented = true;

            if !represented && !declaration.output_name {
                // A C11 anonymous member has no offset of its own to compare against.
                open = -1;
                separated = false;
                entry.layout = true;
                known = false;
            }
        }

        array_add(*entries, entry);
    }

    // Choose each unit's storage between its neighbours.
    previous_end: s64 = 0;
    previous_known := true;
    index := 0;

    while index < entries.count {
        entry := *entries[index];

        if !entry.layout {
            index += 1;
            continue;
        }

        if !entry.group {
            if entry.offset >= 0 {
                previous_end = entry.offset + entry.size;
                previous_known = true;
            } else previous_known = false;

            index += 1;
            continue;
        }

        next_start := entry.last_byte + 1;
        if is_union next_start = total;
        else {
            following := index + 1;

            while following < entries.count && !entries[following].layout following += 1;
            if following >= entries.count next_start = total;
            else if entries[following].offset >= 0 next_start = entries[following].offset;
        }

        lowest := ifx previous_known && !is_union then previous_end else entry.offset;
        if is_union lowest = 0;

        base, size, integer := gm_choose_unit(entry.offset, entry.last_byte, entry.widest, lowest, max(next_start, entry.last_byte + 1));
        group := entry.group;

        group.type = ifx integer then gm_storage_type(generator, size) else gm_metadata_byte_storage(size);
        group.bit_offset_in_parent = base * 8;
        group.field_alignment = ifx integer then size else 1;
        entry.offset = base;
        entry.size = size;
        entry.alignment = group.field_alignment;
        previous_end = base + size;
        previous_known = true;
        index += 1;
    }

    // Name the units after the fields they hold.
    unit := 0;

    for *entry: entries if entry.group {
        entry.group.name = gm_unused_member_name(session, record, ifx unit == 0 then "__bitfield" else tprint("__bitfield_%", unit));
        entry.group.output_name = entry.group.name;
        unit += 1;
    }

    // Compare the placement Jai would give these fields with the native layout.
    explicit := false;
    stored := false;

    for entry: entries if entry.layout stored = true;

    if known && stored && !is_union {
        cursor: s64;
        widest_alignment: s64 = 1;
        matches := true;

        for entry: entries if entry.layout {
            if gm_round_up(cursor, entry.alignment) != entry.offset matches = false;

            cursor = entry.offset + entry.size;
            widest_alignment = max(widest_alignment, entry.alignment);
        }

        if gm_round_up(cursor, max(widest_alignment, alignment)) != total || widest_alignment > alignment matches = false;

        if !matches {
            // Every field must be placed by an explicit offset, so check they are consistent first.
            cursor = 0;
            explicit = true;

            for entry: entries if entry.layout {
                if entry.offset < cursor explicit = false;
                cursor = entry.offset + entry.size;
            }

            if cursor > total explicit = false;
        }
    }

    cursor: s64;

    for *entry: entries {
        if !entry.layout {
            if !entry.declaration return false;
            GM_Sessions.append_movable_owned(session, *info.modified_declarations, entry.declaration);
            continue;
        }

        // A unit wider than the record's own alignment (packed records, MSVC unions) keeps the record's.
        if (explicit || is_union) && entry.alignment > alignment entry.declaration.explicit_field_alignment = alignment;

        if explicit {
            if entry.offset > cursor gm_add_padding(session, record, info, cursor, entry.offset - cursor);
            cursor = entry.offset + entry.size;
        }

        GM_Sessions.append_movable_owned(session, *info.modified_declarations, entry.declaration);
    }

    if explicit {
        if total > cursor gm_add_padding(session, record, info, cursor, total - cursor);
        info.explicit_layout = true;
    }

    return true;
}

gm_add_padding :: (session: *GM_Sessions.Session, record: *Struct, info: *Struct_Extra_Info, offset: s64, bytes: s64) {
    name := gm_unused_member_name(session, record, tprint("__padding_%", offset));
    padding := gm_generated_member(session, record, name, gm_metadata_byte_storage(bytes), offset * 8);

    GM_Sessions.append_movable_owned(session, *info.modified_declarations, padding);
}
