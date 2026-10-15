// `#ifx cond then a; else b;` accepts the `;` before `else`.
main :: () -> s64 {
    name := #ifx OS == .WINDOWS then "windows";
    else "other";
    expected := ifx OS == .WINDOWS then "windows" else "other";
    return ifx name == expected then 0 else 1;
}
