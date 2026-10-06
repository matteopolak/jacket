// rules: code.5
// A string given to #insert is parsed as its own source text; a syntax error in it is
// reported against `<#insert at ...>`, not the surrounding file.
main :: () {
    #insert "value := 1 +;";
}
