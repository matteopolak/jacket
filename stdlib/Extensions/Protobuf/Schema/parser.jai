#scope_module

proto_Node_Kind :: enum {
    MESSAGE;
    ENUM;
}

proto_Node :: struct {
    kind: proto_Node_Kind;
    token: proto_Token;
    name: string;
    full: string;
    jai: string;
    parent: *proto_Node;
    children: [..] *proto_Node;
    fields: [..] *proto_Field;
    oneofs: [..] string;
    values: [..] proto_Enum_Value;
    reserved: [..] proto_Range;
    reserved_names: [..] string;
    allow_alias: bool;
    synthetic: bool;
    package: string;
    source: *proto_Source;
}

proto_Enum_Value :: struct {
    token: proto_Token;
    name: string;
    value: s64;
}

proto_Range :: struct {
    first: s64;
    last: s64;
}

proto_Field :: struct {
    token: proto_Token;
    name: string;
    jai: string;
    type_name: string;
    target: *proto_Node;
    scalar: string;
    number: s64;
    repeated: bool;
    optional: bool;
    packed: bool = true;
    explicit_packed: bool;
    oneof: string;
    json_name: string;
    has_json_name: bool;
    deprecated: bool;
    map_entry: *proto_Node;
}

proto_join_name :: (a: string, b: string) -> string {
    if !a.count return b;
    return proto_schema_format("%.%", a, b);
}

proto_new_node :: (s: *proto_Schema, parent: *proto_Node, kind: proto_Node_Kind, token: proto_Token) -> *proto_Node {
    node := New(proto_Node);
    node.kind = kind;
    node.token = token;
    node.name = token.text;
    node.parent = parent;
    node.package = s.package;
    node.source = s.source;
    scope := s.package;
    if parent scope = parent.full;

    node.full = proto_join_name(scope, node.name);
    if parent array_add(*parent.children, node);
    else array_add(*s.roots, node);

    array_add(*s.nodes, node);

    return node;
}

proto_skip_balanced :: (s: *proto_Schema, open: string, close: string) {
    if !proto_expect(s, open) return;

    depth := 1;
    while depth > 0 && !s.error.message.count {
        token := proto_take(s);
        if token.kind == .END {
            proto_schema_fail(s, token, proto_schema_format("expected '%'", close));
            break;
        }

        if token.kind == .STRING continue;
        if token.text == open depth += 1;
        if token.text == close depth -= 1;
    }
}

proto_skip_option_value :: (s: *proto_Schema) {
    if proto_peek(s).text == "{" {
        proto_skip_balanced(s, "{", "}");
        return;
    }

    if proto_peek(s).text == "[" {
        proto_skip_balanced(s, "[", "]");
        return;
    }

    if proto_eat(s, "-") {} else proto_eat(s, "+");

    token := proto_take(s);
    if token.kind == .END {
        proto_schema_fail(s, token, "expected option value");
        return;
    }

    if token.kind == .STRING while proto_peek(s).kind == .STRING proto_take(s);
    else while proto_eat(s, ".") proto_identifier(s);
}

proto_option_name :: (s: *proto_Schema) -> string {
    if proto_eat(s, "(") {
        name := proto_qualified(s);
        proto_expect(s, ")");
        while proto_eat(s, ".") proto_identifier(s);
        return proto_schema_format("(%)", name);
    }

    return proto_qualified(s);
}

proto_parse_options :: (s: *proto_Schema, field: *proto_Field) {
    if !proto_eat(s, "[") return;

    while !s.error.message.count {
        name := proto_option_name(s);

        proto_expect(s, "=");
        value := proto_peek(s);

        if name == "packed" {
            field.packed = value.text == "true";
            field.explicit_packed = true;
            if value.text != "true" && value.text != "false" proto_schema_fail(s, value, "packed must be true or false");
        }

        if name == "deprecated" field.deprecated = value.text == "true";

        if name == "json_name" {
            if value.kind != .STRING proto_schema_fail(s, value, "json_name must be a string");
            field.json_name = value.text;
            field.has_json_name = true;
        }

        proto_skip_option_value(s);
        if proto_eat(s, "]") break;
        if !proto_expect(s, ",") break;
    }
}

proto_parse_reserved :: (s: *proto_Schema, node: *proto_Node) {
    while !s.error.message.count {
        if proto_peek(s).kind == .STRING array_add(*node.reserved_names, proto_take(s).text);
        else {
            first := proto_number(s);
            last := first;

            if proto_eat(s, "to") {
                if proto_eat(s, "max") {
                    last = 536870911;
                    if node.kind == .ENUM last = 2147483647;
                } else last = proto_number(s);
            }

            if first > last proto_schema_fail(s, proto_peek(s), "reserved range is reversed");

            array_add(*node.reserved, .{ first, last });
        }

        if proto_eat(s, ";") break;
        if !proto_expect(s, ",") break;
    }
}

