// Undo and redo for Text_Buffer. Every edit is recorded as (offset, removed text, inserted text);
// undoing applies the inverse. Edits in one group, or typing that merges, undo together.

// One recorded change: `removed` was replaced by `inserted` at `offset`. Edits with the same
// `group` undo together.
Edit :: struct {
    offset: int;
    removed: string; // heap copies, released with the history
    inserted: string;
    group: int;
}

// A buffer's undo and redo stacks; zero is empty. Use `undo`, `redo`, `begin_group` and
// `end_group` rather than reading it.
History :: struct {
    undo_stack: [..] Edit;
    redo_stack: [..] Edit;
    next_group: int;
    open_depth: int; // begin_group nesting
    open_group: int;
    mergeable: bool; // the last recorded edit may absorb the next one
}

// Release the history's stacks and the text of every edit in them.
free_history :: (history: *History) {
    for history.undo_stack free_edit(it);
    for history.redo_stack free_edit(it);

    array_free(history.undo_stack);
    array_free(history.redo_stack);
    history.* = .{};
}

// Start an undo group: every edit until the matching `end_group` undoes as one step. Groups nest.
begin_group :: (buffer: *Text_Buffer) {
    h := *buffer.history;

    if h.open_depth == 0 {
        h.next_group += 1;
        h.open_group = h.next_group;
        h.mergeable = false;
    }

    h.open_depth += 1;
}

// End the group started by `begin_group`.
end_group :: (buffer: *Text_Buffer) {
    h := *buffer.history;
    if h.open_depth > 0 h.open_depth -= 1;
    h.mergeable = false;
}

// Stop the next typed character from merging into the previous undo step. Call it when the
// cursor moves, so undo steps follow what the user did.
seal_undo :: (buffer: *Text_Buffer) {
    buffer.history.mergeable = false;
}

// Whether `undo` has anything to undo.
can_undo :: (buffer: *Text_Buffer) -> bool {
    return buffer.history.undo_stack.count > 0;
}

// Whether `redo` has anything to redo.
can_redo :: (buffer: *Text_Buffer) -> bool {
    return buffer.history.redo_stack.count > 0;
}

// Undo the last step. Returns whether anything was undone and the offset the cursor belongs at.
undo :: (buffer: *Text_Buffer) -> ok: bool, cursor: int {
    h := *buffer.history;
    if !h.undo_stack.count return false, 0;

    h.mergeable = false;
    group := h.undo_stack[h.undo_stack.count - 1].group;
    cursor := 0;

    while h.undo_stack.count && h.undo_stack[h.undo_stack.count - 1].group == group {
        edit := pop(*h.undo_stack);
        if edit.inserted.count apply_erase(buffer, edit.offset, edit.inserted.count);
        if edit.removed.count apply_insert(buffer, edit.offset, edit.removed);

        cursor = edit.offset + edit.removed.count;
        array_add(*h.redo_stack, edit);
    }

    return true, cursor;
}

// Redo the last undone step. Returns whether anything was redone and the offset the cursor belongs at.
redo :: (buffer: *Text_Buffer) -> ok: bool, cursor: int {
    h := *buffer.history;
    if !h.redo_stack.count return false, 0;

    h.mergeable = false;
    group := h.redo_stack[h.redo_stack.count - 1].group;
    cursor := 0;

    while h.redo_stack.count && h.redo_stack[h.redo_stack.count - 1].group == group {
        edit := pop(*h.redo_stack);
        if edit.removed.count apply_erase(buffer, edit.offset, edit.removed.count);
        if edit.inserted.count apply_insert(buffer, edit.offset, edit.inserted);

        cursor = edit.offset + edit.inserted.count;
        array_add(*h.undo_stack, edit);
    }

    return true, cursor;
}

#scope_module

free_edit :: (edit: Edit) {
    free(edit.removed.data);
    free(edit.inserted.data);
}

record_edit :: (buffer: *Text_Buffer, offset: int, removed: string, inserted: string) {
    h := *buffer.history;

    for h.redo_stack free_edit(it);

    array_reset(*h.redo_stack);

    if h.open_depth > 0 {
        array_add(*h.undo_stack, make_edit(offset, removed, inserted, h.open_group));
        return;
    }

    if h.mergeable && h.undo_stack.count && merge_into(*h.undo_stack[h.undo_stack.count - 1], offset, removed, inserted) {
        return;
    }

    h.next_group += 1;
    array_add(*h.undo_stack, make_edit(offset, removed, inserted, h.next_group));

    // Only single-character typing and single-character deletes merge; pastes and cuts stand alone.
    h.mergeable = (inserted.count > 0 && !removed.count && inserted.count <= 4 && !contains_break(inserted))
        || (removed.count > 0 && !inserted.count && removed.count <= 4 && !contains_break(removed));
}

make_edit :: (offset: int, removed: string, inserted: string, group: int) -> Edit {
    return .{ offset, copy_string(removed), copy_string(inserted), group };
}

contains_break :: (text: string) -> bool {
    for 0..text.count - 1 if text.data[it] == #char "\n" return true;
    return false;
}

// Typing continues a typed run; backspace and forward delete continue a deleted run.
merge_into :: (top: *Edit, offset: int, removed: string, inserted: string) -> bool {
    small := (inserted.count > 0 && inserted.count <= 4 && !contains_break(inserted)) || (removed.count > 0 && removed.count <= 4 && !contains_break(removed));
    if !small return false;

    if top.removed.count == 0 && removed.count == 0 && inserted.count > 0 && top.offset + top.inserted.count == offset {
        old := top.inserted;
        top.inserted = joined(old, inserted);
        free(old.data);
        return true;
    }

    if top.inserted.count == 0 && inserted.count == 0 && removed.count > 0 {
        if offset + removed.count == top.offset {
            old := top.removed;

            top.removed = joined(removed, old);
            free(old.data);
            top.offset = offset;

            return true;
        }

        if offset == top.offset {
            old := top.removed;
            top.removed = joined(old, removed);
            free(old.data);
            return true;
        }
    }

    return false;
}

copy_string :: (text: string) -> string {
    if !text.count return "";

    result: string;
    result.count = text.count;
    result.data = alloc(text.count);
    memcpy(result.data, text.data, text.count);

    return result;
}

// `a` followed by `b` as a new heap string.
joined :: (a: string, b: string) -> string {
    result: string;
    result.count = a.count + b.count;
    result.data = alloc(max(result.count, 1));
    memcpy(result.data, a.data, a.count);
    memcpy(result.data + a.count, b.data, b.count);

    return result;
}
