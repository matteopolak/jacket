// Reviewed module identity follows the written argument list, not effective
// defaults. No SDK implementation or module-name exception is involved.
Implicit :: #import "Example";
Explicit :: #import "Example"(ENABLED = true);
Repeated_Implicit :: #import "Example";
Repeated_Explicit :: #import "Example"(ENABLED = true);

#assert Implicit.Record != Explicit.Record;
#assert Implicit.Record == Repeated_Implicit.Record;
#assert Explicit.Record == Repeated_Explicit.Record;
#assert *Implicit.state == *Repeated_Implicit.state;
#assert *Explicit.state == *Repeated_Explicit.state;
#assert *Implicit.state != *Explicit.state;
main :: () {}
