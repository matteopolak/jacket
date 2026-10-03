AuthoredLibrary :: #library,system "authored-missing-library";
authored_foreign :: () -> int #foreign AuthoredLibrary "authored_missing_symbol";
main :: () -> int { return authored_foreign(); }
