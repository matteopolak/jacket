f :: (a: ..int, b: int) {}
g :: (a: ..$T) {}
main :: () { f(1, 2, b = 3); g(); g(1, "a"); f(..int.[1, 2]); }
