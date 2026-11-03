#import "Basic";

current_label: string;

set_label :: (id: int) {
    free(current_label);
    current_label = sprint("item %", id);
}

main :: () {
    set_label(1);
    print("%\n", current_label);
}
