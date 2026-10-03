#scope_module
am16 :: (p: *u8) -> u16 {
    return (cast(u16) p[0]) | ((cast(u16) p[1]) << 8);
}

am32 :: (p: *u8) -> u32 {
    return (cast(u32) am16(p)) | ((cast(u32) am16(p + 2)) << 16);
}

am_ref :: (m: *Android_Manifest_Binary, index: u32, optional := false) -> bool {
    return (optional && index == DUMMY_U32) || index < m.string_pool.count;
}

am_read :: (m: *Android_Manifest_Binary) -> bool {
    s := m.raw_data;
    if s.count < 8 || !s.data || am16(s.data) != 3 || am16(s.data + 2) != 8 || am32(s.data + 4) != s.count return false;

    at: s64 = 8;
    stack: [..] u64;

    defer array_free(stack);
    namespace_stack: [..] u64;

    defer array_free(namespace_stack);
    pool_seen := false;
    roots := 0;

    while at < s.count {
        if s.count - at < 8 return false;

        p := s.data + at;
        kind := am16(p);
        header := cast(s64) am16(p + 2);
        size := cast(s64) am32(p + 4);

        if header < 8 || size < header || size > s.count - at || size % 4 != 0 return false;

        if kind == 1 {
            if pool_seen || header < 28 return false;

            pool_seen = true;
            count := cast(s64) am32(p + 8);
            styles := cast(s64) am32(p + 12);
            start := cast(s64) am32(p + 20);
            end := cast(s64) am32(p + 24);

            if count > (size - header) / 4 || styles > (size - header - count * 4) / 4 || start < header + (count + styles) * 4 || start > size return false;
            if end == 0 end = size;
            if end < start || end > size return false;
            // Styled text is outside the manifest reader's supported subset.
            if styles != 0 return false;

            pool_memory := cast(*string) alloc(count * size_of(string));
            if count > 0 && !pool_memory return false;

            m.string_pool = .{ count, pool_memory };

            memset(m.string_pool.data, 0, count * size_of(string));

            for i: 0..count - 1 {
                off := cast(s64) am32(p + header + i * 4);
                if off >= end - start return false;

                pos := start + off;
                b: String_Builder;

                defer free_buffers(*b);

                if (am32(p + 16) & 0x100) != 0 {
                    units, pos, valid := am_length(p, pos, end, true);
                    if !valid return false;

                    length: s64;
                    length, pos, valid = am_length(p, pos, end, true);
                    if !valid || length >= end - pos || p[pos + length] != 0 return false;

                    utf_ok, actual_units := am_utf8_units(string.{ length, p + pos });
                    if !utf_ok || actual_units != units return false;

                    append(*b, p + pos, length);
                } else {
                    length, next, valid := am_length(p, pos, end, false);
                    if !valid || end - next < 2 || length > (end - next - 2) / 2 return false;

                    pos = next;
                    j: s64;
                    while j < length {
                        cp := cast(u32) am16(p + pos + j * 2);
                        if cp >= 0xd800 && cp <= 0xdbff {
                            if j + 1 >= length return false;

                            low := cast(u32) am16(p + pos + (j + 1) * 2);
                            if low < 0xdc00 || low > 0xdfff return false;

                            cp = 0x10000 + ((cp - 0xd800) << 10) + low - 0xdc00;
                            j += 1;
                        } else if cp >= 0xdc00 && cp <= 0xdfff return false;

                        am_utf8(*b, cp);
                        j += 1;
                    }

                    if am16(p + pos + length * 2) != 0 return false;
                }

                m.string_pool[i] = builder_to_string(*b);
            }
        } else if kind == 0x180 {
            if !pool_seen || m.resource_ids.data || header != 8 || (size - header) / 4 > m.string_pool.count return false;

            count := (size - 8) / 4;

            m.resource_ids = .{ count, cast(*u32) alloc(count * 4) };
            if count > 0 && !m.resource_ids.data return false;
            for i: 0..count - 1 m.resource_ids[i] = am32(p + 8 + i * 4);
        } else {
            if !pool_seen || header < 16 return false;

            q := p + header;

            if kind == 0x100 || kind == 0x101 {
                if size - header < 8 || !am_ref(m, am32(q)) || !am_ref(m, am32(q + 4)) return false;

                namespace_key := (cast(u64) am32(q)) | ((cast(u64) am32(q + 4)) << 32);

                if kind == 0x100 {
                    array_add(*namespace_stack, namespace_key);
                    uri := m.string_pool[am32(q + 4)];
                    if uri == ANDROID_XML_SCHEMA m.android_namespace = .{ m.string_pool[am32(q)], uri };
                } else {
                    if namespace_stack.count == 0 || namespace_stack[namespace_stack.count - 1] != namespace_key return false;
                    namespace_stack.count -= 1;
                }
            } else if kind == 0x102 || kind == 0x103 {
                if size - header < 8 || !am_ref(m, am32(q), true) || !am_ref(m, am32(q + 4)) return false;

                name := am32(q + 4);
                element_key := (cast(u64) name) | ((cast(u64) am32(q)) << 32);
                e: Android_Manifest_Binary.Xml_Element;
                e.name = m.string_pool[name];

                if kind == 0x102 {
                    if stack.count == 0 roots += 1;

                    array_add(*stack, element_key);
                    if size - header < 20 return false;

                    first := cast(s64) am16(q + 8);
                    stride := cast(s64) am16(q + 10);
                    count := cast(s64) am16(q + 12);

                    if first < 20 || stride < 20 || first > size - header || count > (size - header - first) / stride return false;
                    for k: 0..2 if am16(q + 14 + k * 2) > count return false;

                    e.attributes = .{ count, cast(*ResXMLTree_attribute) alloc(count * size_of(ResXMLTree_attribute)) };
                    if count > 0 && !e.attributes.data return false;

                    for i: 0..count - 1 {
                        a := q + first + i * stride;
                        v := ResXMLTree_attribute.{ .{ am32(a) }, .{ am32(a + 4) }, .{ am32(a + 8) }, .{ am16(a + 12), a[14], a[15], am32(a + 16) } };

                        if !am_ref(m, v.ns.index, true) || !am_ref(m, v.name.index) || !am_ref(m, v.rawValue.index, true) || v.typedValue.size != 8 || v.typedValue.res0 != 0 || (v.typedValue.dataType == 3 && !am_ref(m, v.typedValue.data)) {
                            free(e.attributes.data);
                            return false;
                        }

                        e.attributes[i] = v;
                    }
                } else {
                    e.kind = .END;
                    if stack.count == 0 || stack[stack.count - 1] != element_key return false;
                    stack.count -= 1;
                }

                array_add(*m.elements, e);
            } else if kind == 0x104 {
                if size - header < 12 || !am_ref(m, am32(q)) return false;
            } else return false;
        }

        at += size;
    }

    return pool_seen && roots == 1 && stack.count == 0 && namespace_stack.count == 0;
}

