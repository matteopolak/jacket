// Main-thread LSP integration. Replies carry a document version and caret snapshot.
clear_language_popup :: (ed: *Editor) {
    free(ed.hover);
    ed.hover = "";
    ed.hover_doc = null;
    free_json(*ed.completions);
    ed.completion_doc = null;
}

sync_language :: (ed: *Editor) {
    if !ed.language.ready || ed.language.failed return;

    for doc: ed.docs {
        if !Str.ends_with(doc.path, ".jai") || doc.lsp_version == doc.edit.buffer.version continue;

        text := contents(*doc.edit);
        uri := Lsp.uri(doc.path);
        version := doc.edit.buffer.version;

        if doc.lsp_version < 0 {
            Lsp.send(*ed.language, "textDocument/didOpen", tprint("{\"textDocument\":{\"uri\":%,\"languageId\":\"jai\",\"version\":%,\"text\":%}}", Lsp.quote(uri), version, Lsp.quote(text)));
        } else {
            Lsp.cancel_document(*ed.language, uri);
            change := tprint("{\"text\":%}", Lsp.quote(text));

            if ed.language.incremental change = Lsp.change(doc.lsp_text, text);

            Lsp.send(*ed.language, "textDocument/didChange", tprint("{\"textDocument\":{\"uri\":%,\"version\":%},\"contentChanges\":[%]}", Lsp.quote(uri), version, change));
        }

        free(doc.lsp_text);
        doc.lsp_text = copy_string(text);
        doc.lsp_version = version;
        free_json(*doc.diagnostics);
        array_reset(*doc.diagnostic_marks);
        if doc == active_document(ed) clear_language_popup(ed);
    }
}

update_language :: (ed: *Editor) {
    active := active_document(ed);
    if ed.hover_doc && (ed.hover_doc != active || ed.hover_version != active.edit.buffer.version || ed.hover_cursor != active.edit.cursor) clear_language_popup(ed);

    client := *ed.language;
    if !client.running return;

    if client.failed {
        set_status(ed, "jailsp disconnected: language features unavailable");
        clear_language_popup(ed);
        Lsp.stop(client);
        return;
    }

    Lsp.poll(client);

    for 0..31 {
        body := Lsp.take_frame(client);
        if !body.count break;

        message, error := from_json(Json_Value, body);
        if error.kind == .NONE receive_language(ed, *message);
        else client.failed = true;

        free_json(*error);
        free_json(*message);
        free(body);
    }

    sync_language(ed);
    wake_in(current_ui(), 0.03);
}

