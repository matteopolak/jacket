// `using,only(answer) M;` inside a procedure hides M's other names there too.
M :: #import,string "answer :: 1;\nother :: 2;\n";
main :: () {
    using,only(answer) M;
    x := answer + other;
}
