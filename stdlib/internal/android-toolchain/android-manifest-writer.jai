#scope_module
AM_Writer :: struct {
    strings: [..] string;
    ids: [..] u32;
    attributes: [..] ResXMLTree_attribute;
    nodes: String_Builder;
}

am_put16 :: (b: *String_Builder, v: u16) {
    bytes: [2] u8;
    bytes[0] = cast(u8) (v & 255);
    bytes[1] = cast(u8) (v >> 8);
    append(b, *bytes[0], 2);
}

am_put32 :: (b: *String_Builder, v: u32) {
    am_put16(b, cast(u16) (v & 65535));
    am_put16(b, cast(u16) (v >> 16));
}

am_header :: (b: *String_Builder, kind: u16, header: u16, size: u32) {
    am_put16(b, kind);
    am_put16(b, header);
    am_put32(b, size);
}

am_node :: (b: *String_Builder, kind: u16, size: u32) {
    am_header(b, kind, 16, size);
    am_put32(b, 0);
    am_put32(b, DUMMY_U32);
}

am_string :: (w: *AM_Writer, value: string) -> u32 {
    for s: w.strings if s == value return cast(u32) it_index;
    array_add(*w.strings, value);
    array_add(*w.ids, 0);
    return cast(u32) (w.strings.count - 1);
}

am_id :: (name: string) -> u32 {
    if name == "theme" return 0x01010000;
    if name == "label" return 0x01010001;
    if name == "name" return 0x01010003;
    if name == "hasCode" return 0x0101000c;
    if name == "exported" return 0x01010010;
    if name == "launchMode" return 0x0101001d;
    if name == "screenOrientation" return 0x0101001e;
    if name == "configChanges" return 0x0101001f;
    if name == "value" return 0x01010024;
    if name == "minSdkVersion" return 0x0101020c;
    if name == "versionCode" return 0x0101021b;
    if name == "versionName" return 0x0101021c;
    if name == "targetSdkVersion" return 0x01010270;
    if name == "required" return 0x0101028e;

    return 0;
}

am_attr :: (w: *AM_Writer, name: string, value := "", kind: u8 = 3, data: u32 = 0, android := true) {
    a: ResXMLTree_attribute;
    a.ns.index = DUMMY_U32;
    if android a.ns.index = am_string(w, ANDROID_XML_SCHEMA);

    a.name.index = am_string(w, name);
    if android w.ids[a.name.index] = am_id(name);

    a.rawValue.index = DUMMY_U32;
    a.typedValue = .{ 8, 0, kind, data };

    if kind == 3 {
        a.rawValue.index = am_string(w, value);
        a.typedValue.data = a.rawValue.index;
    }

    array_add(*w.attributes, a);
}

am_start :: (w: *AM_Writer, name: string) {
    b := *w.nodes;

    am_node(b, 0x102, cast(u32) (36 + w.attributes.count * 20));
    am_put32(b, DUMMY_U32);
    am_put32(b, am_string(w, name));
    am_put16(b, 20);
    am_put16(b, 20);
    am_put16(b, cast(u16) w.attributes.count);
    am_put16(b, 0);
    am_put16(b, 0);
    am_put16(b, 0);

    // Framework attributes are ordered by resource identifier.
    for i: 0..w.attributes.count - 1 {
        for j: i + 1..w.attributes.count - 1 {
            if w.ids[w.attributes[j].name.index] < w.ids[w.attributes[i].name.index] {
                swap := w.attributes[i];
                w.attributes[i] = w.attributes[j];
                w.attributes[j] = swap;
            }
        }
    }

    for a: w.attributes {
        am_put32(b, a.ns.index);
        am_put32(b, a.name.index);
        am_put32(b, a.rawValue.index);
        am_put16(b, 8);
        am_put16(b, cast(u16) a.typedValue.dataType << 8);
        am_put32(b, a.typedValue.data);
    }

    w.attributes.count = 0;
}

am_end :: (w: *AM_Writer, name: string) {
    am_node(*w.nodes, 0x103, 24);
    am_put32(*w.nodes, DUMMY_U32);
    am_put32(*w.nodes, am_string(w, name));
}

am_namespace :: (w: *AM_Writer, kind: u16) {
    am_node(*w.nodes, kind, 24);
    am_put32(*w.nodes, am_string(w, "android"));
    am_put32(*w.nodes, am_string(w, ANDROID_XML_SCHEMA));
}

