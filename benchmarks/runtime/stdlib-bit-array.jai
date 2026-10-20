// Bit_Array: setting, clearing and toggling scattered bits, then counting the set ones.
#import "Basic";
#import "Bit_Array";

main :: () {
    SIZE :: 1 << 20;
    bits: Bit_Array;

    init_bit_array(*bits, SIZE);

    for 0..400_000_000 - 1 {
        index := (it * 7_919) & (SIZE - 1);
        if it % 3 == 0 clear_bit(*bits, index);
        else set_bit(*bits, index);
        toggle_bit(*bits, (index * 3) & (SIZE - 1));
    }

    set_count := 0;
    for 0..9 for bits if it set_count += 1;

    print("% %\n", set_count, bits[12345]);
}
