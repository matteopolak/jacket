B :: #import "Bridge";
N :: #import "Leaf"(FLAG = true, SECOND = true);

#assert type_of(B.pointer()) == *N.Record;
main :: () {}
