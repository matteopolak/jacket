plist_start :: (builder: *B.String_Builder) {
    checked_append(builder, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    checked_append(builder, "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    checked_append(builder, "<plist version=\"1.0\">\n<dict>\n");
}

plist_end :: (builder: *B.String_Builder) {
    checked_append(builder, "</dict>\n</plist>\n");
}

plist_write_key :: (builder: *B.String_Builder, key: string) {
    B.assert(xml_text_valid(key), "invalid XML property-list key");
    checked_append(builder, "  <key>");
    escaped(builder, key);
    checked_append(builder, "</key>\n");
}

plist_write_string :: (builder: *B.String_Builder, str: string) {
    B.assert(xml_text_valid(str), "invalid XML property-list string");
    checked_append(builder, "  <string>");
    escaped(builder, str);
    checked_append(builder, "</string>\n");
}

plist_write_bool :: (builder: *B.String_Builder, value: bool) {
    if value checked_append(builder, "  <true/>\n");
    else checked_append(builder, "  <false/>\n");
}

plist_write_key_value :: (builder: *B.String_Builder, key: string, value: Any) -> success: bool {
    if !xml_text_valid(key) || !value.type || !value.value_pointer return false;

    tag := value.type.type;
    if tag == .STRING {
        text := (cast(*string) value.value_pointer).*;
        if !xml_text_valid(text) return false;

        plist_write_key(builder, key);
        plist_write_string(builder, text);

        return true;
    }

    if tag == .BOOL {
        plist_write_key(builder, key);
        plist_write_bool(builder, (cast(*bool) value.value_pointer).*);
        return true;
    }

    if tag == .INTEGER {
        size := value.type.runtime_size;
        if size != 1 && size != 2 && size != 4 && size != 8 return false;

        plist_write_key(builder, key);
        checked_append(builder, "  <integer>");

        #if #exists(B.print_integer) {
            B.print_integer(builder, B.formatInt(value));
        } else B.print_item_to_builder(builder, value);

        B.assert(!builder.failed, "property-list builder allocation failed");
        checked_append(builder, "</integer>\n");

        return true;
    }

    if tag == .FLOAT {
        #if #exists(B.print_float) {
            number: float64;
            if value.type.runtime_size == 4 number = (cast(*float32) value.value_pointer).*;
            else if value.type.runtime_size == 8 number = (cast(*float64) value.value_pointer).*;
            else return false;
            if !Math.is_finite(number) return false;

            plist_write_key(builder, key);
            checked_append(builder, "  <real>");
            B.print_float(builder, B.formatFloat(value, mode = .SHORTEST));
            B.assert(!builder.failed, "property-list builder allocation failed");
            checked_append(builder, "</real>\n");

            return true;
        } else return false;
    }

    return false;
}

#scope_file
B :: #import "Basic";
U :: #import "Unicode";
Math :: #import "Math";

checked_append :: (builder: *B.String_Builder, text: string) {
    B.assert(builder != null);
    B.append(builder, text);
    B.assert(!builder.failed, "property-list builder allocation failed");
}

xml_text_valid :: (text: string) -> bool {
    cursor: s64;
    while cursor < text.count {
        code, consumed, status := U.character_utf8_to_utf32(text.data + cursor, text.count - cursor, true);
        if status != .CONVERSION_OK || consumed <= 0 return false;
        if code < 0x20 && code != 9 && code != 10 && code != 13 return false;
        if code == 0xfffe || code == 0xffff return false;

        cursor += consumed;
    }

    return true;
}

escaped :: (builder: *B.String_Builder, text: string) {
    for byte: text {
        if byte == #char "&" checked_append(builder, "&amp;");
        else if byte == #char "<" checked_append(builder, "&lt;");
        else if byte == #char ">" checked_append(builder, "&gt;");
        else if byte == #char "\"" checked_append(builder, "&quot;");
        else if byte == #char "'" checked_append(builder, "&apos;");
        else if byte == 13 checked_append(builder, "&#13;");
        else {
            B.append(builder, byte);
            B.assert(!builder.failed, "property-list builder allocation failed");
        }
    }
}
