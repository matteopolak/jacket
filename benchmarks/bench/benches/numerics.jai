// Math functions, the Random module and Bit_Array, one step of the loop per iteration.
#import "Basic";
#import "Math";
#import "Random";
#import "Bit_Array";
#import "Extensions/Testing";

// Square roots, trigonometry and rounding of one input.
math_functions :: (b: *Bench) {
    set_items(b, 8);

    sum: float64;

    for 0..b.iterations - 1 {
        x := cast(float64) it * 0.0001;
        sum += sqrt(x) + sin(x) * cos(x) + abs(x - 500) + floor(x) + atan2(x, 3.0) + exp(-x) + log(x + 1);
    }

    black_box(sum);
} @bench

// A raw value, a float in [0, 1) and a value in a range from a fixed seed.
random_values :: (b: *Bench) {
    set_items(b, 3);
    random_seed(1234);

    checksum: u64 = 1469598103934665603;
    sum: float64;

    for 0..b.iterations - 1 {
        checksum = (checksum ^ random_get()) * 1099511628211;
        sum += random_get_zero_to_one();
        sum += random_get_within_range(-3, 3);
    }

    black_box(checksum);
    black_box(sum);
} @bench

// Setting, clearing and toggling scattered bits of a 1 048 576-bit array.
bit_array_scatter :: (b: *Bench) {
    SIZE :: 1 << 20;
    bits: Bit_Array;

    init_bit_array(*bits, SIZE);
    reset_timer(b);
    set_items(b, 2);

    for 0..b.iterations - 1 {
        index := (it * 7_919) & (SIZE - 1);
        if it % 3 == 0 clear_bit(*bits, index);
        else set_bit(*bits, index);
        toggle_bit(*bits, (index * 3) & (SIZE - 1));
    }

    black_box(bits[12345]);
} @bench

// Iterating over every bit of a half-filled 1 048 576-bit array.
bit_array_iterate :: (b: *Bench) {
    SIZE :: 1 << 20;
    bits: Bit_Array;

    init_bit_array(*bits, SIZE);

    for 0..SIZE - 1 if it % 2 == 0 set_bit(*bits, it);

    reset_timer(b);
    set_items(b, SIZE);

    for 0..b.iterations - 1 {
        set_count := 0;
        for bits if it set_count += 1;
        black_box(set_count);
    }
} @bench @samples(20)
