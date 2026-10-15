live_allocations: s64;
diagnostics: s64;
deny_size: s64;
denied: bool;

count_allocation :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if (mode == .ALLOCATE || mode == .RESIZE) && deny_size > 0 && size == deny_size {
        deny_size = 0;
        denied = true;
        return null;
    }

    result := Context_Base.default_allocator.proc(mode, size, old_size, old_memory, data);
    if (mode == .ALLOCATE || mode == .RESIZE) && result && !old_memory live_allocations += 1;
    if mode == .FREE && old_memory live_allocations -= 1;

    return result;
}

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    assert((info.common_flags & .ERROR) != 0);
    diagnostics += 1;
}

release_cv_types :: (types: [] *D.Codeview_Type) {
    for type: types {
        if type.kind == .ARGLIST || type.kind == .BUILDINFO || type.kind == .SUBSTR_LIST free((cast(*D.Codeview_Arguments) type).arguments.data);
        if type.kind == .METHODLIST array_free((cast(*D.Codeview_Method_List) type).elements);

        if type.kind == .FIELDLIST {
            list := cast(*D.Codeview_List) type;

            array_free(list.members);
            array_free(list.overloaded_methods);
            array_free(list.methods);
            array_free(list.enumerates);
            array_free(list.nested);
        }

        free(type);
    }

    free(types.data);
}

release_msf :: (msf: *D.Msf) {
    for stream: msf.stream_table.stream_infos array_free(stream.stream_page_numbers);

    array_free(msf.stream_table.stream_infos);
    array_free(msf.siPnList.stream_page_numbers);
    array_free(msf.fpm.rgw);
    array_free(msf.fpm_freed.rgw);
}

release_pdb :: (pdb: *D.Pdb) {
    release_cv_types(pdb.tpi);
    release_cv_types(pdb.ipi);
    free(pdb.name_table.buffer.data);
    array_free(pdb.name_table.string_to_name_map.keys);
    array_free(pdb.name_table.string_to_name_map.values);
    free(pdb.name_table.string_to_name_map.present.words.data);
    free(pdb.name_table.string_to_name_map.deleted.words.data);
    release_msf(*pdb.msf);
}

release_sequences :: (sequences: [] D.Codeview_Line_Info_Sequence) {
    for sequence: sequences {
        for block: sequence.file_blocks {
            free(block.lines.data);
            free(block.columns.data);
        }

        array_free(sequence.file_blocks);
    }

    free(sequences.data);
}

release_debug_symbols :: (info: *D.Codeview_Debug_Info) {
    for symbol: info.symbols {
        if D.is_procedure(symbol) array_free((cast(*D.Codeview_Symbol_Procedure) symbol).inlinesites);

        if symbol.symbol_type == .INLINESITE {
            site := cast(*D.Codeview_Symbol_Inline_Site) symbol;
            array_free(site.binary_annotations);
            array_free(site.children);
        }

        free(symbol);
    }

    array_free(info.symbols);
    for checksum: info.file_checksums free(checksum.checksum.data);

    array_free(info.file_checksums);

    for sequence: info.line_info {
        for block: sequence.file_blocks {
            free(block.lines.data);
            free(block.columns.data);
        }

        array_free(sequence.file_blocks);
    }

    array_free(info.line_info);
    for inlinee: info.inlinee_line_info free(inlinee.extra_files.data);

    array_free(info.inlinee_line_info);
    free(info.string_table.data);
}
