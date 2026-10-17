A :: #import "Leaf"(FLAG = true, SECOND = true);
B :: #import "Leaf"(FLAG = true, SECOND = true);

#assert A.Record == B.Record;
#assert type_of(A.pointer()) == *B.Record;
main :: () {}
