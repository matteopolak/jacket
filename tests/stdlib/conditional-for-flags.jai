// `for *=cond` and `for <=cond` iterate by pointer / in reverse when the constant holds.
#import "Basic";
BY_POINTER :: true;
main :: () {
    values := int.[1, 2, 3];
    for *= BY_POINTER values it.* *= 10;
    assert(values[0] == 10 && values[2] == 30);
    order: [..] int;
    for <= (1 > 0) values array_add(*order, it);
    assert(order[0] == 30 && order[2] == 10);
    for *= false <= false values array_add(*order, it);
    assert(order[3] == 10);
}
