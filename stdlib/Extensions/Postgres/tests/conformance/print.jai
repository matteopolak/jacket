// Reproduces psql's result layout: aligned, wrapped, unaligned, CSV, HTML, AsciiDoc, LaTeX and
// troff -ms tables, plain and expanded.

Print_Format :: enum u8 {
    ALIGNED;
    UNALIGNED;
    WRAPPED;
    CSV;
    HTML;
    ASCIIDOC;
    LATEX;
    LATEX_LONGTABLE;
    TROFF_MS;
}

Expanded :: enum u8 {
    OFF;
    ON;
    AUTO;
}

Line_Style :: enum u8 {
    ASCII;
    OLD_ASCII;
    UNICODE;
}

Print_Options :: struct {
    format := Print_Format.ALIGNED;
    expanded := Expanded.OFF;
    border: s32 = 1; // 0, 1 or 2; larger values draw like 2
    tuples_only: bool; // \t: no title, header or footer
    footer := true; // \pset footer: the "(N rows)" line only; explicit footers always print
    null_display: string;
    true_display := "t"; // \pset display_true: how a boolean column shows true
    false_display := "f"; // \pset display_false
    field_separator := "|"; // unaligned
    record_separator := "\n"; // unaligned
    field_separator_zero: bool; // unaligned: a zero byte instead of `field_separator`
    record_separator_zero: bool; // unaligned: a zero byte instead of `record_separator`
    csv_field_separator := ","; // its first byte is used
    // The width wrapped output is fitted to, and the width `Expanded.AUTO` compares with. 0 means
    // unknown, which is what psql sees when its output is not a terminal: no wrapping, and AUTO
    // stays in the regular layout.
    columns: s32;
    linestyle := Line_Style.ASCII;
    title: string;
    // html: extra attributes of the <table> tag; latex-longtable: the widths of the left-aligned
    // columns, separated by whitespace
    table_attr: string;
}

Table :: struct {
    title: string;
    headers: [] string;
    align: [] u8; // per column: #char "l" or #char "r"; missing entries are left-aligned
    cells: [] string; // row-major, rows * headers.count
    nulls: [] bool; // same shape as `cells`, or empty when nothing is NULL
    footers: [] string;
    // Rows in the table. Only read when there are no columns, since then `cells` cannot tell.
    rows: s64;
    // Print "(N rows)" when `footers` is empty and the options allow it, the way a query result does.
    default_footer: bool;

    // A result printed in pieces as its rows arrive: only the first piece has the title and
    // headers, only the last the footers, and `prior_records` counts the rows printed before this
    // piece (for record numbers and the row count). The aligned layouts honour these.
    start_table := true;
    stop_table := true;
    prior_records: s64;
}

// Appends the table as psql would print it, ending with the blank line psql leaves after a result
// (the CSV and unaligned layouts end with their last record instead).
print_table :: (out: *String_Builder, table: Table, options: Print_Options) {
    auto_release_temp();

    columns := table.headers.count;
    rows := ifx columns > 0 then table.cells.count / columns else table.rows;

    p: Layout;
    p.options = options;
    p.style = line_style(options.linestyle);
    p.columns = columns;
    p.rows = rows;
    p.title = ifx table.title.count > 0 then table.title else options.title;
    p.border = clamp_border(options.border);
    p.raw_border = max(options.border, 0);
    p.align = table.align;
    p.start_table = table.start_table;
    p.stop_table = table.stop_table;
    p.prior_records = table.prior_records;

    p.headers = NewArray(columns, string,, temp);
    for 0..columns - 1 p.headers[it] = clean_utf8(table.headers[it]);

    p.cells = NewArray(table.cells.count, string,, temp);

    for 0..table.cells.count - 1 {
        isnull := table.nulls.count > it && table.nulls[it];
        p.cells[it] = clean_utf8(ifx isnull then options.null_display else table.cells[it]);
    }

    p.footers = table.footers;

    if table.footers.count == 0 && table.default_footer && options.footer {
        total := table.prior_records + rows;
        p.default_footer = tprint(ifx total == 1 then "(% row)" else "(% rows)", total);
        p.footers = string.[p.default_footer];
        p.footers_are_default = true;
    }

    if options.format == {
        case .CSV;
            if options.expanded == .ON print_csv_vertical(out, *p);
            else print_csv_text(out, *p);

        case .UNALIGNED;
            if options.expanded == .ON print_unaligned_vertical(out, *p);
            else print_unaligned_text(out, *p);

        case .HTML;
            if options.expanded == .ON print_html_vertical(out, *p);
            else print_html_text(out, *p);

        case .ASCIIDOC;
            if options.expanded == .ON print_asciidoc_vertical(out, *p);
            else print_asciidoc_text(out, *p);

        case .LATEX;
            if options.expanded == .ON print_latex_vertical(out, *p);
            else print_latex_text(out, *p);

        case .LATEX_LONGTABLE;
            if options.expanded == .ON print_latex_vertical(out, *p);
            else print_latex_longtable_text(out, *p);

        case .TROFF_MS;
            if options.expanded == .ON print_troff_ms_vertical(out, *p);
            else print_troff_ms_text(out, *p);

        case;
            if options.expanded == .ON print_aligned_vertical(out, *p);
            else print_aligned_text(out, *p);
    }
}

// What psql does for a query result: numbers are right-aligned, and the table gets a
// "(N rows)" line. A result with no columns still has rows, so pass `row_count` for it.
print_query_result :: (out: *String_Builder, headers: [] string, type_oids: [] u32, cells: [] string, nulls: [] bool, options: Print_Options, row_count := -1) {
    auto_release_temp();

    table: Table;
    table.headers = headers;
    table.nulls = nulls;
    table.default_footer = true;

    table.align = NewArray(headers.count, u8,, temp);

    for 0..headers.count - 1 {
        table.align[it] = ifx it < type_oids.count then column_alignment(type_oids[it]) else #char "l";
    }

    table.cells = NewArray(cells.count, string,, temp);

    for 0..cells.count - 1 {
        column := ifx headers.count > 0 then it % headers.count else 0;
        isnull := nulls.count > it && nulls[it];
        table.cells[it] = ifx column < type_oids.count && !isnull then display_bool(options, type_oids[column], cells[it]) else cells[it];
    }

    if row_count >= 0 table.rows = row_count;
    else if headers.count > 0 table.rows = cells.count / headers.count;

    print_table(out, table, options);
}

// `r` for the types psql right-aligns (integers, floats, numeric, money and the id types), else `l`.
column_alignment :: (type_oid: u32) -> u8 {
    if type_oid == {
        case 20; #through; // int8
        case 21; #through; // int2
        case 23; #through; // int4
        case 26; #through; // oid
        case 28; #through; // xid
        case 29; #through; // cid
        case 700; #through; // float4
        case 701; #through; // float8
        case 790; #through; // money
        case 1700; #through; // numeric
        case 5069; #through; // xid8
        case 6437; // oid8
            return #char "r";
    }

    return #char "l";
}

// What `\pset display_true` and `display_false` make of a boolean column's text; the text of any
// other type is returned as it is.
display_bool :: (options: Print_Options, type_oid: u32, text: string) -> string {
    if type_oid != BOOL_OID || text.count == 0 return text;

    if text[0] == #char "t" return options.true_display;
    if text[0] == #char "f" return options.false_display;

    return text;
}

BOOL_OID :: 16;

// How many terminal columns the text takes, as psql counts them: wide East Asian characters are
// two, combining marks none, and each control character the width of the escape it prints as.
// Newlines are not handled; this is the width of one line.
display_width :: (text: string) -> s64 {
    auto_release_temp();

    width := 0;
    for format_text(clean_utf8(text)) width = max(width, it.width);
    return width;
}

#scope_file

// Everything the layouts share, after NULLs and invalid UTF-8 have been dealt with.
Layout :: struct {
    options: Print_Options;
    style: Style;
    columns: s64;
    rows: s64;
    border: s64;
    raw_border: s64; // `border` before it was limited to 2, for the formats that tell 3 apart
    title: string;
    headers: [] string;
    cells: [] string;
    align: [] u8;
    footers: [] string;
    footers_are_default: bool;
    default_footer: string;
    start_table: bool;
    stop_table: bool;
    prior_records: s64;
}

Rule :: struct {
    horizontal: string;
    left: string;
    middle: string;
    right: string;
}

Style :: struct {
    top: Rule;
    separator: Rule; // between the header and the rows
    bottom: Rule;
    data: Rule; // `horizontal` is unused
    middle_newline: string; // column divider where the cell to its right continues after a newline
    middle_wrap: string;
    middle_blank: string; // ... where the cell to its right has no more lines
    header_newline_left: string;
    header_newline_right: string;
    newline_left: string;
    newline_right: string;
    wrap_left: string;
    wrap_right: string;
    wrap_right_border: bool;
    old: bool;
}

