#scope_module
#import "Basic";
#import "String";

gme_raw_tokens :: (value: Macro_Value) -> string {
    if !gm_tokens_valid(value.tokens) return "";
    if value.tokens.count == 0 return copy_string("");

    unit := gm_token_unit();
    if !unit return "";

    builder: String_Builder;

    defer free_buffers(*builder);

    for token, index: value.tokens {
        native := clang_getTokenSpelling(unit, token);
        if index > 0 append(*builder, " ");
        append(*builder, to_string(clang_getCString(native)));
        clang_disposeString(native);
    }

    return builder_to_string(*builder);
}

gme_expect_tokens :: (tokens: [] CXToken, start: int, expected: [] string) -> bool {
    if !gm_tokens_valid(tokens) return false;

    if start < 0 {
        gm_input_error("token matching requires a nonnegative start index");
        return false;
    }

    if start > tokens.count || expected.count > tokens.count - start return false;
    if expected.count == 0 return true;

    unit := gm_token_unit();
    if !unit return false;

    for text, index: expected {
        native := clang_getTokenSpelling(unit, tokens[start + index]);
        equal := to_string(clang_getCString(native)) == text;
        clang_disposeString(native);
        if !equal return false;
    }

    return true;
}

gme_final_child :: (cur: CXCursor) -> *Expression {
    if clang_Cursor_isNull(cur) != 0 return null;

    children := native_children(cur);

    defer array_free(children);
    if children.count == 0 return null;

    expression, skip := create_expression_from_cursor(children[children.count - 1], cur);
    if skip return null;

    return expression;
}

gme_call_expression :: (cur: CXCursor, expect_type_ref: bool) -> *Expression {
    if clang_Cursor_isNull(cur) != 0 return null;

    if expect_type_ref {
        if cur.kind == .GNUNullExpr || cur.kind == .CXXNullPtrLiteralExpr return cast(*Expression) gm_native_literal(cur, clang_getNullCursor());
        if cur.kind == .UnaryExpr return cast(*Expression) gm_native_literal(cur, clang_getNullCursor());
    }

    children := native_children(cur);

    defer array_free(children);
    if children.count == 0 return null;

    first := 0;
    if children[0].kind == .TypeRef first = 1;

    if cur.kind == .CXXFunctionalCastExpr && children.count - first == 1 {
        child := children[first];
        if child.kind == .InitListExpr return gm_build_expression(child, cur);

        native := clang_getCanonicalType(clang_getCursorType(cur));
        if expect_type_ref && native.kind != .Record && native.kind != .Unexposed {
            if child.kind == .IntegerLiteral || child.kind == .FloatingLiteral || child.kind == .CharacterLiteral || child.kind == .StringLiteral || child.kind == .CXXBoolLiteralExpr return cast(*Expression) gm_native_literal(cur, clang_getNullCursor());
        }
    }

    values: [..] CXCursor;

    defer array_free(values);

    for index: first..children.count - 1 {
        child := children[index];
        if expect_type_ref && children.count - first == 1 && (child.kind == .GNUNullExpr || child.kind == .CXXNullPtrLiteralExpr || child.kind == .UnaryExpr) return cast(*Expression) gm_native_literal(child, cur);

        if child.kind == .MemberRefExpr || child.kind == .DeclRefExpr || child.kind == .CallExpr || child.kind == .LambdaExpr || child.kind == .NamespaceRef || child.kind == .TemplateRef {
            gm_construction_error("unsupported native child in constructor expression");
            return null;
        }

        if child.kind == .UnaryOperator && clang_getCursorUnaryOperatorKind(child) == .AddrOf {
            gm_construction_error("constructor expression cannot represent native address-of");
            return null;
        }

        if clang_isExpression(child.kind) == 0 {
            gm_construction_error("constructor expression requires expression children after its type reference");
            return null;
        }

        array_add(*values, child);
    }

    return gm_aggregate_expression(cur, values);
}

GME_Token :: struct {
    text: string;
    kind: CXTokenKind;
    file: CXFile;
    start: u32;
    end: u32;
    adjacent: bool;
}

