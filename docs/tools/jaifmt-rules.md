# Formatting

What `jaifmt` (and `jacket fmt`) changes, with the code before and after. The formatter moves line breaks, indentation and spaces, and never changes what a program means: output whose tokens differ from the input is refused. Lines are never re-wrapped. [jaifmt](jaifmt.md) says how to run it and where settings live.

What every run does is on this page. The settings change the details; they go in the `[format]` table of the nearest `jacket.toml`:

```toml
[format]
indent_width = 2
blank_line_before_return = true
```

## Indentation

Each line is indented by how deeply it is nested in `{ }`, `( )` and `[ ]`. Continuation lines keep their offset from the line they continue.

```jai before
main :: () {
for i: 0..2 {
if i == 1 {
          print("%\n", i);
}
}
}
```

```jai after
main :: () {
    for i: 0..2 {
        if i == 1 {
            print("%\n", i);
        }
    }
}
```

## Bodies on their own lines

A non-empty `{ }` body is split onto its own lines. Literals (`.{ }`, `.[ ]`) stay as written.

```jai before
main :: () {
    if ready { start(); }
    point := Vector2.{ 1, 2 };
}
```

```jai after
main :: () {
    if ready {
        start();
    }
    point := Vector2.{ 1, 2 };
}
```

## One statement per line

Statements on one line separated by `;` go on lines of their own.

```jai before
main :: () {
    x := 1; y := 2; z := x + y;
}
```

```jai after
main :: () {
    x := 1;
    y := 2;
    z := x + y;
}
```

## Spacing

Between two tokens there is either no space or one, following the standard library's style. Nothing is aligned with runs of spaces.

```jai before
main :: () {
    x:=add(1 ,2)*3;
    name   :   string = "jai";
}
```

```jai after
main :: () {
    x := add(1, 2) * 3;
    name: string = "jai";
}
```

## Braces

A `{` alone on a line joins its header, and `else` joins the `}` before it. `brace_style` turns this off.

```jai before
main :: () {
    if ok
    {
        go();
    }
    else
    {
        stop();
    }
}
```

```jai after
main :: () {
    if ok {
        go();
    } else {
        stop();
    }
}
```

## Blank lines

Trailing spaces go, a run of blank lines is cut to `max_blank_lines`, and a file ends with one newline. Blank lines between statements are kept, never removed.

```jai before
main :: () {
    a();   




    b();
}
```

```jai after
main :: () {
    a();


    b();
}
```

## Comments and strings