line_style :: (style: Line_Style) -> Style {
    s: Style;

    if style == .UNICODE {
        s.top = .{ "─", "┌", "┬", "┐" };
        s.separator = .{ "─", "├", "┼", "┤" };
        s.bottom = .{ "─", "└", "┴", "┘" };
        s.data = .{ "", "│", "│", "│" };
        s.middle_newline = "│";
        s.middle_wrap = "│";
        s.middle_blank = "│";
        s.header_newline_left = " ";
        s.header_newline_right = "↵";
        s.newline_left = " ";
        s.newline_right = "↵";
        s.wrap_left = "…";
        s.wrap_right = "…";
        s.wrap_right_border = true;

        return s;
    }

    s.top = .{ "-", "+", "+", "+" };
    s.separator = s.top;
    s.bottom = s.top;
    s.data = .{ "", "|", "|", "|" };

    if style == .OLD_ASCII {
        s.middle_newline = ":";
        s.middle_wrap = ";";
        s.middle_blank = " ";
        s.header_newline_left = "+";
        s.header_newline_right = " ";
        s.newline_left = " ";
        s.newline_right = " ";
        s.wrap_left = " ";
        s.wrap_right = " ";
        s.old = true;

        return s;
    }

    s.middle_newline = "|";
    s.middle_wrap = "|";
    s.middle_blank = "|";
    s.header_newline_left = " ";
    s.header_newline_right = "+";
    s.newline_left = " ";
    s.newline_right = "+";
    s.wrap_left = ".";
    s.wrap_right = ".";
    s.wrap_right_border = true;

    return s;
}

clamp_border :: (border: s32) -> s64 {
    if border < 0 return 0;
    if border > 2 return 2;
    return border;
}

// ---- Text and widths -------------------------------------------------------------------------

// One line of a cell as it is drawn, with tabs and control characters already replaced.
Text_Line :: struct {
    text: string;
    width: s64;
}

// The width of a code point in columns, or -1 for a control character.
code_width :: (cp: u32) -> s64 {
    if cp == 0 return 0;
    if cp < 0x20 || (cp >= 0x7f && cp < 0xa0) || cp > 0x10ffff return -1;
    if cp < 0x300 return 1;
    if in_ranges(NONSPACING, cp) return 0;
    if in_ranges(WIDE, cp) return 2;

    return 1;
}

// `ranges` holds inclusive (first, last) pairs in ascending order.
in_ranges :: (ranges: [] u32, cp: u32) -> bool {
    low := 0;
    high := ranges.count / 2 - 1;

    while low <= high {
        mid := (low + high) / 2;
        if cp > ranges[mid * 2 + 1] low = mid + 1;
        else if cp < ranges[mid * 2] high = mid - 1;
        else return true;
    }

    return false;
}

// The length of the UTF-8 sequence at `i`, 0 when it is not a valid one, and its code point.
read_char :: (s: string, i: s64) -> length: s64, cp: u32 {
    c := s[i];
    if c < 0x80 return 1, c;

    length := 0;
    cp: u32;

    if (c & 0xe0) == 0xc0 {
        length = 2;
        cp = c & 0x1f;
    } else if (c & 0xf0) == 0xe0 {
        length = 3;
        cp = c & 0x0f;
    } else if (c & 0xf8) == 0xf0 {
        length = 4;
        cp = c & 0x07;
    } else return 0, 0;

    if i + length > s.count return 0, 0;

    for 1..length - 1 {
        b := s[i + it];
        if (b & 0xc0) != 0x80 return 0, 0;
        cp = (cp << 6) | (b & 0x3f);
    }

    if length == 2 && cp < 0x80 return 0, 0;
    if length == 3 && (cp < 0x800 || (cp >= 0xd800 && cp <= 0xdfff)) return 0, 0;
    if length == 4 && (cp < 0x10000 || cp > 0x10ffff) return 0, 0;

    return length, cp;
}

// The text with any bytes that are not valid UTF-8 dropped, as psql does before printing.
clean_utf8 :: (text: string) -> string {
    bad := false;
    i := 0;

    while i < text.count {
        if text[i] < 0x80 {
            i += 1;
            continue;
        }

        length, _ := read_char(text, i);
        if length == 0 {
            bad = true;
            break;
        }

        i += length;
    }

    if !bad return text;

    kept := NewArray(text.count, u8,, temp);
    n := 0;

    i = 0;
    while i < text.count {
        length := 1;
        if text[i] >= 0x80 length, _ = read_char(text, i);

        if length == 0 {
            i += 1;
            continue;
        }

        for 0..length - 1 kept[n + it] = text[i + it];

        n += length;
        i += length;
    }

    return .{ n, kept.data };
}

HEX_DIGITS :: "0123456789ABCDEF";

// Splits a cell into the lines psql draws. A tab pads to the next multiple of 8 columns, a
// carriage return shows as `\r`, and other control characters as `\xNN` or `\uNNNN`.
format_text :: (text: string) -> [] Text_Line {
    plain := true;

    for text {
        if it < 0x20 || it >= 0x7f {
            plain = false;
            break;
        }
    }

    if plain {
        lines := NewArray(1, Text_Line,, temp);
        lines[0] = .{ text, text.count };
        return lines;
    }

    buffer := NewArray(text.count * 8 + 1, u8,, temp);
    lines: [..] Text_Line;
    lines.allocator = temp;

    n := 0;
    start := 0;
    width := 0;
    i := 0;

    while i < text.count {
        length, cp := read_char(text, i);
        c := text[i];

        i += length;

        if length == 1 {
            if c == #char "\n" {
                array_add(*lines, .{ .{ n - start, buffer.data + start }, width });
                start = n;
                width = 0;
            } else if c == #char "\r" {
                buffer[n] = #char "\\";
                buffer[n + 1] = #char "r";
                n += 2;
                width += 2;
            } else if c == #char "\t" {
                while true {
                    buffer[n] = #char " ";
                    n += 1;
                    width += 1;
                    if width % 8 == 0 break;
                }
            } else if c < 0x20 || c == 0x7f {
                buffer[n] = #char "\\";
                buffer[n + 1] = #char "x";
                buffer[n + 2] = HEX_DIGITS[c >> 4];
                buffer[n + 3] = HEX_DIGITS[c & 15];
                n += 4;
                width += 4;
            } else {
                buffer[n] = c;
                n += 1;
                width += 1;
            }
        } else if code_width(cp) < 0 {
            buffer[n] = #char "\\";
            buffer[n + 1] = #char "u";
            for 0..3 buffer[n + 2 + it] = HEX_DIGITS[(cp >> cast(u32) ((3 - it) * 4)) & 15];

            n += 6;
            width += 6;
        } else {
            for 0..length - 1 buffer[n + it] = text[i - length + it];
            n += length;
            width += code_width(cp);
        }
    }

    array_add(*lines, .{ .{ n - start, buffer.data + start }, width });

    return lines;
}

// Splits a formatted line into the longest prefix that fits in `target` columns: its length in
// bytes and its width. A first character wider than `target` is taken anyway.
fit_width :: (line: string, target: s64) -> bytes: s64, width: s64 {
    width := 0;
    i := 0;

    while i < line.count {
        length, cp := read_char(line, i);
        if length == 0 length = 1;

        w := code_width(cp);
        if w < 0 w = 1;
        if target < width + w && width != 0 break;

        width += w;
        i += length;
    }

    return i, width;
}

// ---- Output helpers --------------------------------------------------------------------------

SPACES :: "                                ";

pad :: (out: *String_Builder, count: s64) {
    left := count;
    while left > 0 {
        n := min(left, SPACES.count);
        append(out, SPACES.data, n);
        left -= n;
    }
}

repeat :: (out: *String_Builder, text: string, count: s64) {
    if count <= 0 return;
    for 1..count append(out, text);
}

print_rule :: (out: *String_Builder, p: *Layout, widths: [] s64, rule: Rule) {
    border := p.border;

    if border == 1 append(out, rule.horizontal);
    else if border == 2 {
        append(out, rule.left);
        append(out, rule.horizontal);
    }

    for i: 0..p.columns - 1 {
        repeat(out, rule.horizontal, widths[i]);

        if i < p.columns - 1 {
            if border == 0 append(out, " ");
            else {
                append(out, rule.horizontal);
                append(out, rule.middle);
                append(out, rule.horizontal);
            }
        }
    }

    if border == 2 {
        append(out, rule.horizontal);
        append(out, rule.right);
    } else if border == 1 append(out, rule.horizontal);

    append(out, "\n");
}

column_aligned_right :: (p: *Layout, column: s64) -> bool {
    return column < p.align.count && p.align[column] == #char "r";
}

// ---- Aligned and wrapped ---------------------------------------------------------------------

Wrap :: enum u8 {
    NONE;
    WRAP;
    NEWLINE;
}

// Longest line of a formatted cell.
text_width :: (lines: [] Text_Line) -> s64 {
    width := 0;
    for lines width = max(width, it.width);
    return width;
}

// What the aligned text layout needs to know about every column before drawing.
Text_Metrics :: struct {
    header_lines: [][] Text_Line;
    cell_lines: [][] Text_Line;
    header_width: [] s64;
    max_width: [] s64;
    wrap_width: [] s64;
    average: [] s64;
    width_total: s64;
    total_header_width: s64;
}

// Where each cell of the row being drawn stands.
Cell_Cursor :: struct {
    line_index: [] s64; // the line of each cell being drawn
    consumed: [] s64; // bytes of that line already drawn
    wrap: [] Wrap;
}

border_width_total :: (border: s64, columns: s64) -> s64 {
    if border == 0 return columns;
    if border == 1 return columns * 3 - ifx columns > 0 then 1 else 0;
    return columns * 3 + 1;
}

