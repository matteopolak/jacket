// rules: ctexec.10
// A false `#assert cond, "msg"` fails the compilation with the message.
SIZE :: 4;
#assert SIZE == 8, "SIZE must be eight";

main :: () {}
