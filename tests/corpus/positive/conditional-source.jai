SELECTED :: true;
#if SELECTED { chosen :: 13; } else { broken :: absent; }
main :: () -> int { return chosen; }