measure_text_table :: (p: *Layout) -> Text_Metrics {
    m: Text_Metrics;
    columns := p.columns;

    m.header_lines = NewArray(columns, [] Text_Line,, temp);
    for 0..columns - 1 m.header_lines[it] = format_text(p.headers[it]);

    m.cell_lines = NewArray(p.cells.count, [] Text_Line,, temp);
    for 0..p.cells.count - 1 m.cell_lines[it] = format_text(p.cells[it]);

    m.header_width = NewArray(columns, s64,, temp);
    m.max_width = NewArray(columns, s64,, temp);
    m.wrap_width = NewArray(columns, s64,, temp);
    m.average = NewArray(columns, s64,, temp);

    for 0..columns - 1 {
        m.header_width[it] = text_width(m.header_lines[it]);
        m.max_width[it] = m.header_width[it];
    }

    for 0..p.cells.count - 1 {
        column := it % columns;
        w := text_width(m.cell_lines[it]);
        m.max_width[column] = max(m.max_width[column], w);
        m.average[column] += w;
    }

    if p.rows > 0 for 0..columns - 1 m.average[it] /= p.rows;

    m.width_total = border_width_total(p.border, columns);
    m.total_header_width = m.width_total;

    for 0..columns - 1 {
        m.width_total += m.max_width[it];
        m.total_header_width += m.header_width[it];
        m.wrap_width[it] = m.max_width[it];
    }

    return m;
}

// The column whose widest value is furthest above its average, or -1 when none can shrink.
column_to_shrink :: (m: *Text_Metrics) -> s64 {
    worst := -1;
    worst_ratio: float64;

    for 0..m.wrap_width.count - 1 {
        if m.average[it] == 0 || m.wrap_width[it] <= m.header_width[it] continue;

        ratio := cast(float64) m.wrap_width[it] / cast(float64) m.average[it] + cast(float64) m.max_width[it] * 0.01;
        if ratio > worst_ratio {
            worst_ratio = ratio;
            worst = it;
        }
    }

    return worst;
}

// Shrink the columns whose widest value is far above their average first: they wrap the
// fewest values per column removed.
shrink_to_target :: (m: *Text_Metrics, target: s64) {
    while m.width_total > target {
        worst := column_to_shrink(m);
        if worst < 0 break;

        m.wrap_width[worst] -= 1;
        m.width_total -= 1;
    }
}

print_text_title :: (out: *String_Builder, p: *Layout, width_total: s64) {
    title_width := 0;
    for split_lines(p.title) title_width = max(title_width, display_width(it));

    if title_width < width_total pad(out, (width_total - title_width) / 2);

    append(out, p.title);
    append(out, "\n");
}

// Draws line `current` of the header of column `i`. True when that column has no more lines.
print_header_cell :: (out: *String_Builder, p: *Layout, m: *Text_Metrics, done: [] bool, i: s64, current: s64) -> bool {
    style := *p.style;
    border := p.border;
    lines := m.header_lines[i];
    finished := false;

    if border != 0 || (!style.wrap_right_border && i > 0) {
        append(out, ifx current > 0 then style.header_newline_left else " ");
    }

    if !done[i] {
        line := lines[current];
        space := m.wrap_width[i] - line.width;

        pad(out, space / 2);
        append(out, line.text);
        pad(out, (space + 1) / 2);

        if current + 1 >= lines.count {
            done[i] = true;
            finished = true;
        }
    } else {
        pad(out, m.wrap_width[i]);
    }

    if border != 0 || style.wrap_right_border {
        append(out, ifx !done[i] then style.header_newline_right else " ");
    }

    if border != 0 && i < p.columns - 1 append(out, style.data.middle);

    return finished;
}

print_text_header :: (out: *String_Builder, p: *Layout, m: *Text_Metrics) {
    style := *p.style;
    border := p.border;

    if border == 2 print_rule(out, p, m.wrap_width, style.top);

    done := NewArray(p.columns, bool,, temp);
    remaining := p.columns;
    current := 0;

    while remaining > 0 {
        if border == 2 append(out, style.data.left);

        for i: 0..p.columns - 1 {
            if print_header_cell(out, p, m, done, i, current) remaining -= 1;
        }

        current += 1;
        if border == 2 append(out, style.data.right);

        append(out, "\n");
    }

    print_rule(out, p, m.wrap_width, style.separator);
}

// What a cell shows at its edges once the line it is on is drawn.
next_wrap :: (c: *Cell_Cursor, line_count: s64, j: s64) -> Wrap {
    if c.line_index[j] >= line_count return .NONE;
    if c.consumed[j] != 0 return .WRAP;
    if c.line_index[j] != 0 return .NEWLINE;
    return .NONE;
}

// Draws the part of the current line of cell `j` that fits. Returns the width drawn and whether
// the cell has more to draw.
draw_cell_chunk :: (out: *String_Builder, lines: [] Text_Line, c: *Cell_Cursor, j: s64, width: s64, right: bool) -> drawn: s64, more: bool {
    more := false;
    line := lines[c.line_index[j]];
    rest := slice(line.text, c.consumed[j], line.text.count);
    bytes, drawn := fit_width(rest, width);

    if drawn > width drawn = width;

    if right pad(out, width - drawn);

    append(out, slice(rest, 0, bytes));

    c.consumed[j] += bytes;
    if c.consumed[j] < line.text.count {
        more = true;
    } else {
        c.line_index[j] += 1;
        if c.line_index[j] < lines.count more = true;
        c.consumed[j] = 0;
    }

    return drawn, more;
}

append_cell_left :: (out: *String_Builder, style: *Style, wrap: Wrap) {
    if wrap == .WRAP append(out, style.wrap_left);
    else if wrap == .NEWLINE append(out, style.newline_left);
    else append(out, " ");
}

append_cell_right :: (out: *String_Builder, style: *Style, wrap: Wrap, last_edge: bool) {
    if wrap == .WRAP append(out, style.wrap_right);
    else if wrap == .NEWLINE append(out, style.newline_right);
    else if last_edge append(out, " ");
}

// Draws one line of cell `j` of `row`. True when the cell has more lines to draw.
print_text_cell :: (out: *String_Builder, p: *Layout, m: *Text_Metrics, c: *Cell_Cursor, row: s64, j: s64) -> bool {
    style := *p.style;
    border := p.border;
    lines := m.cell_lines[row * p.columns + j];
    width := m.wrap_width[j];
    drawn := width;
    final_spaces := border == 2 || j < p.columns - 1;
    right := column_aligned_right(p, j);
    more := false;

    if border != 0 append_cell_left(out, style, c.wrap[j]);

    if c.line_index[j] >= lines.count {
        if final_spaces pad(out, width);
    } else {
        drawn, more = draw_cell_chunk(out, lines, c, j, width, right);
    }

    c.wrap[j] = next_wrap(c, lines.count, j);

    if !right && (final_spaces || c.wrap[j] != .NONE) pad(out, width - drawn);

    append_cell_right(out, style, c.wrap[j], final_spaces);

    return more;
}

// The divider between cell `j` and the one to its right, which depends on how that one continues.
append_cell_divider :: (out: *String_Builder, p: *Layout, m: *Text_Metrics, c: *Cell_Cursor, row: s64, j: s64) {
    style := *p.style;
    next_exhausted := c.line_index[j + 1] >= m.cell_lines[row * p.columns + j + 1].count;

    if c.wrap[j + 1] == .WRAP append(out, style.middle_wrap);
    else if c.wrap[j + 1] == .NEWLINE append(out, style.middle_newline);
    else if next_exhausted append(out, style.middle_blank);
    else append(out, style.data.middle);
}

// Draws one physical line of `row`. True when some cell has more lines.
print_text_row_line :: (out: *String_Builder, p: *Layout, m: *Text_Metrics, c: *Cell_Cursor, row: s64) -> bool {
    style := *p.style;
    border := p.border;
    more := false;

    if border == 2 append(out, style.data.left);

    for j: 0..p.columns - 1 {
        if print_text_cell(out, p, m, c, row, j) more = true;
        if border != 0 && j < p.columns - 1 append_cell_divider(out, p, m, c, row, j);
    }

    if border == 2 append(out, style.data.right);

    append(out, "\n");

    return more;
}

print_text_rows :: (out: *String_Builder, p: *Layout, m: *Text_Metrics) {
    columns := p.columns;
    c: Cell_Cursor;
    c.line_index = NewArray(columns, s64,, temp);
    c.consumed = NewArray(columns, s64,, temp);
    c.wrap = NewArray(columns, Wrap,, temp);

    for row: 0..(ifx columns > 0 then p.rows - 1 else -1) {
        for 0..columns - 1 {
            c.line_index[it] = 0;
            c.consumed[it] = 0;
            c.wrap[it] = .NONE;
        }

        more := true;
        while more more = print_text_row_line(out, p, m, *c, row);
    }
}

print_footer_lines :: (out: *String_Builder, footers: [] string) {
    for footers {
        append(out, it);
        append(out, "\n");
    }
}

print_aligned_text :: (out: *String_Builder, p: *Layout) {
    options := *p.options;
    columns := p.columns;
    tuples_only := options.tuples_only;
    m := measure_text_table(p);
    target := cast(s64) options.columns;

    if options.format == .WRAPPED && target > 0 && target >= m.total_header_width {
        shrink_to_target(*m, target);
    }

    if options.expanded == .AUTO && target > 0 && columns > 1 && (target < m.total_header_width || target < m.width_total) {
        print_aligned_vertical(out, p);
        return;
    }

    if p.start_table && p.title.count > 0 && !tuples_only print_text_title(out, p, m.width_total);
    if p.start_table && !tuples_only print_text_header(out, p, *m);

    print_text_rows(out, p, *m);

    if !p.stop_table return;

    if p.border == 2 print_rule(out, p, m.wrap_width, p.style.bottom);
    if !tuples_only print_footer_lines(out, p.footers);

    append(out, "\n");
}