receive_language :: (ed: *Editor, message: *Json_Value) {
    method := Lsp.text(Lsp.get(message, "method"));
    params := Lsp.get(message, "params");

    if method == "textDocument/publishDiagnostics" {
        for doc: ed.docs {
            if Lsp.uri(doc.path) != Lsp.text(Lsp.get(params, "uri")) continue;

            version := Lsp.get(params, "version");
            if doc.lsp_version != doc.edit.buffer.version || (version && Lsp.number(version) != doc.lsp_version) return;

            diagnostics := Lsp.get(params, "diagnostics");
            if !diagnostics return;

            free_json(*doc.diagnostics);
            encoded := to_json(diagnostics.*);

            doc.diagnostic_version = doc.lsp_version;
            doc.diagnostics, _ = from_json(Json_Value, encoded);
            free(encoded);
            array_reset(*doc.diagnostic_marks);
            text := contents(*doc.edit);

            for *diagnostic: doc.diagnostics.items {
                range := Lsp.get(diagnostic, "range");
                a := Lsp.byte_offset(text, Lsp.read_position(Lsp.get(range, "start")));
                b := Lsp.byte_offset(text, Lsp.read_position(Lsp.get(range, "end")));
                array_add(*doc.diagnostic_marks, .{ start = a, end = max(a + 1, b), color = current_ui().theme.error, underline = true });
            }
        }

        return;
    }

    id := Lsp.number(Lsp.get(message, "id"));
    pending, found := Lsp.finish(*ed.language, id);

    if !found return;

    defer Lsp.free_pending(*pending);
    result := Lsp.get(message, "result");

    if pending.method == "initialize" {
        if !result {
            ed.language.failed = true;
            return;
        }

        caps := Lsp.get(result, "capabilities");
        sync := Lsp.get(caps, "textDocumentSync");

        ed.language.incremental = Lsp.number(Lsp.get(sync, "change")) == 2 || Lsp.number(sync) == 2;
        encoding := Lsp.text(Lsp.get(caps, "positionEncoding"));
        if encoding.count && encoding != "utf-16" {
            ed.language.failed = true;
            return;
        }

        Lsp.send(*ed.language, "initialized", "{}");
        ed.language.ready = true;
        set_status(ed, "jailsp ready | F1 hover, F12 definition, Alt+Left back, Ctrl+Space completion, Ctrl+Shift+F format");

        return;
    }

    doc := active_document(ed);
    if !doc || Lsp.uri(doc.path) != pending.uri || doc.edit.buffer.version != pending.version || doc.edit.cursor != pending.cursor return;

    if !result || result.kind == .NULL {
        set_status(ed, "No language result");
        return;
    }

    if pending.method == "textDocument/hover" {
        clear_language_popup(ed);
        ed.hover = copy_string(hover_text(Lsp.get(result, "contents")));
        ed.hover_doc = doc;
        ed.hover_cursor = doc.edit.cursor;
        ed.hover_version = doc.edit.buffer.version;
    } else if pending.method == "textDocument/definition" {
        location := result;
        if result.kind == .ARRAY location = json_at(result, 0);

        path := Lsp.path_from_uri(Lsp.text(Lsp.get(location, "uri")));
        range := Lsp.get(location, "range");

        if !path.count return;

        free(ed.back_path);
        ed.back_path = copy_string(doc.path);
        ed.back_cursor = doc.edit.cursor;
        target := open_document(ed, path);
        target.edit.cursor = Lsp.byte_offset(contents(*target.edit), Lsp.read_position(Lsp.get(range, "start")));
        target.edit.anchor = target.edit.cursor;
    } else if pending.method == "textDocument/completion" {
        clear_language_popup(ed);
        items := result;
        if result.kind == .OBJECT items = Lsp.get(result, "items");
        if !items return;

        encoded := to_json(items.*);

        ed.completions, _ = from_json(Json_Value, encoded);
        free(encoded);
        ed.completion_doc = doc;
        ed.completion_version = pending.version;
        ed.completion_end = doc.edit.cursor;
        ed.completion_start = doc.edit.cursor;
        text := contents(*doc.edit);
        while ed.completion_start > 0 && identifier_byte(text[ed.completion_start - 1]) ed.completion_start -= 1;

        set_contents(*ed.completion_filter, Str.slice(text, ed.completion_start, ed.completion_end - ed.completion_start));
    }
}

hover_text :: (value: *Json_Value) -> string {
    if !value return "";
    if value.kind == .STRING return value.text;
    if value.kind == .OBJECT return Lsp.text(Lsp.get(value, "value"));

    builder: Str.String_Builder;
    builder.allocator = temp;

    for *item: value.items {
        Str.append(*builder, hover_text(item));
        Str.append(*builder, "\n");
    }

    return Str.builder_to_string(*builder,, temp);
}

identifier_byte :: (c: u8) -> bool {
    return c >= 128 || (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) || c == 95;
}

language_request :: (ed: *Editor, method: string) {
    doc := active_document(ed);
    if !doc || !ed.language.ready || ed.language.failed || !Str.ends_with(doc.path, ".jai") return;

    sync_language(ed);
    uri := Lsp.uri(doc.path);

    Lsp.cancel_document(*ed.language, uri);
    pos := Lsp.position(contents(*doc.edit), doc.edit.cursor);

    Lsp.request(*ed.language, method, tprint("{\"textDocument\":{\"uri\":%},\"position\":{\"line\":%,\"character\":%}}", Lsp.quote(uri), pos.line, pos.character), uri, doc.edit.buffer.version, doc.edit.cursor);
}

language_keys :: (ed: *Editor, ui: *UI_State) {
    for event: ui.input.keys {
        if event.kind != .KEY continue;
        if event.key == .F1 language_request(ed, "textDocument/hover");
        if event.key == .F12 language_request(ed, "textDocument/definition");
        if event.key == .SPACEBAR && event.mods.ctrl language_request(ed, "textDocument/completion");

        if event.key == .ARROW_LEFT && event.mods.alt && ed.back_path.count {
            doc := open_document(ed, ed.back_path);
            doc.edit.cursor = min(ed.back_cursor, doc.edit.buffer.length);
            doc.edit.anchor = doc.edit.cursor;
        }

        if event.key == cast(Key_Code) #char "F" && primary(event.mods) && event.mods.shift format_document(ed);
        if event.key == .ESCAPE clear_language_popup(ed);
    }
}