Comments keep their place and text. Strings, here-strings, `#asm` bodies and everything between `// jaifmt: off` and `// jaifmt: on` are left exactly as written. The one exception is the Jai in fenced blocks inside `//` comments, which is formatted too (see [format_doc_examples](#format_doc_examples)).

```jai before
main :: () {
    // jaifmt: off
    table := int.[1,0,
                   0,1];
    // jaifmt: on
    x:=1;   // kept
}
```

```jai after
main :: () {
    // jaifmt: off
    table := int.[1,0,
                   0,1];
    // jaifmt: on
    x := 1; // kept
}
```

## Settings

Each key of `[format]`, with an example of what it does.

### indent_width

Spaces per level of nesting, 1 to 16. Default 4.

```jai before
main :: () {
    if ok {
        go();
    }
}
```

```jai after indent_width=2
main :: () {
  if ok {
    go();
  }
}
```

### case_indent

How far `case` lines sit inside `if x == {`, 0 to 16. Default: `indent_width`.

```jai before
main :: () {
    if x == {
        case 1; one();
        case; other();
    }
}
```

```jai after case_indent=0
main :: () {
    if x == {
    case 1; one();
    case; other();
    }
}
```

### case_body_indent

How far statements under a `case` sit, 0 to 16. Default: `indent_width`.

```jai before
main :: () {
    if x == {
        case 1;
            one();
            more();
    }
}
```

```jai after case_body_indent=2
main :: () {
    if x == {
        case 1;
          one();
          more();
    }
}
```

### max_blank_lines

The longest run of blank lines kept, 0 to 100. Default 2.

```jai before
main :: () {
    a();


    b();
}
```

```jai after max_blank_lines=1
main :: () {
    a();

    b();
}
```

### brace_style

`"same_line"` (the default) joins a lone `{` to its header and `else` to the `}` before it. `"preserve"` leaves them where they are.

```jai before
main :: () {
    if ok
    {
        go();
    }
}
```

```jai after brace_style="preserve"
main :: () {
    if ok
    {
        go();
    }
}
```

### blank_lines_between_items

A blank line around file-level items that span several lines, and around groups of imports. On by default; `false` turns it off.

```jai before
#import "Basic";
main :: () {
    go();
}
go :: () {
    print("go\n");
}
```

```jai after
#import "Basic";

main :: () {
    go();
}

go :: () {
    print("go\n");
}
```

### blank_line_after_declarations

In code, a blank line after a run of declarations. Off by default.

```jai before
main :: () {
    width := 640;
    height := 480;
    open_window(width, height);
    clear_screen();
    present();
}
```

```jai after blank_line_after_declarations=true
main :: () {
    width := 640;
    height := 480;

    open_window(width, height);
    clear_screen();
    present();
}
```

### blank_lines_around_blocks

In code, a blank line before and after a statement with a block body. Off by default.

```jai before
main :: () {
    start();
    for 0..2 {
        step(it);
    }
    stop();
}
```

```jai after blank_lines_around_blocks=true
main :: () {
    start();

    for 0..2 {
        step(it);
    }

    stop();
}
```

### blank_line_between_cases

In an `if x == {` switch, a blank line before a `case` that follows a multi-line arm. Off by default.

```jai before
main :: () {
    if x == {
        case 1;
            one();
            more();
        case 2;
            two();
    }
}
```

```jai after blank_line_between_cases=true
main :: () {
    if x == {
        case 1;
            one();
            more();

        case 2;
            two();
    }
}
```

### blank_line_before_comments

In code, a blank line above a comment that follows code. Off by default.

```jai before
main :: () {
    start();
    // Now the work.
    work();
}
```

```jai after blank_line_before_comments=true
main :: () {
    start();

    // Now the work.
    work();
}
```

### blank_line_after_guards

In code, a blank line after a one-line `if`, `for` or `while`. Off by default.

```jai before
load :: (path: string) -> bool {
    if !path return false;
    if !exists(path) return false;
    data := read(path);
    parse(data);
    return true;
}
```

```jai after blank_line_after_guards=true
load :: (path: string) -> bool {
    if !path return false;
    if !exists(path) return false;

    data := read(path);
    parse(data);
    return true;
}
```

### blank_line_before_return

In code, a blank line above a `return`. Off by default.

```jai before
save :: (path: string) -> bool {
    file := open(path);
    write(file, header);
    write(file, body);
    close(file);
    return true;
}
```

```jai after blank_line_before_return=true
save :: (path: string) -> bool {
    file := open(path);
    write(file, header);
    write(file, body);
    close(file);

    return true;
}
```

### format_doc_examples

Formats the code in `jai` fenced blocks inside `//` comments, such as the examples in doc comments. On by default.

A block is formatted with the file's settings, and its comment prefix and indentation stay. A block with a `main` is formatted as a file; any other block as the body of a procedure, the way `jacket test` runs it. The fence line and its options (`no_run`, `only(linux)`) are never touched.

A block is left exactly as written when its fence has `ignore` or `compile_fail`, when it contains `// jaifmt: off`, when it lies in a `// jaifmt: off` region, when it does not parse, or when formatting it twice would not give the same text.

```jai before
// Adds two numbers.
//
// ```jai
// total:=add(1,2);
// if total>2{
// print("big\n");
// }
// ```
add :: (a: int, b: int) -> int {
    return a + b;
}
```

```jai after
// Adds two numbers.
//
// ```jai
// total := add(1, 2);
// if total > 2 {
//     print("big\n");
// }
// ```
add :: (a: int, b: int) -> int {
    return a + b;
}
```

With `format_doc_examples = false` the comments stay as written.

### max_width

The line length `jaifmt --verbose` reports lines over. Default 100. Lines are never re-wrapped, so it changes no output.

### ignore

Globs, relative to the `jacket.toml`, of files `jacket fmt` skips: `ignore = ["generated/**"]`.