// The lines of `text` as separated by newlines.
split_lines :: (text: string) -> [] string {
    lines: [..] string;
    lines.allocator = temp;

    start := 0;

    for *text {
        if it.* == #char "\n" {
            array_add(*lines, slice(text, start, it_index - start));
            start = it_index + 1;
        }
    }

    array_add(*lines, slice(text, start, text.count - start));

    return lines;
}

// ---- Expanded --------------------------------------------------------------------------------

// A rule between records: `[ RECORD n ]` sits in it, and it spans the name and value columns.
print_record_rule :: (out: *String_Builder, p: *Layout, record: s64, name_width: s64, values_width: s64, rule: Rule) {
    border := p.border;
    value_width := values_width;

    if border == 2 {
        append(out, rule.left);
        append(out, rule.horizontal);
    } else if border == 1 append(out, rule.horizontal);

    used := 0;

    if record > 0 {
        label := tprint(ifx border == 0 then "* Record %" else "[ RECORD % ]", record);
        append(out, label);
        used = label.count;
    }

    if border != 2 used += 1;

    fill := ifx border > 0 then rule.horizontal else " ";

    repeat(out, fill, name_width - used);
    used -= name_width;

    // The gap and divider after the name column, then up to the value column's own edge.
    if border > 0 {
        if used <= 0 append(out, rule.horizontal);

        used -= 1;
        if used <= 0 append(out, rule.middle);

        used -= 1;
        if used <= 0 append(out, rule.horizontal);

        used -= 1;
    } else {
        if used <= 0 append(out, " ");
        used -= 1;
    }

    if used < 0 used = 0;
    if value_width < used value_width = used;

    repeat(out, fill, value_width - used);

    if border == 2 {
        append(out, rule.horizontal);
        append(out, rule.right);
    }

    append(out, "\n");
}

// What the vertical layout needs to know about names and values before drawing.
Vertical_Metrics :: struct {
    header_lines: [][] Text_Line;
    cell_lines: [][] Text_Line;
    name_width: s64;
    name_multiline: bool;
    value_width: s64;
    value_multiline: bool;
}

// Where the name and the value of the record cell being drawn stand.
Vertical_Cursor :: struct {
    name_line: s64;
    value_line: s64;
    names_done: bool;
    values_done: bool;
    offset: s64;
    left: s64;
}

// The last piece of a result printed in pieces: the records already said how many.
print_vertical_trailer :: (out: *String_Builder, p: *Layout) {
    if !p.stop_table return;

    footers := ifx p.start_table then p.footers else explicit_footers(p);
    if !p.options.tuples_only print_footer_lines(out, footers);

    append(out, "\n");
}

measure_vertical :: (p: *Layout) -> Vertical_Metrics {
    m: Vertical_Metrics;

    m.header_lines = NewArray(p.columns, [] Text_Line,, temp);

    for 0..p.columns - 1 {
        m.header_lines[it] = format_text(p.headers[it]);
        m.name_width = max(m.name_width, text_width(m.header_lines[it]));
        if m.header_lines[it].count > 1 m.name_multiline = true;
    }

    m.cell_lines = NewArray(p.cells.count, [] Text_Line,, temp);

    for 0..p.cells.count - 1 {
        m.cell_lines[it] = format_text(p.cells[it]);
        m.value_width = max(m.value_width, text_width(m.cell_lines[it]));
        if m.cell_lines[it].count > 1 m.value_multiline = true;
    }

    return m;
}

// Columns taken by what sits between the name and the value.
vertical_separator_width :: (border: s64, name_multiline: bool, value_multiline: bool, old: bool) -> s64 {
    separator: s64;

    if border == 0 {
        separator = 1;
        if name_multiline separator += 1;
    } else if border == 1 {
        separator = 3;
        if name_multiline && old separator += 1;
    } else {
        separator = 7;
    }

    if value_multiline && border < 2 && !old separator += 1;

    return separator;
}

// The width the `[ RECORD n ]` label needs.
vertical_record_width :: (p: *Layout) -> s64 {
    if p.options.tuples_only return 0;

    record_width: s64;
    if p.rows > 0 record_width = digit_count(p.prior_records + p.rows);

    if p.border == 0 return record_width + 9;
    if p.border == 1 return record_width + 12;

    return record_width + 15;
}

// The width of the value column once the name column, the separator and the target are known.
vertical_value_width :: (p: *Layout, m: *Vertical_Metrics, separator: s64, record_width: s64) -> s64 {
    target := cast(s64) p.options.columns;
    width := max(m.name_width + separator + m.value_width, record_width);

    if p.options.format != .WRAPPED || target <= 0 return width - m.name_width - separator;

    min_width := max(m.name_width + separator + 3, record_width);

    if target >= width return width - m.name_width - separator;
    if target < min_width return min_width - m.name_width - separator;

    return target - m.name_width - separator;
}

fit_vertical_values :: (p: *Layout, m: *Vertical_Metrics) {
    old := p.style.old;
    border := p.border;
    separator := vertical_separator_width(border, m.name_multiline, m.value_multiline, old);
    record_width := vertical_record_width(p);
    new_width: s64;

    while true {
        new_width = vertical_value_width(p, m, separator, record_width);

        // Wrapping needs a column for the marker, so make room for it and measure again.
        wraps_without_marker := new_width < m.value_width && !m.value_multiline && border < 2 && !old;
        if !wraps_without_marker break;

        m.value_multiline = true;
        separator += 1;
    }

    m.value_width = new_width;
}

// The rule above a record, or none when there is no label and it is the very first line.
print_vertical_record_rule :: (out: *String_Builder, p: *Layout, m: *Vertical_Metrics, index: s64, record: *s64) {
    border := p.border;
    first := index == 0 && p.start_table;
    rule := ifx first then p.style.top else p.style.separator;

    shown_name_width := m.name_width;

    if border < 2 && m.name_multiline && p.style.old shown_name_width += 1;

    if !p.options.tuples_only {
        print_record_rule(out, p, record.*, shown_name_width, m.value_width, rule);
        record.* += 1;
    } else if !first || border == 2 {
        print_record_rule(out, p, 0, shown_name_width, m.value_width, rule);
    }
}

print_vertical_name :: (out: *String_Builder, p: *Layout, m: *Vertical_Metrics, names: [] Text_Line, c: *Vertical_Cursor) {
    style := *p.style;
    border := p.border;
    old := style.old;

    if c.names_done {
        gap := m.name_width + border;
        if border < 2 && m.name_multiline && old gap += 1;
        if border == 0 && !old && m.name_multiline gap += 1;

        pad(out, gap);

        return;
    }

    if border == 2 || (m.name_multiline && old) {
        append(out, ifx c.name_line > 0 then style.header_newline_left else " ");
    }

    append(out, names[c.name_line].text);
    pad(out, m.name_width - names[c.name_line].width);

    marker := border > 0 || (m.name_multiline && !old);

    if c.name_line + 1 < names.count {
        if marker append(out, style.header_newline_right);
        c.name_line += 1;
    } else {
        if marker append(out, " ");
        c.names_done = true;
    }
}

print_vertical_value :: (out: *String_Builder, p: *Layout, m: *Vertical_Metrics, values: [] Text_Line, c: *Vertical_Cursor) {
    style := *p.style;
    border := p.border;

    if c.values_done {
        if border < 2 {
            append(out, "\n");
        } else {
            pad(out, m.value_width + 2);
            append(out, style.data.right);
            append(out, "\n");
        }

        return;
    }

    append(out, ifx c.offset == 0 then " " else style.wrap_left);

    line := values[c.value_line];
    rest := slice(line.text, c.offset, line.text.count);
    bytes, drawn := fit_width(rest, m.value_width);

    append(out, slice(rest, 0, bytes));

    c.left -= drawn;
    c.offset += bytes;
    spare := m.value_width - drawn;
    marker := border > 1 || (m.value_multiline && !style.old);

    if c.left != 0 {
        if marker {
            pad(out, spare);
            append(out, style.wrap_right);
        }
    } else if c.value_line + 1 < values.count {
        if marker {
            pad(out, spare);
            append(out, style.newline_right);
        }

        c.value_line += 1;
        c.offset = 0;
        c.left = values[c.value_line].width;
    } else {
        if border > 1 {
            pad(out, spare);
            append(out, " ");
        }

        c.values_done = true;
    }

    if border == 2 append(out, style.data.right);

    append(out, "\n");
}

// The divider between the name and the value.
append_vertical_divider :: (out: *String_Builder, style: *Style, c: *Vertical_Cursor) {
    if c.offset > 0 append(out, style.middle_wrap);
    else if c.value_line == 0 append(out, style.data.middle);
    else append(out, style.middle_newline);
}

