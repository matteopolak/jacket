// Every type carries a Type_Info at runtime. Enough to write a serializer in 40 lines.

Save_Game :: struct {
    player: string;
    level: int;
    position: Vec2;
    inventory: [] string;
    weather: Weather;
    hardcore: bool;
    secret_note: string; @NoSave // notes are free-form tags that reflection can read
}

tour_reflection :: () {
    // Types are values: store them, compare them, print them.
    kinds := Type.[int, float, string, Vec2, [] int, *Save_Game, Weather];
    for kinds {
        info := cast(*Type_Info) it;
        print("  % is %, % bytes\n", it, info.type, info.runtime_size);
    }

    // Walk a struct's members by name, type and offset.
    print("Save_Game members:\n");
    for member: type_info(Save_Game).members {
        offset := formatInt(member.offset_in_bytes, minimum_digits = 2);
        print("  +%  %: %", offset, member.name, member.type.type);
        for member.notes print("  @%", it);
        print("\n");
    }

    // `Any` holds a pointer to a value plus its type: a type-safe `void*`.
    save := Save_Game.{
        player = "Zaphod",
        level = 42,
        position = .{ 1.5, -2 },
        inventory = .["towel", "babel fish", "peanuts"],
        weather = .RAINY,
        hardcore = true,
        secret_note = "do not save this",
    };
    builder: String_Builder;
    write_json(*builder, save);
    json := builder_to_string(*builder);
    defer free(json);
    print("As JSON, via reflection:\n  %\n", json);
}

// Writes any value as JSON by asking its Type_Info what it is.
write_json :: (builder: *String_Builder, value: Any) {
    info := value.type;
    if info.type == {
        case .INTEGER; #through;
        case .FLOAT; #through;
        case .BOOL;
            print_to_builder(builder, "%", value);
        case .STRING;
            print_to_builder(builder, "\"%\"", value);
        case .ENUM;
            print_to_builder(builder, "\"%\"", value);
        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            // Fixed arrays keep their elements inline; views and [..] arrays start
            // with a count and a data pointer.
            count, data := array.array_count, value.value_pointer;
            if array.array_type != .FIXED {
                view := cast(*Array_View_64) value.value_pointer;
                count, data = view.count, view.data;
            }
            append(builder, "[");
            for 0..count - 1 {
                if it append(builder, ", ");
                element: Any;
                element.type = array.element_type;
                element.value_pointer = data + it * array.element_type.runtime_size;
                write_json(builder, element);
            }
            append(builder, "]");
        case .STRUCT;
            struct_info := cast(*Type_Info_Struct) info;
            append(builder, "{");
            written := 0;
            for member: struct_info.members {
                if array_find(member.notes, "NoSave") continue;
                if written append(builder, ", ");
                print_to_builder(builder, "\"%\": ", member.name);
                field: Any;
                field.type = member.type;
                field.value_pointer = value.value_pointer + member.offset_in_bytes;
                write_json(builder, field);
                written += 1;
            }
            append(builder, "}");
        case;
            append(builder, "null");
    }
}
