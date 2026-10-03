update :: (s: []int) { s[1] = 17; }
main :: () -> int { a: [2]int = .[3, 4]; update(a); return a[0]+a[1]; }