print_vertical_cell :: (out: *String_Builder, p: *Layout, m: *Vertical_Metrics, index: s64) {
    style := *p.style;
    border := p.border;
    names := m.header_lines[index % p.columns];
    values := m.cell_lines[index];

    c: Vertical_Cursor;
    c.left = values[0].width;

    while !c.values_done || !c.names_done {
        if border == 2 append(out, style.data.left);

        print_vertical_name(out, p, m, names, *c);

        if border > 0 append_vertical_divider(out, style, *c);

        print_vertical_value(out, p, m, values, *c);
    }
}

print_aligned_vertical :: (out: *String_Builder, p: *Layout) {
    options := *p.options;
    border := p.border;

    if p.cells.count == 0 {
        print_vertical_trailer(out, p);
        return;
    }

    m := measure_vertical(p);

    if p.start_table && p.title.count > 0 && !options.tuples_only {
        append(out, p.title);
        append(out, "\n");
    }

    if options.format == .WRAPPED || options.format == .ALIGNED fit_vertical_values(p, *m);

    record := p.prior_records + 1;

    for index: 0..p.cells.count - 1 {
        if index % p.columns == 0 print_vertical_record_rule(out, p, *m, index, *record);

        print_vertical_cell(out, p, *m, index);
    }

    if !p.stop_table return;

    if border == 2 print_record_rule(out, p, 0, m.name_width, m.value_width, p.style.bottom);

    if !options.tuples_only && p.footers.count > 0 && !p.footers_are_default {
        if border < 2 append(out, "\n");

        print_footer_lines(out, p.footers);
    }

    append(out, "\n");
}

digit_count :: (n: s64) -> s64 {
    count := 1;
    value := n;

    while value >= 10 {
        value /= 10;
        count += 1;
    }

    return count;
}

// ---- Unaligned -------------------------------------------------------------------------------

append_field_separator :: (out: *String_Builder, options: *Print_Options) {
    if options.field_separator_zero append(out, cast(u8) 0);
    else append(out, options.field_separator);
}

append_record_separator :: (out: *String_Builder, options: *Print_Options) {
    if options.record_separator_zero append(out, cast(u8) 0);
    else append(out, options.record_separator);
}

end_unaligned :: (out: *String_Builder, options: *Print_Options) {
    if options.record_separator_zero append(out, cast(u8) 0);
    else append(out, "\n");
}

print_unaligned_text :: (out: *String_Builder, p: *Layout) {
    options := *p.options;
    columns := p.columns;
    need_record_separator := false;

    if !options.tuples_only {
        if p.title.count > 0 {
            append(out, p.title);
            append_record_separator(out, options);
        }

        for p.headers {
            if it_index > 0 append_field_separator(out, options);
            append(out, it);
        }

        need_record_separator = true;
    }

    for p.cells {
        if need_record_separator {
            append_record_separator(out, options);
            need_record_separator = false;
        }

        append(out, it);

        if (it_index + 1) % columns != 0 append_field_separator(out, options);
        else need_record_separator = true;
    }

    if !options.tuples_only {
        for p.footers {
            if need_record_separator {
                append_record_separator(out, options);
                need_record_separator = false;
            }

            append(out, it);
            need_record_separator = true;
        }
    }

    // The last record ends with a newline whatever the record separator is, unless that is a zero byte.
    if need_record_separator end_unaligned(out, options);
}

print_unaligned_vertical :: (out: *String_Builder, p: *Layout) {
    options := *p.options;
    columns := p.columns;
    need_record_separator := false;

    if !options.tuples_only && p.title.count > 0 {
        append(out, p.title);
        need_record_separator = true;
    }

    for p.cells {
        if need_record_separator {
            append_record_separator(out, options);
            append_record_separator(out, options);
            need_record_separator = false;
        }

        append(out, p.headers[it_index % columns]);
        append_field_separator(out, options);
        append(out, it);

        if (it_index + 1) % columns != 0 append_record_separator(out, options);
        else need_record_separator = true;
    }

    // Only footers the caller supplied: records are numbered by their blank line instead.
    if !options.tuples_only && p.footers.count > 0 && !p.footers_are_default {
        append_record_separator(out, options);

        for p.footers {
            append_record_separator(out, options);
            append(out, it);
        }
    }

    if need_record_separator end_unaligned(out, options);
}

// ---- CSV -------------------------------------------------------------------------------------

print_csv_field :: (out: *String_Builder, text: string, separator: u8) {
    quote := separator == #char "\\" || separator == #char "." || text == "\\.";

    for text {
        c := it;
        if c == separator || c == #char "\r" || c == #char "\n" || c == #char "\"" {
            quote = true;
            break;
        }
    }

    if !quote {
        append(out, text);
        return;
    }

    append(out, "\"");

    for text {
        if it == #char "\"" append(out, "\"");
        append(out, it);
    }

    append(out, "\"");
}

csv_separator :: (p: *Layout) -> u8 {
    return ifx p.options.csv_field_separator.count > 0 then p.options.csv_field_separator[0] else #char ",";
}

