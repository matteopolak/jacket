#import "Basic";

current_label: string;

set_label :: (id: int) {
    current_label = tprint("item %", id);
}

main :: () {
    set_label(1);
    print("%\n", current_label);
}
