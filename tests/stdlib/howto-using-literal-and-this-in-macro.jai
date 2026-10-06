#import "Basic";

// `using` on an anonymous struct literal, `using` on a global variable declaration, and
// `#this` inside a macro (the procedure the macro expanded into).

Ice_Cream :: struct {
    flavor: string;
    num_scoops: int;
    cone_style: string = "waffle";
}

using best := Ice_Cream.{ flavor = "mint", num_scoops = 3 };

seen_proc: *void;
seen_exit: *void;

Track :: () #expand {
    seen_proc = cast(*void) #this;
    `defer seen_exit = cast(*void) #this;
}

tracked :: () {
    Track();
}

bump :: () {
    num_scoops += 4;
}

main :: () {
    using Ice_Cream.{ num_scoops = 1, cone_style = "cake" };
    assert(num_scoops == 1);
    assert(cone_style == "cake");

    bump();
    assert(best.num_scoops == 7);
    assert(num_scoops == 1);

    tracked();
    assert(seen_proc == cast(*void) tracked);
    assert(seen_exit == cast(*void) tracked);
}