am_length :: (p: *u8, pos: s64, end: s64, utf8: bool) -> s64, s64, bool {
    unit: s64 = ifx utf8 then 1 else 2;
    if end - pos < unit return 0, pos, false;

    n := cast(s64) p[pos];
    if !utf8 n = am16(p + pos);

    pos += unit;
    flag: s64 = ifx utf8 then 0x80 else 0x8000;
    if (n & flag) != 0 {
        if end - pos < unit return 0, pos, false;

        tail := cast(s64) p[pos];
        if !utf8 tail = am16(p + pos);

        n = ((n & (flag - 1)) * flag * 2) + tail;
        pos += unit;
    }

    return n, pos, true;
}

am_utf8 :: (b: *String_Builder, cp: u32) {
    bytes: [4] u8;
    n := 1;

    if cp < 0x80 bytes[0] = cast(u8) cp;
    else {
        if cp < 0x800 {
            n = 2;
            bytes[0] = cast(u8) (0xc0 | (cp >> 6));
        } else if cp < 0x10000 {
            n = 3;
            bytes[0] = cast(u8) (0xe0 | (cp >> 12));
        } else {
            n = 4;
            bytes[0] = cast(u8) (0xf0 | (cp >> 18));
        }

        for i: 1..n - 1 bytes[i] = cast(u8) (0x80 | ((cp >> ((n - i - 1) * 6)) & 0x3f));
    }

    append(b, *bytes[0], n);
}

#import "Basic";

am_utf8_units :: (s: string) -> bool, u32 {
    pos: s64;
    units: u32;

    while pos < s.count {
        first := s[pos];
        if first < 0x80 {
            pos += 1;
            units += 1;
            continue;
        }

        count := 0;
        cp: u32;
        minimum: u32;

        if first >= 0xc2 && first <= 0xdf {
            count = 2;
            cp = first & 31;
            minimum = 0x80;
        } else if first >= 0xe0 && first <= 0xef {
            count = 3;
            cp = first & 15;
            minimum = 0x800;
        } else if first >= 0xf0 && first <= 0xf4 {
            count = 4;
            cp = first & 7;
            minimum = 0x10000;
        } else return false, 0;

        if count > s.count - pos return false, 0;

        for i: 1..count - 1 {
            byte := s[pos + i];
            if byte < 0x80 || byte > 0xbf return false, 0;
            cp = (cp << 6) | (byte & 63);
        }

        if cp < minimum || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff) return false, 0;

        units += ifx cp >= 0x10000 then 2 else 1;
        pos += count;
    }

    return true, units;
}