am_write :: (out: *String_Builder, c: Android_Project_Config) -> bool {
    if c.app_id.count == 0 || c.version_code < 0 || c.min_sdk_level < 1 || c.target_sdk_level < c.min_sdk_level return false;

    if c.android_manifest.count > 0 {
        ok, manifest := read_binary_manifest_from_memory(c.android_manifest);
        if !ok return false;

        defer deinit(*manifest);
        append(out, c.android_manifest);

        return !out.failed;
    }

    w: AM_Writer;

    defer array_free(w.strings);
    defer array_free(w.ids);
    defer array_free(w.attributes);
    defer free_buffers(*w.nodes);
    am_namespace(*w, 0x100);
    am_attr(*w, "package", c.app_id, android = false);
    am_attr(*w, "versionCode", kind = 0x10, data = cast(u32) c.version_code);
    am_attr(*w, "versionName", c.version_name);
    am_start(*w, "manifest");
    am_attr(*w, "minSdkVersion", kind = 0x10, data = cast(u32) c.min_sdk_level);
    am_attr(*w, "targetSdkVersion", kind = 0x10, data = cast(u32) c.target_sdk_level);
    am_start(*w, "uses-sdk");
    am_end(*w, "uses-sdk");

    for permission: c.permissions {
        am_attr(*w, "name", permission);
        am_start(*w, "uses-permission");
        am_end(*w, "uses-permission");
    }

    for feature: c.features {
        am_attr(*w, "name", feature.name);
        am_attr(*w, "required", kind = 0x12, data = ifx feature.required then 0xffffffff else 0);
        am_start(*w, "uses-feature");
        am_end(*w, "uses-feature");
    }

    am_attr(*w, "label", c.app_name);
    am_attr(*w, "hasCode", kind = 0x12);
    am_attr(*w, "theme", kind = 1, data = 0x01030000 | cast(u32) c.theme);
    am_start(*w, "application");

    for metadata: c.application_meta_data {
        am_attr(*w, "name", metadata.name);
        am_attr(*w, "value", metadata.value);
        am_start(*w, "meta-data");
        am_end(*w, "meta-data");
    }

    if c.vr {
        am_attr(*w, "name", "com.oculus.intent.category.VR");
        am_attr(*w, "value", "vr_only");
        am_start(*w, "meta-data");
        am_end(*w, "meta-data");
    }

    if c.quest_enable_passthrough_splash {
        am_attr(*w, "name", "com.oculus.ossplash.passthrough_context");
        am_attr(*w, "value", "true");
        am_start(*w, "meta-data");
        am_end(*w, "meta-data");
    }

    am_attr(*w, "name", "android.app.NativeActivity");
    am_attr(*w, "exported", kind = 0x12, data = 0xffffffff);
    am_attr(*w, "launchMode", kind = 0x10, data = LAUNCH_MODE_SINGLE_TASK);
    am_attr(*w, "screenOrientation", kind = 0x10, data = cast(u32) (cast(s32) c.screen_orientation - 1));
    am_attr(*w, "configChanges", kind = 0x11, data = 0x00000da0);
    am_start(*w, "activity");
    am_attr(*w, "name", "android.app.lib_name");
    am_attr(*w, "value", c.lib_name);
    am_start(*w, "meta-data");
    am_end(*w, "meta-data");
    am_start(*w, "intent-filter");
    am_attr(*w, "name", "android.intent.action.MAIN");
    am_start(*w, "action");
    am_end(*w, "action");
    am_attr(*w, "name", "android.intent.category.LAUNCHER");
    am_start(*w, "category");
    am_end(*w, "category");

    if c.vr {
        am_attr(*w, "name", "com.oculus.intent.category.VR");
        am_start(*w, "category");
        am_end(*w, "category");
    }

    am_end(*w, "intent-filter");
    am_end(*w, "activity");
    am_end(*w, "application");
    am_end(*w, "manifest");
    am_namespace(*w, 0x101);
    pool: String_Builder;
    offsets: String_Builder;

    defer free_buffers(*pool);
    defer free_buffers(*offsets);

    for s: w.strings {
        // UTF-8 pool lengths are limited to the format's 15-bit byte count.
        if s.count > 32767 return false;

        valid, units := am_utf8_units(s);
        if !valid return false;

        am_put32(*offsets, cast(u32) builder_string_length(*pool));
        am_len8(*pool, units);
        am_len8(*pool, cast(u32) s.count);
        append(*pool, s);
        append(*pool, "\0");
    }

    while builder_string_length(*pool) % 4 != 0 append(*pool, "\0");

    pool_size := 28 + w.strings.count * 4 + builder_string_length(*pool);
    total := 8 + pool_size + 8 + w.ids.count * 4 + builder_string_length(*w.nodes);

    if total > 0xffffffff || w.nodes.failed || pool.failed || offsets.failed return false;

    am_header(out, 3, 8, cast(u32) total);
    am_header(out, 1, 28, cast(u32) pool_size);
    am_put32(out, cast(u32) w.strings.count);
    am_put32(out, 0);
    am_put32(out, 0x100);
    am_put32(out, cast(u32) (28 + w.strings.count * 4));
    am_put32(out, 0);
    offset_bytes := builder_to_string(*offsets);
    pool_bytes := builder_to_string(*pool);
    node_bytes := builder_to_string(*w.nodes);

    defer free(offset_bytes);
    defer free(pool_bytes);
    defer free(node_bytes);
    append(out, offset_bytes);
    append(out, pool_bytes);
    am_header(out, 0x180, 8, cast(u32) (8 + w.ids.count * 4));
    for id: w.ids am_put32(out, id);

    append(out, node_bytes);

    return !out.failed;
}

am_len8 :: (b: *String_Builder, length: u32) {
    bytes: [2] u8;

    if length < 128 {
        bytes[0] = cast(u8) length;
        append(b, *bytes[0], 1);
    } else {
        bytes[0] = cast(u8) ((length >> 8) | 0x80);
        bytes[1] = cast(u8) (length & 255);
        append(b, *bytes[0], 2);
    }
}

#import "Basic";
