#import "Basic";

count_down :: (n: u32) {
    i := n;
    while i >= 0 {
        print("%\n", i);
        if i == 0 break;
        i -= 1;
    }
}

check :: (index: u64, small: s8) -> bool {
    if index < 0 return false;
    if 0 > index return false;
    if small < -128 return false;
    return true;
}

main :: () {
    count_down(3);
    print("%\n", check(1, 2));
}
