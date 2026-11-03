#import "Basic";

constant_condition :: () {
    p := New(int);
    if false free(p);
    print("%", p.*);
}

unchanged_condition :: (release: bool) {
    p := New(int);
    if release free(p);
    if !release print("%", p.*);
}

changed_condition :: (release: bool) {
    p := New(int);
    if release free(p);
    release = false;
    if !release print("%", p.*);
}

escaped_condition :: (release: bool, update: (p: *bool)) {
    p := New(int);
    if release free(p);
    update(*release);
    if !release print("%", p.*);
}
