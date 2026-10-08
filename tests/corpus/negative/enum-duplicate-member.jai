// Each enum member name is declared once.
Letter :: enum {
    A;
    A;
}

main :: () {
    l := Letter.A;
}
