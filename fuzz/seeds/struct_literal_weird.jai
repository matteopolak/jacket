S :: struct { a: int; b: string; }
main :: () { s := S.{1, "x", 3}; t := S.{c = 1}; u := S.{a = 1, a = 2}; }
