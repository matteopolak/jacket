calls := 0;
bump :: () -> bool { calls += 1; return true; }
main :: () -> int { if false && bump() return 1; if true || bump() return calls + 12; return 2; }
