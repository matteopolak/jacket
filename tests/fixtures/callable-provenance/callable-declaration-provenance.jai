#import "Basic";

Public_Callback :: (public_option := true) -> s64;
Other_Callback :: (private_option := false) -> s64;
#assert Public_Callback == Other_Callback;

Holder :: struct {
    callback: Public_Callback;
}

Other :: struct {
    callback: Other_Callback;
}

choose :: (flag: bool) -> s64 {
    if flag return 19;
    return 23;
}

global_public: Public_Callback = choose;
global_other: Other_Callback = choose;

invoke :: (callback: Public_Callback) -> s64 {
    return callback(public_option = false);
}

invoke_macro :: (callback: Public_Callback) -> s64 #expand {
    return callback();
}

make :: () -> Public_Callback {
    return choose;
}

Maker :: () -> Public_Callback;

Frozen_Holder :: struct(callback: Public_Callback) {
    storage: s64;
}

Frozen :: Frozen_Holder(#run make());
#assert Frozen.callback == choose;

make_pair :: () -> (Public_Callback, Other_Callback) {
    return choose, choose;
}

Pair_Maker :: () -> (Public_Callback, Other_Callback);

Wrapped :: struct {
    using,only(callback) inner: Holder;
}

Overlay :: struct {
    first: Public_Callback;
    #overlay(first) second: Other_Callback;
}

verify :: () -> bool {
    first: Holder;
    second: Other;

    first.callback = choose;
    second.callback = choose;
    alias := first.callback;
    queried: type_of(first.callback) = choose;
    renamed := cast(Other_Callback) first.callback;
    wrapped: Wrapped;
    wrapped.callback = choose;
    overlaid: Overlay;
    overlaid.first = choose;
    left, right := overlaid.first, overlaid.second;
    factory: Maker = make;
    direct_result := make();
    indirect_result := factory();
    pair_factory: Pair_Maker = make_pair;
    pair_public, pair_other := pair_factory();

    return first.callback(public_option = false) == 23 && first.callback() == 19
        && second.callback() == 23 && alias(public_option = true) == 19
        && queried() == 19 && renamed() == 23
        && global_public() == 19 && global_other() == 23 && invoke(choose) == 23
        && invoke_macro(choose) == 19 && wrapped.callback() == 19
        && direct_result(public_option = false) == 23 && indirect_result() == 19
        && pair_public() == 19 && pair_other() == 23
        && Frozen.callback(true) == 19 && left() == 19 && right() == 23 && *overlaid.first == *overlaid.second;
}

#assert verify();

main :: () {
    assert(verify());
}
