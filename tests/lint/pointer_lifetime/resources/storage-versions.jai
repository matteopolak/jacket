#import "Basic";
Holder :: struct { storage: *int; }
replace_then_free :: (holder: *Holder, replacement: *int) {
    holder.storage = replacement;
    free(holder.storage);
}
free_then_replace :: (holder: *Holder, replacement: *int) {
    free(holder.storage);
    holder.storage = replacement;
}
unsafe_replacement :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{initial};
    replace_then_free(*holder, replacement);
    print("%", replacement.*); // replacement was freed
    free(initial);
}
unsafe_initial :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{initial};
    free_then_replace(*holder, replacement);
    print("%", initial.*); // initial was freed
    free(replacement);
}
safe_initial :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{initial};
    replace_then_free(*holder, replacement);
    print("%", initial.*);
    free(initial);
}
safe_replacement :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{initial};
    free_then_replace(*holder, replacement);
    print("%", replacement.*);
    free(replacement);
}
main :: () {}
