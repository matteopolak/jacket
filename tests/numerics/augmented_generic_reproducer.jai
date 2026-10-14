// Minimal language-only reproducer; no stdlib names or paths are involved.
// The reviewed contextual shape uses `using`, not `#as using`.
Key :: struct {
    value: float;
}

Augmented :: struct {
    using base: Key;
    active: bool;
}

sample :: (keys: [] *$T/Key) -> float {
    return keys[0].value;
}

main :: () {
    key: Augmented;
    key.value = 0.75;
    keys: [1] *Augmented = .[*key];
    assert(sample(keys) == 0.75);
}

#scope_file
#import "Basic";
