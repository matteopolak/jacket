// A deliberately small two-pass assembler: hexadecimal op/byte plus labelled control flow.
#import "Basic";
#import "String";

Label :: struct {
    name: string;
    address: int;
}

hex_number :: (text: string) -> int, bool {
    if text.count == 0 return 0, false;

    result := 0;

    for text {
        digit := cast(int) it;
        if digit >= 48 && digit <= 57 digit -= 48;
        else if digit >= 65 && digit <= 70 digit -= 55;
        else if digit >= 97 && digit <= 102 digit -= 87;
        else return 0, false;
        if result > 0xFFFF return 0, false;

        result = result * 16 + digit;
    }

    return result, true;
}

// Returns owned bytes and an owned error string (both freed by the caller).
assemble :: (source: string) -> string, string {
    labels: [..] Label;
    bytes: [..] u8;

    defer array_free(labels);
    defer array_free(bytes);
    lines := split(source, "\n");

    defer array_free(lines);

    for pass: 0..1 {
        address := 0x200;

        for lines {
            line := trim(it);
            comment := find_index_from_left(line, #char ";");

            if comment >= 0 line = trim(slice(line, 0, comment));
            if line.count == 0 continue;

            if ends_with(line, ":") {
                name := slice(line, 0, line.count - 1);
                if name.count == 0 return "", copy_string("empty label");

                if pass == 0 {
                    for labels if it.name == name return "", copy_string("duplicate label");
                    array_add(*labels, .{ name, address });
                }

                continue;
            }

            space := find_index_from_left(line, #char " ");
            if space < 0 return "", copy_string("expected mnemonic and operand");

            mnemonic := slice(line, 0, space);
            operand := trim(slice(line, space + 1, line.count - space - 1));
            size := ifx mnemonic == "byte" then 1 else 2;

            if mnemonic != "byte" && mnemonic != "op" && mnemonic != "jp" && mnemonic != "call" && mnemonic != "index" return "", copy_string("unknown mnemonic");
            if size == 2 && address % 2 != 0 return "", copy_string("unaligned instruction");

            if pass == 1 {
                value, ok := hex_number(operand);

                if mnemonic == "jp" || mnemonic == "call" || mnemonic == "index" {
                    for labels if it.name == operand {
                        value = it.address;
                        ok = true;
                        break;
                    }

                    if !ok || value > 4095 return "", copy_string("bad address or unknown label");

                    value |= ifx mnemonic == "jp" then 0x1000 else ifx mnemonic == "call" then 0x2000 else 0xA000;
                }

                if !ok || value > (ifx size == 1 then 255 else 65535) return "", copy_string("bad hexadecimal operand");
                if size == 2 array_add(*bytes, cast(u8) (value >> 8));

                array_add(*bytes, cast(u8) (value & 255));
            }

            address += size;
            if address > 4096 return "", copy_string("program exceeds memory");
        }
    }

    if bytes.count == 0 return "", copy_string("empty program");

    return copy_string(string.{ bytes.count, bytes.data }), "";
}

// Lossless raw-op syntax, including a trailing data byte; useful for round-trip tests.
disassemble :: (bytes: string) -> string {
    builder: String_Builder;
    digits :: "0123456789ABCDEF";
    at := 0;

    while at < bytes.count {
        pair := at + 1 < bytes.count;

        append(*builder, ifx pair then "op " else "byte ");
        count := ifx pair then 2 else 1;
        for 0..count - 1 {
            b := bytes[at + it];
            append(*builder, slice(digits, b >> 4, 1));
            append(*builder, slice(digits, b & 15, 1));
        }

        append(*builder, "\n");
        at += count;
    }

    return builder_to_string(*builder);
}
