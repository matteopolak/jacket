A :: #import,file "support/literal-params.jai"(MODE = .READ, NAMES = .["ab", "cde"]);
B :: #import,file "support/literal-params.jai"(MODE = .WRITE);
main :: () -> int {
    result := A.total();
    if A.is_read() result += 10;
    if B.is_read() result += 100;
    return result + B.total();
}