// Titles and footers are never printed in CSV.
print_csv_text :: (out: *String_Builder, p: *Layout) {
    separator := csv_separator(p);

    if !p.options.tuples_only {
        for p.headers {
            if it_index > 0 append(out, separator);
            print_csv_field(out, it, separator);
        }

        append(out, "\n");
    }

    for p.cells {
        print_csv_field(out, it, separator);
        append(out, ifx (it_index + 1) % p.columns != 0 then separator else #char "\n");
    }
}

print_csv_vertical :: (out: *String_Builder, p: *Layout) {
    separator := csv_separator(p);

    for p.cells {
        print_csv_field(out, p.headers[it_index % p.columns], separator);
        append(out, separator);
        print_csv_field(out, it, separator);
        append(out, "\n");
    }
}

// ---- Markup formats --------------------------------------------------------------------------

// Footers the caller supplied; the vertical layouts leave out the "(N rows)" line.
explicit_footers :: (p: *Layout) -> [] string {
    if p.footers_are_default return .[];
    return p.footers;
}

column_align_char :: (p: *Layout, column: s64) -> u8 {
    if column_aligned_right(p, column) return #char "r";
    return #char "l";
}

only_blanks :: (text: string) -> bool {
    for text {
        if it != #char " " && it != #char "\t" return false;
    }

    return true;
}

html_escaped :: (out: *String_Builder, text: string) {
    leading_space := true;

    for text {
        c := it;

        if c == {
            case #char "&"; append(out, "&amp;");
            case #char "<"; append(out, "&lt;");
            case #char ">"; append(out, "&gt;");
            case #char "\n"; append(out, "<br />\n");
            case #char "\""; append(out, "&quot;");
            case #char " "; append(out, ifx leading_space then "&nbsp;" else " "); // keeps indentation in EXPLAIN output
            case; append(out, c);
        }

        if c != #char " " leading_space = false;
    }
}

html_open_table :: (out: *String_Builder, p: *Layout) {
    print(out, "<table border=\"%\"", p.raw_border);
    if p.options.table_attr.count > 0 print(out, " %", p.options.table_attr);

    append(out, ">\n");

    if !p.options.tuples_only && p.title.count > 0 {
        append(out, "  <caption>");
        html_escaped(out, p.title);
        append(out, "</caption>\n");
    }
}

html_close_table :: (out: *String_Builder, p: *Layout, footers: [] string) {
    append(out, "</table>\n");

    if !p.options.tuples_only && footers.count > 0 {
        append(out, "<p>");

        for footers {
            html_escaped(out, it);
            append(out, "<br />\n");
        }

        append(out, "</p>");
    }

    append(out, "\n");
}

html_cell :: (out: *String_Builder, p: *Layout, text: string, column: s64) {
    print(out, "    <td align=\"%\">", ifx column_aligned_right(p, column) then "right" else "left");

    if only_blanks(text) append(out, "&nbsp; ");
    else html_escaped(out, text);

    append(out, "</td>\n");
}

print_html_text :: (out: *String_Builder, p: *Layout) {
    html_open_table(out, p);

    if !p.options.tuples_only {
        append(out, "  <tr>\n");

        for p.headers {
            append(out, "    <th align=\"center\">");
            html_escaped(out, it);
            append(out, "</th>\n");
        }

        append(out, "  </tr>\n");
    }

    for p.cells {
        column := it_index % p.columns;
        if column == 0 append(out, "  <tr valign=\"top\">\n");

        html_cell(out, p, it, column);

        if column == p.columns - 1 append(out, "  </tr>\n");
    }

    html_close_table(out, p, p.footers);
}

print_html_vertical :: (out: *String_Builder, p: *Layout) {
    html_open_table(out, p);

    record := 1;

    for p.cells {
        column := it_index % p.columns;

        if column == 0 {
            if !p.options.tuples_only {
                print(out, "\n  <tr><td colspan=\"2\" align=\"center\">Record %</td></tr>\n", record);
                record += 1;
            } else {
                append(out, "\n  <tr><td colspan=\"2\">&nbsp;</td></tr>\n");
            }
        }

        append(out, "  <tr valign=\"top\">\n    <th>");
        html_escaped(out, p.headers[column]);
        append(out, "</th>\n");

        html_cell(out, p, it, column);

        append(out, "  </tr>\n");
    }

    html_close_table(out, p, explicit_footers(p));
}

asciidoc_escaped :: (out: *String_Builder, text: string) {
    for text {
        if it == #char "|" append(out, "\\");
        append(out, it);
    }
}

asciidoc_open_table :: (out: *String_Builder, p: *Layout, columns: string) {
    append(out, "\n");

    if !p.options.tuples_only && p.title.count > 0 {
        append(out, ".");
        append(out, p.title);
        append(out, "\n");
    }

    append(out, columns);

    if p.raw_border == 0 append(out, ",frame=\"none\",grid=\"none\"");
    else if p.raw_border == 1 append(out, ",frame=\"none\"");
    else if p.raw_border == 2 append(out, ",frame=\"all\",grid=\"all\"");

    append(out, "]\n|====\n");
}

asciidoc_footers :: (out: *String_Builder, p: *Layout, footers: [] string) {
    if p.options.tuples_only || footers.count == 0 return;

    append(out, "\n....\n");

    for footers {
        append(out, it);
        append(out, "\n");
    }

    append(out, "....\n");
}

print_asciidoc_text :: (out: *String_Builder, p: *Layout) {
    spec: String_Builder;

    defer free_buffers(*spec);

    append(*spec, ifx p.options.tuples_only then "[cols=\"" else "[options=\"header\",cols=\"");

    for 0..p.columns - 1 {
        if it > 0 append(*spec, ",");
        append(*spec, ifx column_aligned_right(p, it) then ">l" else "<l");
    }

    append(*spec, "\"");

    asciidoc_open_table(out, p, builder_to_string(*spec,, temp));

    if !p.options.tuples_only {
        for p.headers {
            if it_index > 0 append(out, " ");
            append(out, "^l|");
            asciidoc_escaped(out, it);
        }

        append(out, "\n");
    }

    for p.cells {
        column := it_index % p.columns;

        if column != 0 append(out, " ");

        append(out, "|");

        if only_blanks(it) {
            if column != p.columns - 1 append(out, " ");
        } else {
            asciidoc_escaped(out, it);
        }

        if column == p.columns - 1 append(out, "\n");
    }

    append(out, "|====\n");

    asciidoc_footers(out, p, p.footers);
}

print_asciidoc_vertical :: (out: *String_Builder, p: *Layout) {
    asciidoc_open_table(out, p, "[cols=\"h,l\"");

    record := 1;

    for p.cells {
        column := it_index % p.columns;

        if column == 0 {
            if !p.options.tuples_only {
                print(out, "2+^|Record %\n", record);
                record += 1;
            } else {
                append(out, "2+|\n");
            }
        }

        append(out, "<l|");
        asciidoc_escaped(out, p.headers[column]);

        print(out, " %|", ifx column_aligned_right(p, column) then ">l" else "<l");

        if only_blanks(it) append(out, " ");
        else asciidoc_escaped(out, it);

        append(out, "\n");
    }

    append(out, "|====\n");

    asciidoc_footers(out, p, explicit_footers(p));
}

latex_escaped :: (out: *String_Builder, text: string) {
    for text {
        c := it;

        if c == {
            case #char "#"; append(out, "\\#");
            case #char "$"; append(out, "\\$");
            case #char "%"; append(out, "\\%");
            case #char "&"; append(out, "\\&");
            case #char "<"; append(out, "\\textless{}");
            case #char ">"; append(out, "\\textgreater{}");
            case #char "\\"; append(out, "\\textbackslash{}");
            case #char "^"; append(out, "\\^{}");
            case #char "_"; append(out, "\\_");
            case #char "{"; append(out, "\\{");
            case #char "|"; append(out, "\\textbar{}");
            case #char "}"; append(out, "\\}");
            case #char "~"; append(out, "\\~{}");
            case #char "\n"; append(out, "\\\\");
            case; append(out, c);
        }
    }
}

latex_title :: (out: *String_Builder, p: *Layout) {
    if p.options.tuples_only || p.title.count == 0 return;

    append(out, "\\begin{center}\n");
    latex_escaped(out, p.title);
    append(out, "\n\\end{center}\n\n");
}

latex_footers :: (out: *String_Builder, p: *Layout, footers: [] string) {
    append(out, "\\end{tabular}\n\n\\noindent ");

    if !p.options.tuples_only {
        for footers {
            latex_escaped(out, it);
            append(out, " \\\\\n");
        }
    }

    append(out, "\n");
}

print_latex_text :: (out: *String_Builder, p: *Layout) {
    border := min(p.raw_border, 3);
    tuples_only := p.options.tuples_only;

    latex_title(out, p);

    append(out, "\\begin{tabular}{");

    if border >= 2 append(out, "| ");

    for 0..p.columns - 1 {
        append(out, column_align_char(p, it));
        if border != 0 && it < p.columns - 1 append(out, " | ");
    }

    if border >= 2 append(out, " |");

    append(out, "}\n");

    if !tuples_only && border >= 2 append(out, "\\hline\n");

    if !tuples_only {
        for p.headers {
            if it_index > 0 append(out, " & ");
            append(out, "\\textit{");
            latex_escaped(out, it);
            append(out, "}");
        }

        append(out, " \\\\\n\\hline\n");
    }

    for p.cells {
        latex_escaped(out, it);

        if (it_index + 1) % p.columns == 0 {
            append(out, " \\\\\n");
            if border == 3 append(out, "\\hline\n");
        } else {
            append(out, " & ");
        }
    }

    if border == 2 append(out, "\\hline\n");

    latex_footers(out, p, p.footers);
}

// The next whitespace-separated word of `text` from `position`, advancing it. Empty at the end.
next_word :: (text: string, position: *s64) -> string {
    while position.* < text.count && is_attr_space(text[position.*]) position.* += 1;

    start := position.*;
    while position.* < text.count && !is_attr_space(text[position.*]) position.* += 1;

    return slice(text, start, position.* - start);
}

is_attr_space :: (c: u8) -> bool {
    return c == #char " " || c == #char "\t" || c == #char "\n";
}

print_latex_longtable_text :: (out: *String_Builder, p: *Layout) {
    border := min(p.raw_border, 3);
    tuples_only := p.options.tuples_only;
    attr := p.options.table_attr;
    position := 0;
    last_width: string;
    have_attr := attr.count > 0;

    append(out, "\\begin{longtable}{");

    if border >= 2 append(out, "| ");

    for 0..p.columns - 1 {
        left := !column_aligned_right(p, it);

        if left && have_attr {
            word := next_word(attr, *position);

            if word.count > 0 {
                last_width = word;
                print(out, "p{%\\textwidth}", word);
            } else if last_width.count > 0 {
                print(out, "p{%\\textwidth}", last_width);
            } else {
                append(out, "l");
            }
        } else {
            append(out, column_align_char(p, it));
        }

        if border != 0 && it < p.columns - 1 append(out, " | ");
    }

    if border >= 2 append(out, " |");

    append(out, "}\n");

    if !tuples_only {
        for 0..1 {
            if border >= 2 append(out, "\\toprule\n");

            for header, h: p.headers {
                if h > 0 append(out, " & ");
                append(out, "\\small\\textbf{\\textit{");
                latex_escaped(out, header);
                append(out, "}}");
            }

            append(out, " \\\\\n");

            if it == 0 append(out, "\\midrule\n\\endfirsthead\n");
            else {
                // A rule already follows each row when border is 3
                if border != 3 append(out, "\\midrule\n");
                append(out, "\\endhead\n");
            }
        }

        if p.title.count > 0 {
            if border == 2 append(out, "\\bottomrule\n");

            append(out, "\\caption[");
            latex_escaped(out, p.title);
            append(out, " (Continued)]{");
            latex_escaped(out, p.title);
            append(out, "}\n\\endfoot\n");

            if border == 2 append(out, "\\bottomrule\n");

            append(out, "\\caption[");
            latex_escaped(out, p.title);
            append(out, "]{");
            latex_escaped(out, p.title);
            append(out, "}\n\\endlastfoot\n");
        } else if border >= 2 {
            append(out, "\\bottomrule\n\\endfoot\n\\bottomrule\n\\endlastfoot\n");
        }
    }

    for p.cells {
        if it_index % p.columns != 0 append(out, "\n&\n");

        append(out, "\\raggedright{");
        latex_escaped(out, it);
        append(out, "}");

        if (it_index + 1) % p.columns == 0 {
            append(out, " \\tabularnewline\n");
            if border == 3 append(out, " \\hline\n");
        }
    }

    append(out, "\\end{longtable}\n");
}

print_latex_vertical :: (out: *String_Builder, p: *Layout) {
    border := min(p.border, 2);
    tuples_only := p.options.tuples_only;

    latex_title(out, p);

    append(out, "\\begin{tabular}{");
    append(out, ifx border == 0 then "cl" else ifx border == 1 then "c|l" else "|c|l|");
    append(out, "}\n");

    record := 1;

    for p.cells {
        column := it_index % p.columns;

        if column == 0 {
            if !tuples_only {
                if border == 2 {
                    append(out, "\\hline\n");
                    print(out, "\\multicolumn{2}{|c|}{\\textit{Record %}} \\\\\n", record);
                } else {
                    print(out, "\\multicolumn{2}{c}{\\textit{Record %}} \\\\\n", record);
                }

                record += 1;
            }

            if border >= 1 append(out, "\\hline\n");
        }

        latex_escaped(out, p.headers[column]);
        append(out, " & ");
        latex_escaped(out, it);
        append(out, " \\\\\n");
    }

    if border == 2 append(out, "\\hline\n");

    latex_footers(out, p, explicit_footers(p));
}

troff_escaped :: (out: *String_Builder, text: string) {
    for text {
        if it == #char "\\" append(out, "\\(rs");
        else append(out, it);
    }
}

troff_open :: (out: *String_Builder, p: *Layout, border: s64) {
    if !p.options.tuples_only && p.title.count > 0 {
        append(out, ".LP\n.DS C\n");
        troff_escaped(out, p.title);
        append(out, "\n.DE\n");
    }

    append(out, ".LP\n.TS\n");
    append(out, ifx border == 2 then "center box;\n" else "center;\n");
}

troff_close :: (out: *String_Builder, p: *Layout, footers: [] string) {
    append(out, ".TE\n.DS L\n");

    if !p.options.tuples_only {
        for footers {
            troff_escaped(out, it);
            append(out, "\n");
        }
    }

    append(out, ".DE\n");
}

print_troff_ms_text :: (out: *String_Builder, p: *Layout) {
    border := min(p.border, 2);

    troff_open(out, p, border);

    for 0..p.columns - 1 {
        append(out, column_align_char(p, it));
        if border > 0 && it < p.columns - 1 append(out, " | ");
    }

    append(out, ".\n");

    if !p.options.tuples_only {
        for p.headers {
            if it_index > 0 append(out, "\t");
            append(out, "\\fI");
            troff_escaped(out, it);
            append(out, "\\fP");
        }

        append(out, "\n_\n");
    }

    for p.cells {
        troff_escaped(out, it);
        append(out, ifx (it_index + 1) % p.columns == 0 then "\n" else "\t");
    }

    troff_close(out, p, p.footers);
}

print_troff_ms_vertical :: (out: *String_Builder, p: *Layout) {
    border := min(p.border, 2);
    tuples_only := p.options.tuples_only;

    troff_open(out, p, border);

    if tuples_only append(out, "c l;\n");

    current_format := 0; // 0 none, 1 record heading, 2 body
    record := 1;

    for p.cells {
        column := it_index % p.columns;

        if column == 0 {
            if !tuples_only {
                if current_format != 1 {
                    if border == 2 && record > 1 append(out, "_\n");
                    if current_format != 0 append(out, ".T&\n");

                    append(out, "c s.\n");
                    current_format = 1;
                }

                print(out, "\\fIRecord %\\fP\n", record);
                record += 1;
            }

            if border >= 1 append(out, "_\n");
        }

        if !tuples_only && current_format != 2 {
            if current_format != 0 append(out, ".T&\n");
            append(out, ifx border != 1 then "c l.\n" else "c | l.\n");
            current_format = 2;
        }

        troff_escaped(out, p.headers[column]);
        append(out, "\t");
        troff_escaped(out, it);
        append(out, "\n");
    }

    troff_close(out, p, explicit_footers(p));
}

// ---- Width tables ----------------------------------------------------------------------------

// Unicode code points that take no column: combining marks and format characters.
NONSPACING :: u32.[
    0x00AD, 0x00AD, 0x0300, 0x036F, 0x0483, 0x0489, 0x0591, 0x05BD, 0x05BF, 0x05BF, 0x05C1, 0x05C2,
    0x05C4, 0x05C5, 0x05C7, 0x05C7, 0x0600, 0x0605, 0x0610, 0x061A, 0x061C, 0x061C, 0x064B, 0x065F,
    0x0670, 0x0670, 0x06D6, 0x06DD, 0x06DF, 0x06E4, 0x06E7, 0x06E8, 0x06EA, 0x06ED, 0x070F, 0x070F,
    0x0711, 0x0711, 0x0730, 0x074A, 0x07A6, 0x07B0, 0x07EB, 0x07F3, 0x07FD, 0x07FD, 0x0816, 0x0819,
    0x081B, 0x0823, 0x0825, 0x0827, 0x0829, 0x082D, 0x0859, 0x085B, 0x0890, 0x089F, 0x08CA, 0x0902,
    0x093A, 0x093A, 0x093C, 0x093C, 0x0941, 0x0948, 0x094D, 0x094D, 0x0951, 0x0957, 0x0962, 0x0963,
    0x0981, 0x0981, 0x09BC, 0x09BC, 0x09C1, 0x09C4, 0x09CD, 0x09CD, 0x09E2, 0x09E3, 0x09FE, 0x0A02,
    0x0A3C, 0x0A3C, 0x0A41, 0x0A51, 0x0A70, 0x0A71, 0x0A75, 0x0A75, 0x0A81, 0x0A82, 0x0ABC, 0x0ABC,
    0x0AC1, 0x0AC8, 0x0ACD, 0x0ACD, 0x0AE2, 0x0AE3, 0x0AFA, 0x0B01, 0x0B3C, 0x0B3C, 0x0B3F, 0x0B3F,
    0x0B41, 0x0B44, 0x0B4D, 0x0B56, 0x0B62, 0x0B63, 0x0B82, 0x0B82, 0x0BC0, 0x0BC0, 0x0BCD, 0x0BCD,
    0x0C00, 0x0C00, 0x0C04, 0x0C04, 0x0C3C, 0x0C3C, 0x0C3E, 0x0C40, 0x0C46, 0x0C56, 0x0C62, 0x0C63,
    0x0C81, 0x0C81, 0x0CBC, 0x0CBC, 0x0CBF, 0x0CBF, 0x0CC6, 0x0CC6, 0x0CCC, 0x0CCD, 0x0CE2, 0x0CE3,
    0x0D00, 0x0D01, 0x0D3B, 0x0D3C, 0x0D41, 0x0D44, 0x0D4D, 0x0D4D, 0x0D62, 0x0D63, 0x0D81, 0x0D81,
    0x0DCA, 0x0DCA, 0x0DD2, 0x0DD6, 0x0E31, 0x0E31, 0x0E34, 0x0E3A, 0x0E47, 0x0E4E, 0x0EB1, 0x0EB1,
    0x0EB4, 0x0EBC, 0x0EC8, 0x0ECE, 0x0F18, 0x0F19, 0x0F35, 0x0F35, 0x0F37, 0x0F37, 0x0F39, 0x0F39,
    0x0F71, 0x0F7E, 0x0F80, 0x0F84, 0x0F86, 0x0F87, 0x0F8D, 0x0FBC, 0x0FC6, 0x0FC6, 0x102D, 0x1030,
    0x1032, 0x1037, 0x1039, 0x103A, 0x103D, 0x103E, 0x1058, 0x1059, 0x105E, 0x1060, 0x1071, 0x1074,
    0x1082, 0x1082, 0x1085, 0x1086, 0x108D, 0x108D, 0x109D, 0x109D, 0x135D, 0x135F, 0x1712, 0x1714,
    0x1732, 0x1733, 0x1752, 0x1753, 0x1772, 0x1773, 0x17B4, 0x17B5, 0x17B7, 0x17BD, 0x17C6, 0x17C6,
    0x17C9, 0x17D3, 0x17DD, 0x17DD, 0x180B, 0x180F, 0x1885, 0x1886, 0x18A9, 0x18A9, 0x1920, 0x1922,
    0x1927, 0x1928, 0x1932, 0x1932, 0x1939, 0x193B, 0x1A17, 0x1A18, 0x1A1B, 0x1A1B, 0x1A56, 0x1A56,
    0x1A58, 0x1A60, 0x1A62, 0x1A62, 0x1A65, 0x1A6C, 0x1A73, 0x1A7F, 0x1AB0, 0x1B03, 0x1B34, 0x1B34,
    0x1B36, 0x1B3A, 0x1B3C, 0x1B3C, 0x1B42, 0x1B42, 0x1B6B, 0x1B73, 0x1B80, 0x1B81, 0x1BA2, 0x1BA5,
    0x1BA8, 0x1BA9, 0x1BAB, 0x1BAD, 0x1BE6, 0x1BE6, 0x1BE8, 0x1BE9, 0x1BED, 0x1BED, 0x1BEF, 0x1BF1,
    0x1C2C, 0x1C33, 0x1C36, 0x1C37, 0x1CD0, 0x1CD2, 0x1CD4, 0x1CE0, 0x1CE2, 0x1CE8, 0x1CED, 0x1CED,
    0x1CF4, 0x1CF4, 0x1CF8, 0x1CF9, 0x1DC0, 0x1DFF, 0x200B, 0x200F, 0x202A, 0x202E, 0x2060, 0x206F,
    0x20D0, 0x20F0, 0x2CEF, 0x2CF1, 0x2D7F, 0x2D7F, 0x2DE0, 0x2DFF, 0x302A, 0x302D, 0x3099, 0x309A,
    0xA66F, 0xA672, 0xA674, 0xA67D, 0xA69E, 0xA69F, 0xA6F0, 0xA6F1, 0xA802, 0xA802, 0xA806, 0xA806,
    0xA80B, 0xA80B, 0xA825, 0xA826, 0xA82C, 0xA82C, 0xA8C4, 0xA8C5, 0xA8E0, 0xA8F1, 0xA8FF, 0xA8FF,
    0xA926, 0xA92D, 0xA947, 0xA951, 0xA980, 0xA982, 0xA9B3, 0xA9B3, 0xA9B6, 0xA9B9, 0xA9BC, 0xA9BD,
    0xA9E5, 0xA9E5, 0xAA29, 0xAA2E, 0xAA31, 0xAA32, 0xAA35, 0xAA36, 0xAA43, 0xAA43, 0xAA4C, 0xAA4C,
    0xAA7C, 0xAA7C, 0xAAB0, 0xAAB0, 0xAAB2, 0xAAB4, 0xAAB7, 0xAAB8, 0xAABE, 0xAABF, 0xAAC1, 0xAAC1,
    0xAAEC, 0xAAED, 0xAAF6, 0xAAF6, 0xABE5, 0xABE5, 0xABE8, 0xABE8, 0xABED, 0xABED, 0xFB1E, 0xFB1E,
    0xFE00, 0xFE0F, 0xFE20, 0xFE2F, 0xFEFF, 0xFEFF, 0xFFF9, 0xFFFB, 0x101FD, 0x101FD, 0x102E0, 0x102E0,
    0x10376, 0x1037A, 0x10A01, 0x10A0F, 0x10A38, 0x10A3F, 0x10AE5, 0x10AE6, 0x10D24, 0x10D27, 0x10EAB, 0x10EAC,
    0x10EFD, 0x10EFF, 0x10F46, 0x10F50, 0x10F82, 0x10F85, 0x11001, 0x11001, 0x11038, 0x11046, 0x11070, 0x11070,
    0x11073, 0x11074, 0x1107F, 0x11081, 0x110B3, 0x110B6, 0x110B9, 0x110BA, 0x110BD, 0x110BD, 0x110C2, 0x110CD,
    0x11100, 0x11102, 0x11127, 0x1112B, 0x1112D, 0x11134, 0x11173, 0x11173, 0x11180, 0x11181, 0x111B6, 0x111BE,
    0x111C9, 0x111CC, 0x111CF, 0x111CF, 0x1122F, 0x11231, 0x11234, 0x11234, 0x11236, 0x11237, 0x1123E, 0x1123E,
    0x11241, 0x11241, 0x112DF, 0x112DF, 0x112E3, 0x112EA, 0x11300, 0x11301, 0x1133B, 0x1133C, 0x11340, 0x11340,
    0x11366, 0x11374, 0x11438, 0x1143F, 0x11442, 0x11444, 0x11446, 0x11446, 0x1145E, 0x1145E, 0x114B3, 0x114B8,
    0x114BA, 0x114BA, 0x114BF, 0x114C0, 0x114C2, 0x114C3, 0x115B2, 0x115B5, 0x115BC, 0x115BD, 0x115BF, 0x115C0,
    0x115DC, 0x115DD, 0x11633, 0x1163A, 0x1163D, 0x1163D, 0x1163F, 0x11640, 0x116AB, 0x116AB, 0x116AD, 0x116AD,
    0x116B0, 0x116B5, 0x116B7, 0x116B7, 0x1171D, 0x1171F, 0x11722, 0x11725, 0x11727, 0x1172B, 0x1182F, 0x11837,
    0x11839, 0x1183A, 0x1193B, 0x1193C, 0x1193E, 0x1193E, 0x11943, 0x11943, 0x119D4, 0x119DB, 0x119E0, 0x119E0,
    0x11A01, 0x11A0A, 0x11A33, 0x11A38, 0x11A3B, 0x11A3E, 0x11A47, 0x11A47, 0x11A51, 0x11A56, 0x11A59, 0x11A5B,
    0x11A8A, 0x11A96, 0x11A98, 0x11A99, 0x11C30, 0x11C3D, 0x11C3F, 0x11C3F, 0x11C92, 0x11CA7, 0x11CAA, 0x11CB0,
    0x11CB2, 0x11CB3, 0x11CB5, 0x11CB6, 0x11D31, 0x11D45, 0x11D47, 0x11D47, 0x11D90, 0x11D91, 0x11D95, 0x11D95,
    0x11D97, 0x11D97, 0x11EF3, 0x11EF4, 0x11F00, 0x11F01, 0x11F36, 0x11F3A, 0x11F40, 0x11F40, 0x11F42, 0x11F42,
    0x13430, 0x13440, 0x13447, 0x13455, 0x16AF0, 0x16AF4, 0x16B30, 0x16B36, 0x16F4F, 0x16F4F, 0x16F8F, 0x16F92,
    0x16FE4, 0x16FE4, 0x1BC9D, 0x1BC9E, 0x1BCA0, 0x1CF46, 0x1D167, 0x1D169, 0x1D173, 0x1D182, 0x1D185, 0x1D18B,
    0x1D1AA, 0x1D1AD, 0x1D242, 0x1D244, 0x1DA00, 0x1DA36, 0x1DA3B, 0x1DA6C, 0x1DA75, 0x1DA75, 0x1DA84, 0x1DA84,
    0x1DA9B, 0x1DAAF, 0x1E000, 0x1E02A, 0x1E08F, 0x1E08F, 0x1E130, 0x1E136, 0x1E2AE, 0x1E2AE, 0x1E2EC, 0x1E2EF,
    0x1E4EC, 0x1E4EF, 0x1E8D0, 0x1E8D6, 0x1E944, 0x1E94A, 0xE0001, 0xE01EF,
];

// Unicode code points that take two columns: East Asian wide and fullwidth.
WIDE :: u32.[
    0x1100, 0x115F, 0x231A, 0x231B, 0x2329, 0x232A, 0x23E9, 0x23EC, 0x23F0, 0x23F0, 0x23F3, 0x23F3,
    0x25FD, 0x25FE, 0x2614, 0x2615, 0x2648, 0x2653, 0x267F, 0x267F, 0x2693, 0x2693, 0x26A1, 0x26A1,
    0x26AA, 0x26AB, 0x26BD, 0x26BE, 0x26C4, 0x26C5, 0x26CE, 0x26CE, 0x26D4, 0x26D4, 0x26EA, 0x26EA,
    0x26F2, 0x26F3, 0x26F5, 0x26F5, 0x26FA, 0x26FA, 0x26FD, 0x26FD, 0x2705, 0x2705, 0x270A, 0x270B,
    0x2728, 0x2728, 0x274C, 0x274C, 0x274E, 0x274E, 0x2753, 0x2755, 0x2757, 0x2757, 0x2795, 0x2797,
    0x27B0, 0x27B0, 0x27BF, 0x27BF, 0x2B1B, 0x2B1C, 0x2B50, 0x2B50, 0x2B55, 0x2B55, 0x2E80, 0x2E99,
    0x2E9B, 0x2EF3, 0x2F00, 0x2FD5, 0x2FF0, 0x303E, 0x3041, 0x3096, 0x3099, 0x30FF, 0x3105, 0x312F,
    0x3131, 0x318E, 0x3190, 0x31E3, 0x31EF, 0x321E, 0x3220, 0x3247, 0x3250, 0x4DBF, 0x4E00, 0xA48C,
    0xA490, 0xA4C6, 0xA960, 0xA97C, 0xAC00, 0xD7A3, 0xF900, 0xFAFF, 0xFE10, 0xFE19, 0xFE30, 0xFE52,
    0xFE54, 0xFE66, 0xFE68, 0xFE6B, 0xFF01, 0xFF60, 0xFFE0, 0xFFE6, 0x16FE0, 0x16FE4, 0x16FF0, 0x16FF1,
    0x17000, 0x187F7, 0x18800, 0x18CD5, 0x18D00, 0x18D08, 0x1AFF0, 0x1AFF3, 0x1AFF5, 0x1AFFB, 0x1AFFD, 0x1AFFE,
    0x1B000, 0x1B122, 0x1B132, 0x1B132, 0x1B150, 0x1B152, 0x1B155, 0x1B155, 0x1B164, 0x1B167, 0x1B170, 0x1B2FB,
    0x1F004, 0x1F004, 0x1F0CF, 0x1F0CF, 0x1F18E, 0x1F18E, 0x1F191, 0x1F19A, 0x1F200, 0x1F202, 0x1F210, 0x1F23B,
    0x1F240, 0x1F248, 0x1F250, 0x1F251, 0x1F260, 0x1F265, 0x1F300, 0x1F320, 0x1F32D, 0x1F335, 0x1F337, 0x1F37C,
    0x1F37E, 0x1F393, 0x1F3A0, 0x1F3CA, 0x1F3CF, 0x1F3D3, 0x1F3E0, 0x1F3F0, 0x1F3F4, 0x1F3F4, 0x1F3F8, 0x1F43E,
    0x1F440, 0x1F440, 0x1F442, 0x1F4FC, 0x1F4FF, 0x1F53D, 0x1F54B, 0x1F54E, 0x1F550, 0x1F567, 0x1F57A, 0x1F57A,
    0x1F595, 0x1F596, 0x1F5A4, 0x1F5A4, 0x1F5FB, 0x1F64F, 0x1F680, 0x1F6C5, 0x1F6CC, 0x1F6CC, 0x1F6D0, 0x1F6D2,
    0x1F6D5, 0x1F6D7, 0x1F6DC, 0x1F6DF, 0x1F6EB, 0x1F6EC, 0x1F6F4, 0x1F6FC, 0x1F7E0, 0x1F7EB, 0x1F7F0, 0x1F7F0,
    0x1F90C, 0x1F93A, 0x1F93C, 0x1F945, 0x1F947, 0x1F9FF, 0x1FA70, 0x1FA7C, 0x1FA80, 0x1FA88, 0x1FA90, 0x1FABD,
    0x1FABF, 0x1FAC5, 0x1FACE, 0x1FADB, 0x1FAE0, 0x1FAE8, 0x1FAF0, 0x1FAF8, 0x20000, 0x2FFFD, 0x30000, 0x3FFFD,
];