gme_token_copy :: (value: Macro_Value) -> [..] GME_Token, bool {
    result: [..] GME_Token;
    if !gm_tokens_valid(value.tokens) return result, false;
    if value.tokens.count == 0 return result, true;

    unit := gm_token_unit();
    if !unit return result, false;

    for token: value.tokens {
        native := clang_getTokenSpelling(unit, token);
        item: GME_Token;
        item.text = copy_string(to_string(clang_getCString(native)));
        clang_disposeString(native);
        item.kind = clang_getTokenKind(token);
        range := clang_getTokenExtent(unit, token);
        end_file: CXFile;

        clang_getSpellingLocation(clang_getRangeStart(range), *item.file, null, null, *item.start);
        clang_getSpellingLocation(clang_getRangeEnd(range), *end_file, null, null, *item.end);
        if item.file != end_file item.end = item.start;

        if result.count > 0 {
            previous := result[result.count - 1];
            item.adjacent = item.file != null && item.file == previous.file && item.start == previous.end;
        }

        array_add(*result, item);
    }

    return result, true;
}

gme_free_tokens :: (tokens: *[..] GME_Token) {
    for item: tokens.* free(item.text);
    array_free(tokens.*);
    tokens.* = .{};
}

gme_unsupported :: (name: string, reason: string) {
    options := gm_active_options();
    if options && options.log_unsupported log("Bindings_Generator: macro % is unsupported: %\n", name, reason);
}

gme_comment :: (builder: *String_Builder, text: string) {
    for byte, index: text {
        if byte == 13 {
            append(builder, "\n");
        } else if byte != 10 || index == 0 || text[index - 1] != 13 {
            append(builder, *byte, 1);
        }
    }
}

gme_enclosed :: (tokens: [] GME_Token) -> bool {
    if tokens.count < 2 || tokens[0].text != "(" || tokens[tokens.count - 1].text != ")" return false;

    depth := 0;

    for token, index: tokens {
        if token.kind == .Comment continue;
        if token.text == "(" depth += 1;
        if token.text == ")" depth -= 1;
        if depth < 0 || (depth == 0 && index < tokens.count - 1) return false;
    }

    return depth == 0;
}

gme_macro_text :: (value: Macro_Value, enumeration: *Enum, name: string, ignore_omit: bool) -> string, bool, string {
    tokens, valid := gme_token_copy(value);

    defer gme_free_tokens(*tokens);
    if !valid return "", false, "invalid or unavailable native token view";
    if tokens.count == 0 return "", false, "empty expression";

    builder: String_Builder;

    defer free_buffers(*builder);
    start := 0;
    limit := tokens.count;

    if gme_enclosed(tokens) {
        start = 1;
        limit -= 1;
    }

    index := start;
    while index < limit {
        token := tokens[index];
        if index > start && !token.adjacent append(*builder, " ");

        if token.kind == .Comment {
            gme_comment(*builder, token.text);
        } else if token.kind == .Literal {
            converted, success, _was_string := convert_literal_value(token.text);
            if !success return "", false, "unsupported literal spelling";
            append(*builder, converted);
        } else if token.text == "(" {
            fragment: String_Builder;

            defer free_buffers(*fragment);
            converted, end := maybe_append_cast(*fragment, value.tokens, index, enumeration);
            if converted {
                if end < index || end >= limit return "", false, "cast helper returned an invalid token extent";
                if end + 1 < limit && tokens[end + 1].text == "&" return "", false, "prefix address-of after a cast";

                text := builder_to_string(*fragment);

                append(*builder, text);
                free(text);
                index = end + 1;
                continue;
            }

            append(*builder, token.text);
        } else if token.kind == .Identifier || token.kind == .Keyword {
            replacement := false;

            for pair: TOKENS_TO_REPLACE if pair[0] == token.text {
                append(*builder, pair[1]);
                replacement = true;
                break;
            }

            if !replacement {
                if token.text == name && name.count > 0 return "", false, "self identifier";

                if token.text == "true" || token.text == "false" {
                    append(*builder, token.text);
                } else {
                    found, output, _type, parent := find_and_check_macro_identifier(token.text, enumeration, ignore_omit);
                    if !found return "", false, "unresolved identifier";

                    if parent && parent != enumeration {
                        append(*builder, gm_name(cast(*Declaration) parent));
                        append(*builder, ".");
                    }

                    append(*builder, output);
                }
            }
        } else {
            append(*builder, token.text);
        }

        index += 1;
    }

    result := builder_to_string(*builder);
    if result == name && name.count > 0 {
        free(result);
        return "", false, "converted value equals its macro name";
    }

    return result, true, "";
}

gme_continuation :: (text: string) -> bool {
    for byte, index: text if byte == 92 && index + 1 < text.count {
        if text[index + 1] == 10 || text[index + 1] == 13 return true;
    }

    return false;
}

