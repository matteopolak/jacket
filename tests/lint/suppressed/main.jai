// Every finding in this file is silenced in source.
// jailint: allow-file(float_equality)
#import "Basic";
#import "Math"; // jailint: allow(unused_import)

double_value :: (value: int, label: string) -> int {
    return value * 2;
} @jailint_allow(unused_parameter)

sum :: (xs: [] int) -> int {
    total := 0;
    // jailint: allow(index_only_loop)
    for i: 0..xs.count - 1 {
        total += xs[i];
    }
    return total;
}

main :: () {
    // jailint: allow(unused_variable, lossy_xx)
    leftover: u8 = xx sum(.[1, 2]);
    ready := double_value(1, "x") > 1;
    if ready == true print("ready\n"); // jailint: allow(bool_comparison)
    print("% %\n", 1); // jailint: allow(format_arg_count)
    x: s32 = 5;
    y := cast(s32) x; // jailint: allow(all)
    a := 0.1;
    b := a * 3;
    if a == b print("same\n");
    print("%\n", y);
}