format_document :: (ed: *Editor) {
    doc := active_document(ed);
    if !doc return;

    config, _, ok, error := editor_format_config(doc.path);

    if !ok {
        set_status(ed, error);
        return;
    }

    if Fmt.format_is_ignored(config, doc.path) {
        set_status(ed, "File ignored by formatter config");
        return;
    }

    formatted, success, problem := Fmt.format_source(contents(*doc.edit), config);

    defer free(formatted);

    if !success {
        set_status(ed, problem);
        return;
    }

    cursor := doc.edit.cursor;

    select_all(*doc.edit);
    insert_text(*doc.edit, formatted);
    doc.edit.cursor = min(cursor, doc.edit.buffer.length);
    doc.edit.anchor = doc.edit.cursor;
    set_status(ed, "Document formatted");
}

language_panel :: (ed: *Editor, doc: *Document) {
    if ed.hover.count {
        for panel(.{ padding = 8 }) label(ed.hover);
    }

    if ed.completion_doc == doc && ed.completion_version == doc.edit.buffer.version && ed.completion_end == doc.edit.cursor {
        for panel(.{ padding = 6, height = 160 }) {
            label("Completion filter (click an item to insert)");
            text_field(*ed.completion_filter);
            filter := contents(*ed.completion_filter);

            for scroll(.{ grow = 1 }) {
                for *item: ed.completions.items {
                    name := Lsp.text(Lsp.get(item, "label"));
                    if !Str.contains(name, filter) continue;

                    key(name);

                    if list_item(name, false) {
                        apply_completion(ed, doc, item);
                        break;
                    }
                }
            }
        }
    }

    if doc.diagnostics.items.count {
        for panel(.{ height = 110, padding = 6 }) {
            label(tprint("Diagnostics (%)", doc.diagnostics.items.count));

            for scroll(.{ grow = 1 }) {
                for *diagnostic: doc.diagnostics.items {
                    pos := Lsp.read_position(Lsp.get(Lsp.get(diagnostic, "range"), "start"));
                    if list_item(tprint("%:%  %", pos.line + 1, pos.character + 1, Lsp.text(Lsp.get(diagnostic, "message"))), false) {
                        doc.edit.cursor = Lsp.byte_offset(contents(*doc.edit), pos);
                        doc.edit.anchor = doc.edit.cursor;
                    }
                }
            }
        }
    }
}

apply_completion :: (ed: *Editor, doc: *Document, item: *Json_Value) {
    if doc.edit.buffer.version != ed.completion_version return;

    // Additional edits (auto-import) are deliberately not silently dropped.
    extra := Lsp.get(item, "additionalTextEdits");
    if extra && extra.items.count {
        set_status(ed, "Completion needs additional edits; choose a local item");
        return;
    }

    if Lsp.number(Lsp.get(item, "insertTextFormat")) == 2 {
        set_status(ed, "Snippet completion unsupported");
        return;
    }

    start := ed.completion_start;
    end := ed.completion_end;
    insertion := Lsp.text(Lsp.get(item, "insertText"));

    if !insertion.count insertion = Lsp.text(Lsp.get(item, "label"));

    edit := Lsp.get(item, "textEdit");
    if edit {
        range := Lsp.get(edit, "range");
        if !range {
            set_status(ed, "Unsupported completion edit");
            return;
        }

        text := contents(*doc.edit);

        start = Lsp.byte_offset(text, Lsp.read_position(Lsp.get(range, "start")));
        end = Lsp.byte_offset(text, Lsp.read_position(Lsp.get(range, "end")));
        insertion = Lsp.text(Lsp.get(edit, "newText"));
    }

    doc.edit.anchor = start;
    doc.edit.cursor = end;
    insert_text(*doc.edit, insertion);
    clear_language_popup(ed);
}

read_format_file :: (path: string) -> string, bool {
    source, ok := read_entire_file(path);
    defer free(source);
    return copy_string(source,, temp), ok;
}

editor_format_config :: (path: string) -> Fmt.Format_Config, string, bool, string {
    push_allocator(temp);
    return Fmt.format_find_config(parent_directory(path), read_format_file);
}
