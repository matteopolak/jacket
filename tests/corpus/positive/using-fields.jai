Inner :: struct { number: int; }
Outer :: struct { using inner: Inner; }
main :: () -> int { item: Outer; item.number = 31; return item.inner.number; }
