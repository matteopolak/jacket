main :: () -> int { a: [3]int = .[2, 5, 8]; b := a; a[1] = 90; return b[0]+b[1]+b[2]; }
