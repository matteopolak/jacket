// Random set, clear, toggle and assign operations on a 197-bit array agree with a plain bool array.
#import "Basic";
Bits :: #import "Bit_Array";
R :: #import "Random";

COUNT :: 197;

main :: () {
    bits := Bits.make_bit_array(COUNT);

    defer Bits.deinit(*bits);

    model: [COUNT] bool;
    state: R.Random_State;

    R.random_seed(*state, cast(u64) 2024);

    for step: 0..19999 {
        i := cast(int) (R.random_get(*state) % COUNT);
        op := R.random_get(*state) % 5;
        value := (R.random_get(*state) & 1) == 1;

        if op == 0 {
            Bits.set_bit(*bits, i);
            model[i] = true;
        } else if op == 1 {
            Bits.clear_bit(*bits, i);
            model[i] = false;
        } else if op == 2 {
            Bits.toggle_bit(*bits, i);
            model[i] = !model[i];
        } else if op == 3 {
            Bits.set_bit_to(*bits, i, value);
            model[i] = value;
        } else {
            bits[i] = value;
            model[i] = value;
        }

        // Compare everything every 100 steps, and read one bit on every step.
        assert(bits[i] == model[i]);

        if step % 100 == 0 {
            for 0..COUNT - 1 assert(bits[it] == model[it]);
        }
    }

    for 0..COUNT - 1 assert(bits[it] == model[it]);

    set_in_model := 0;
    for model if it set_in_model += 1;

    set_in_bits := 0;
    for bits if it set_in_bits += 1;

    assert(set_in_model == set_in_bits);

    print("ok\n");
}
