#scope_module

// Empty members represent an override that has not been specified.
proto_Features :: struct {
    presence: string;
    enums: string;
    repeated: string;
    utf8: string;
    message: string;
    json: string;
}

proto_parse_feature :: (s: *proto_Schema, f: *proto_Features, name: string) {
    // Custom feature extensions are opaque, just like custom options.
    if begins_with(name, "features.(") return;

    value := proto_peek(s);
    valid := false;

    if name == "features.field_presence" {
        valid = value.text == "EXPLICIT" || value.text == "IMPLICIT" || value.text == "LEGACY_REQUIRED";
        f.presence = value.text;
    } else if name == "features.enum_type" {
        valid = value.text == "OPEN" || value.text == "CLOSED";
        f.enums = value.text;
    } else if name == "features.repeated_field_encoding" {
        valid = value.text == "PACKED" || value.text == "EXPANDED";
        f.repeated = value.text;
    } else if name == "features.utf8_validation" {
        valid = value.text == "VERIFY" || value.text == "NONE";
        f.utf8 = value.text;
    } else if name == "features.message_encoding" {
        valid = value.text == "LENGTH_PREFIXED" || value.text == "DELIMITED";
        f.message = value.text;
    } else if name == "features.json_format" {
        valid = value.text == "ALLOW" || value.text == "LEGACY_BEST_EFFORT";
        f.json = value.text;
    } else {
        proto_schema_fail(s, value, proto_schema_format("unknown feature '%'", name));
        return;
    }

    if !valid proto_schema_fail(s, value, proto_schema_format("invalid value for '%'", name));
}

proto_inherit :: (base: proto_Features, overrides: proto_Features) -> proto_Features {
    result := base;
    if overrides.presence.count result.presence = overrides.presence;
    if overrides.enums.count result.enums = overrides.enums;
    if overrides.repeated.count result.repeated = overrides.repeated;
    if overrides.utf8.count result.utf8 = overrides.utf8;
    if overrides.message.count result.message = overrides.message;
    if overrides.json.count result.json = overrides.json;

    return result;
}

proto_resolve_features :: (s: *proto_Schema) {
    for source: s.sources {
        defaults := proto_Features.{ presence = "EXPLICIT", enums = "OPEN", repeated = "PACKED", utf8 = "VERIFY", message = "LENGTH_PREFIXED", json = "ALLOW" };

        if source.syntax == "proto2" {
            defaults.enums = "CLOSED";
            defaults.repeated = "EXPANDED";
            defaults.utf8 = "NONE";
            defaults.json = "LEGACY_BEST_EFFORT";
        } else if source.syntax == "proto3" defaults.presence = "IMPLICIT";

        source.features = proto_inherit(defaults, source.features);
    }

    // Parsing inserts parents before their children.
    for node: s.nodes {
        base := node.source.features;
        if node.parent base = node.parent.features;

        node.features = proto_inherit(base, node.features);
        for f: node.fields {
            base = node.features;
            for node.oneofs if it == f.oneof base = proto_inherit(base, node.oneof_features[it_index]);

            f.features = proto_inherit(base, f.features);

            if node.synthetic {
                f.features.message = "LENGTH_PREFIXED";
                f.features.presence = "IMPLICIT";
            }

            if f.required f.features.presence = "LEGACY_REQUIRED";

            f.required = !f.repeated && !f.map_entry && !f.oneof.count && f.features.presence == "LEGACY_REQUIRED";
            if !node.synthetic && !f.repeated && !f.map_entry && !f.oneof.count && f.features.presence != "IMPLICIT" f.optional = true;
            if !f.explicit_packed f.packed = f.features.repeated == "PACKED";
            if f.group f.features.message = "DELIMITED";

            // An extension is always a singly declared field with explicit presence.
            if node.kind == .EXTENSION {
                if f.features.presence == "LEGACY_REQUIRED" proto_schema_fail(s, f.token, "extensions cannot be required");
                f.features.presence = "EXPLICIT";
                f.required = false;
                f.optional = !f.repeated;
            }
        }
    }
}
