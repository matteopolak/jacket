#import "Basic";
#import "Math";

main :: () {
    a := 0.1;
    b := 0.2;
    total := a + b;
    expected := 0.3;
    if total == expected print("equal\n");
    if sqrt(a) != b print("different\n");
}
