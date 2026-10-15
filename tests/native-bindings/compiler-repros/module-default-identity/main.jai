// Original language-contract question: default versus explicit default import.
// No SDK/library implementation or module-name exception is involved.
Implicit :: #import "Example";
Explicit :: #import "Example"(ENABLED = true);

#assert Implicit.Record == Explicit.Record;
main :: () {}
