Random_State :: struct {
    low: u64;
    high: u64;
}

Some_Big_Time_Value :: struct {
    low: u64;
    high: s64;
}

random_get :: (state: *Random_State) -> u64 #no_aoc {
    high, low := multiply_64bits_with_overflow(state.low, 0x866458cd56f5e605);
    state.high = high + state.high * 0x866458cd56f5e605 + state.low * 0x0fc94e3bf4e9ab32;
    state.low = low;
    return state.high;
}

random_seed :: inline (state: *Random_State, new_seed: u64) {
    state.* = .{ 1, new_seed };
}

random_seed :: inline (state: *Random_State, new_seed: Random_State) {
    state.* = .{ new_seed.low | 1, new_seed.high };
}

random_seed :: inline (state: *Random_State, time: $T/interface Some_Big_Time_Value) {
    state.* = .{ time.low | 1, cast(u64) time.high };
}

random_get :: inline () -> u64 {
    return random_get(*context.mcg_state);
}

random_seed :: inline (new_seed: u64) {
    random_seed(*context.mcg_state, new_seed);
}

random_seed :: inline (new_seed: Random_State) {
    random_seed(*context.mcg_state, new_seed);
}

random_seed :: inline (time: $T/interface Some_Big_Time_Value) {
    random_seed(*context.mcg_state, time);
}

random_get_zero_to_one :: inline (state: *Random_State) -> float {
    return cast(float) (random_get(state) >> 40) / 16777215.0;
}

random_get_zero_to_one_open :: inline (state: *Random_State) -> float {
    return cast(float) (random_get(state) >> 40) / 16777216.0;
}

random_get_zero_to_one_new :: (state: *Random_State) -> float64 #no_aoc {
    result := cast(float64) random_get(state) / 18446744073709551616.0;
    if result == 1 return 0.9999999999999999;
    return result;
}

random_get_zero_to_one_open_new :: (state: *Random_State) -> float64 {
    return cast(float64) (random_get(state) >> 11) / 9007199254740992.0;
}

random_get_zero_to_one :: inline () -> float {
    return random_get_zero_to_one(*context.mcg_state);
}

random_get_zero_to_one_open :: inline () -> float {
    return random_get_zero_to_one_open(*context.mcg_state);
}

random_get_zero_to_one_new :: inline () -> float64 {
    return random_get_zero_to_one_new(*context.mcg_state);
}

random_get_zero_to_one_open_new :: inline () -> float64 {
    return random_get_zero_to_one_open_new(*context.mcg_state);
}

random_get_within_range :: inline (state: *Random_State, min: float, max: float) -> float {
    value := min + (max - min) * random_get_zero_to_one(state);
    low := min;
    high := max;

    if low > high swap(*low, *high);

    return clamp(value, low, high);
}

random_get_within_range :: inline (min: float, max: float) -> float {
    return random_get_within_range(*context.mcg_state, min, max);
}

#scope_file
#import "Basic";

#add_context mcg_state: Random_State = .{ 1, 0 };
