#add_context number: int = 42;
bridge :: () -> int #c_call { push_context { return context.number; } }
main :: () -> int { return bridge(); }
