// `using,only(answer) M;` brings in only `answer`; M's other names stay qualified.
M :: #import,string "answer :: 1;\nother :: 2;\n";
using,only(answer) M;
main :: () {
    x := answer + other;
}