proto_parse_field :: (s: *proto_Schema, node: *proto_Node, oneof: string = "") {
    field := New(proto_Field);
    field.token = proto_peek(s);
    field.oneof = oneof;
    if proto_eat(s, "repeated") field.repeated = true;
    else if proto_eat(s, "optional") field.optional = true;
    else if proto_eat(s, "required") {
        proto_schema_fail(s, field.token, "required fields are not supported yet (proto2)");
        return;
    }

    if oneof.count && (field.repeated || field.optional || proto_peek(s).text == "map") {
        proto_schema_fail(s, field.token, "repeated, optional and map fields are not allowed in a oneof");
        return;
    }

    if proto_eat(s, "group") {
        proto_schema_fail(s, field.token, "groups are not supported yet");
        return;
    }

    key: string;
    value: string;
    is_map := proto_eat(s, "map");

    if is_map {
        if field.repeated || field.optional proto_schema_fail(s, field.token, "map cannot have a field label");

        proto_expect(s, "<");
        key = proto_qualified(s);
        proto_expect(s, ",");
        value = proto_qualified(s);
        proto_expect(s, ">");
    } else field.type_name = proto_qualified(s);

    field.name = proto_identifier(s).text;
    proto_expect(s, "=");
    field.number = proto_number(s);
    proto_parse_options(s, field);
    proto_expect(s, ";");

    if is_map {
        tok := field.token;
        tok.text = proto_schema_format("proto_map_%", field.number);
        entry := proto_new_node(s, node, .MESSAGE, tok);
        entry.synthetic = true;
        k := New(proto_Field);
        k.token = field.token;
        k.name = "key";
        k.type_name = key;
        k.number = 1;
        v := New(proto_Field);
        v.token = field.token;
        v.name = "value";
        v.type_name = value;
        v.number = 2;
        array_add(*entry.fields, k);
        array_add(*entry.fields, v);
        field.map_entry = entry;
    }

    array_add(*node.fields, field);
}

proto_parse_node :: (s: *proto_Schema, parent: *proto_Node, kind: proto_Node_Kind) {
    node := proto_new_node(s, parent, kind, proto_identifier(s));

    if !proto_expect(s, "{") return;

    while !s.error.message.count && !proto_eat(s, "}") {
        tok := proto_peek(s);
        if tok.kind == .END {
            proto_schema_fail(s, tok, "unterminated type declaration");
            break;
        }

        if proto_eat(s, ";") continue;

        if proto_eat(s, "reserved") {
            proto_parse_reserved(s, node);
            continue;
        }

        if proto_eat(s, "option") {
            name := proto_option_name(s);

            proto_expect(s, "=");
            if name == "allow_alias" node.allow_alias = proto_peek(s).text == "true";

            proto_skip_option_value(s);
            proto_expect(s, ";");
            continue;
        }

        if kind == .ENUM {
            entry: proto_Enum_Value;
            entry.token = proto_identifier(s);
            entry.name = entry.token.text;
            proto_expect(s, "=");
            entry.value = proto_number(s);
            unused: proto_Field;

            proto_parse_options(s, *unused);
            proto_expect(s, ";");
            array_add(*node.values, entry);
            continue;
        }

        if proto_eat(s, "message") {
            proto_parse_node(s, node, .MESSAGE);
            continue;
        }

        if proto_eat(s, "enum") {
            proto_parse_node(s, node, .ENUM);
            continue;
        }

        if proto_eat(s, "oneof") {
            name := proto_identifier(s).text;

            array_add(*node.oneofs, name);
            proto_expect(s, "{");

            while !s.error.message.count && !proto_eat(s, "}") {
                if proto_eat(s, "option") {
                    proto_option_name(s);
                    proto_expect(s, "=");
                    proto_skip_option_value(s);
                    proto_expect(s, ";");
                } else if !proto_eat(s, ";") proto_parse_field(s, node, name);
            }

            continue;
        }

        if tok.text == "extend" || tok.text == "extensions" {
            proto_schema_fail(s, tok, "extensions are not supported yet");
            break;
        }

        proto_parse_field(s, node);
    }
}

proto_parse_schema :: (s: *proto_Schema) {
    syntax_seen := false;

    while !s.error.message.count && proto_peek(s).kind != .END {
        tok := proto_take(s);
        if tok.text == ";" continue;

        if tok.text == "syntax" {
            if syntax_seen {
                proto_schema_fail(s, tok, "duplicate syntax declaration");
                break;
            }

            syntax_seen = true;
            proto_expect(s, "=");
            value := proto_take(s);
            if value.text == "proto2" proto_schema_fail(s, value, "proto2 is not supported yet");
            else if value.kind != .STRING || value.text != "proto3" proto_schema_fail(s, value, "expected syntax = 'proto3'");

            proto_expect(s, ";");
        } else if tok.text == "edition" proto_schema_fail(s, tok, "editions are not supported yet");
        else if tok.text == "import" {
            entry := proto_Import.{ token = tok };
            entry.public = proto_eat(s, "public");
            if !entry.public proto_eat(s, "weak");

            path := proto_take(s);
            if path.kind != .STRING proto_schema_fail(s, path, "expected import path string");

            entry.name = path.text;
            proto_expect(s, ";");
            array_add(*s.imports, entry);
        } else if tok.text == "extend" proto_schema_fail(s, tok, "extensions are not supported yet");
        else if tok.text == "package" {
            if s.package.count || s.nodes.count {
                proto_schema_fail(s, tok, "package must precede type declarations");
                break;
            }

            s.package = proto_qualified(s);
            proto_expect(s, ";");
        } else if tok.text == "message" proto_parse_node(s, null, .MESSAGE);
        else if tok.text == "enum" proto_parse_node(s, null, .ENUM);
        else if tok.text == "option" {
            proto_option_name(s);
            proto_expect(s, "=");
            proto_skip_option_value(s);
            proto_expect(s, ";");
        } else if tok.text == "service" {
            proto_identifier(s);
            proto_skip_balanced(s, "{", "}");
        } else proto_schema_fail(s, tok, "expected syntax, package, option, message, enum or service");
    }

    if !syntax_seen && !s.error.message.count proto_schema_fail(s, s.tokens[0], "proto2 is not supported yet; specify syntax = 'proto3'");
}
