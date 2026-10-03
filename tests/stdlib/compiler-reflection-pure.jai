#load "../../stdlib/Protocol_For_Memory_Visualization.jai";
#load "../../stdlib/Jai_Lexer/classify.jai";
probe :: () -> bool {
    if !starts_identifier(#char "_") || !starts_identifier(#char "Z") return false;
    if starts_identifier(#char "9") || starts_identifier(#char "@") return false;
    if !continues_identifier(#char "0") || continues_identifier(-1) return false;
    if !starts_number(#char "9") || starts_number(#char ":") return false;
    if has_a_big_exponent(0.0) || has_a_big_exponent(1.0) return false;
    if !has_a_big_exponent(1e300) || !has_a_big_exponent(1e-100) return false;
    first, second: s64;
    identical := Alloc_And_Free_Traces.{*first, *second};
    unequal := Alloc_And_Free_Traces.{*first, *first};
    if !compare_alloc_and_free_traces(identical, identical) return false;
    if compare_alloc_and_free_traces(identical, unequal) return false;
    return true;
}
VERIFIED :: #run probe();
#assert VERIFIED;
