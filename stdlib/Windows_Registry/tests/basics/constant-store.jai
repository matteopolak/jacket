// Reviewed public predefined handles are immutable constants, not storage cells.
R :: #import "Windows_Registry";
W :: #import "Windows";

main :: () {
    R.HKEY_CURRENT_USER = cast(W.HKEY) 0;
}
