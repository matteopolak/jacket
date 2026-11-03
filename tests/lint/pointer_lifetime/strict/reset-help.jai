#import "Basic";

reset_borrow :: () {
    text := tprint("temporary");
    reset_temporary_storage();
    print("%", text);
}
