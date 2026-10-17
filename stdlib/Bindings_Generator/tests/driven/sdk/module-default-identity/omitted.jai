B :: #import "Bridge";
N :: #import "Leaf"(FLAG = true);

#assert type_of(B.pointer()) == *N.Record;
main :: () {}
