// Two `using` locals may bring different names into scope, and a `using` in an inner block
// takes over a name the outer one also has.
#import "Basic";

Left :: struct {
    model: int;
    wheels: int;
}

Right :: struct {
    colour: int;
    model: int;
}

main :: () {
    left := Left.{ 1, 4 };
    right := Right.{ 2, 3 };

    {
        using left;
        assert(wheels == 4);
    }

    using left;
    assert(model == 1 && wheels == 4);

    {
        using right;
        assert(model == 3 && colour == 2);
    }

    assert(model == 1);
    print("ok\n");
}
