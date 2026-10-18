// A #run of a procedure with no result cannot give a constant its value.
hi :: () {}

X :: #run hi();

main :: () {}
