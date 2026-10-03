#add_context number: int = 40;
bump :: () { context.number += 2; }
main :: () -> int { bump(); return context.number; }
