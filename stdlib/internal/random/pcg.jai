random_get :: (state: *u64) -> u32 #no_aoc {
    old := state.*;

    state.* = old * 6364136223846793005 + 1442695040888963407;
    value := cast,truncate(u32) (((old >> 18) ^ old) >> 27);
    rotation := cast(u32) (old >> 59);

    return (value >> rotation) | (value << ((0 - rotation) & 31));
}

random_get_within_bound :: (state: *u64, range: u32) -> u32 #no_aoc {
    assert(range != 0);
    threshold := (0 - range) % range;
    value := random_get(state);

    while value < threshold value = random_get(state);

    return value % range;
}

random_get_within_bound2 :: (state: *u64, range: u32) -> u32 {
    return random_get_within_bound(state, range);
}

random_seed :: (state: *u64, new_seed: u32) #no_aoc {
    state.* = cast(u64) new_seed * 6364136223846793005 + 1442695040888963407;
}

random_get_zero_to_one :: (state: *u64) -> float {
    return cast(float) random_get(state) / 4294967295.0;
}

random_get_within_range :: (state: *u64, min: float, max: float) -> float {
    return min + (max - min) * random_get_zero_to_one(state);
}

random_get :: inline () -> u32 {
    return random_get(*global_state);
}

random_get_within_bound :: inline (bound: u32) -> u32 {
    return random_get_within_bound(*global_state, bound);
}

random_seed :: (new_seed: u32) {
    random_seed(*global_state, new_seed);
}

random_get_zero_to_one :: () -> float {
    return random_get_zero_to_one(*global_state);
}

random_get_within_range :: (min: float, max: float) -> float {
    return random_get_within_range(*global_state, min, max);
}

#scope_file
#import "Basic";

global_state: u64 = 1442695040888963407;
