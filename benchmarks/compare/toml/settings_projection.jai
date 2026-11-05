// Full public formatter schema projection for the independent oracle and actual benchmark.
// Example: value := project_settings(config); defer Json.free_json(*value);
Json :: #import "Extensions/Json";

project_settings :: (config: $T) -> Json.Json_Value {
    resolved := Json.json_object();

    Json.json_set(*resolved, "indent_width", Json.json_string(tprint("%", formatInt(config.indent_width))));
    Json.json_set(*resolved, "case_indent", Json.json_string(tprint("%", formatInt(ifx config.case_indent < 0 then config.indent_width else config.case_indent))));
    Json.json_set(*resolved, "case_body_indent", Json.json_string(tprint("%", formatInt(ifx config.case_body_indent < 0 then config.indent_width else config.case_body_indent))));
    Json.json_set(*resolved, "max_blank_lines", Json.json_string(tprint("%", formatInt(config.max_blank_lines))));
    Json.json_set(*resolved, "max_width", Json.json_string(tprint("%", formatInt(config.max_width))));
    Json.json_set(*resolved, "brace_style", Json.json_string(ifx config.brace_style == .PRESERVE then "\"preserve\"" else "\"same_line\""));
    Json.json_set(*resolved, "blank_lines_between_items", Json.json_string(ifx config.blank_lines_between_items then "true" else "false"));
    Json.json_set(*resolved, "blank_line_after_declarations", Json.json_string(ifx config.blank_line_after_declarations then "true" else "false"));
    Json.json_set(*resolved, "blank_lines_around_blocks", Json.json_string(ifx config.blank_lines_around_blocks then "true" else "false"));
    Json.json_set(*resolved, "blank_line_between_cases", Json.json_string(ifx config.blank_line_between_cases then "true" else "false"));
    Json.json_set(*resolved, "blank_line_before_comments", Json.json_string(ifx config.blank_line_before_comments then "true" else "false"));
    Json.json_set(*resolved, "blank_line_after_guards", Json.json_string(ifx config.blank_line_after_guards then "true" else "false"));
    Json.json_set(*resolved, "blank_line_before_return", Json.json_string(ifx config.blank_line_before_return then "true" else "false"));
    Json.json_set(*resolved, "format_doc_examples", Json.json_string(ifx config.format_doc_examples then "true" else "false"));
    overrides := Json.json_object();
    for config.doc_format Json.json_set(*overrides, it.key, Json.json_string(it.value));

    ignore := Json.json_array();
    for config.ignore Json.json_push(*ignore, Json.json_string(it));

    unknown := Json.json_array();
    for config.unknown_keys Json.json_push(*unknown, Json.json_string(it));

    result := Json.json_object();

    Json.json_set(*result, "resolved", resolved);
    Json.json_set(*result, "overrides", overrides);
    Json.json_set(*result, "ignore", ignore);
    Json.json_set(*result, "unknown", unknown);

    return result;
}
