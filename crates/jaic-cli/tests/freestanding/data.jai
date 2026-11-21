initialized: u32 = 0xc0ffee;
table: [4] u16 = .[10, 20, 30, 40];
name: string = "freestanding";
zeroed: [256] u8;
counter: s64;

main :: () {
    print("% % % %\n", initialized, table[2], name, zeroed[255]);
    counter += 5;
    print("%\n", counter);
}