gme_macro_form :: (value: Macro_Value, name: string) -> bool, string {
    tokens, valid := gme_token_copy(value);

    defer gme_free_tokens(*tokens);
    if !valid return false, "invalid or unavailable native token view";
    if tokens.count == 0 return false, "empty expression";

    unit := gm_token_unit();
    if !unit return false, "missing translation unit";

    previous := "";
    previous_word := false;
    previous_string := false;
    previous_operand := false;
    have_expression := false;

    for token, index: tokens {
        if gme_continuation(token.text) return false, "continuation syntax";

        if index > 0 {
            prior := tokens[index - 1];
            if prior.file && prior.file == token.file && token.start > prior.end {
                size: u64;
                bytes := clang_getFileContents(unit, token.file, *size);

                if !bytes || token.start > size return false, "unavailable token source extent";

                gap := string.{ cast(s64) (token.start - prior.end), bytes + prior.end };
                if gme_continuation(gap) return false, "continuation syntax";
            }
        }

        if token.kind == .Comment continue;

        have_expression = true;
        if token.text == "{" || token.text == "}" || token.text == "," || token.text == "\\" return false, "unsupported macro punctuation";
        if token.text == name && name.count > 0 return false, "self identifier";
        if token.text == "." || token.text == "->" return false, "identifier member syntax";
        if token.text == ")" && previous == "(" return false, "empty call syntax";

        if token.text == "&" {
            if !previous_operand return false, "prefix address-of";
        }

        is_string := false;

        if token.kind == .Literal {
            _converted, success, string_value := convert_literal_value(token.text);
            if !success return false, "unsupported literal conversion";
            is_string = string_value;
        }

        word := token.kind == .Identifier || token.kind == .Keyword;
        if (word || is_string) && (previous_word || previous_string) return false, "adjacent identifier/string concatenation";

        previous = token.text;
        previous_word = word;
        previous_string = is_string;
        previous_operand = word || token.kind == .Literal || token.text == ")" || token.text == "]";
    }

    if !have_expression return false, "empty expression";

    return true, "";
}

gme_check_macro :: (declaration: *Declaration) {
    if !declaration || declaration.kind != .MACRO_DEFINITION || !declaration.expression || declaration.expression.kind != .LITERAL {
        gm_input_error("macro validation requires a macro literal declaration");
        return;
    }

    literal := cast(*Literal) declaration.expression;
    if literal.literal_kind != .MACRO {
        gm_input_error("macro validation requires replacement-token storage");
        return;
    }

    if declaration.macro_value_result != .NEEDS_CHECKING return;

    if !gm_tokens_valid(literal.macro_value.tokens) return;

    evaluated, native_type, available, supported := gm_evaluate_macro_declaration(declaration);
    // An absent exact parse receipt or native environment is not a conversion
    // decision. The canonical evaluator diagnoses that unavailable dependency.
    if !available return;

    if !supported || !evaluated || !native_type {
        gme_unsupported(declaration.name, "native expression cannot be represented");
        declaration.macro_value_result = .DISCARD;
        return;
    }

    form, reason := gme_macro_form(literal.macro_value, declaration.name);
    if !form {
        gme_unsupported(declaration.name, reason);
        declaration.macro_value_result = .DISCARD;
        return;
    }

    if evaluated.literal_kind != .INTEGER && evaluated.literal_kind != .UNSIGNED_INTEGER && evaluated.literal_kind != .FLOAT && evaluated.literal_kind != .STRING && evaluated.literal_kind != .NULL {
        gme_unsupported(declaration.name, "unsupported evaluated literal category");
        declaration.macro_value_result = .DISCARD;
        return;
    }

    if evaluated.literal_kind == .FLOAT {
        bits := (cast(*u64) *evaluated.float_value).*;
        if (bits & 0x7ff0000000000000) == 0x7ff0000000000000 {
            gme_unsupported(declaration.name, "nonfinite numeric literal");
            declaration.macro_value_result = .DISCARD;
            return;
        }
    }

    text, converted, conversion_reason := gme_macro_text(literal.macro_value, null, declaration.name, false);

    defer free(text);

    if !converted {
        gme_unsupported(declaration.name, conversion_reason);
        declaration.macro_value_result = .DISCARD;
        return;
    }

    // Keep the public token expression and any caller-supplied type intact.
    // Validation's established public mutation is the conversion decision.
    declaration.macro_value_result = .KEEP;
}
