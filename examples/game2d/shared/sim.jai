// The rules of the game that the server can reason about: what an enemy is worth and how many
// arrive in a wave. The game uses the same table to score kills.

Enemy_Kind :: enum u8 {
    DRONE;
    CHASER;
    TANK;
}

ENEMY_POINTS :: s64.[10, 25, 50];
MAX_POINTS_PER_KILL :: 50;

// A wave of `wave` (counting from 1) has this many enemies; the level seed decides which kinds.
wave_size :: (wave: s64) -> s64 {
    return 4 + 2 * wave;
}

// The first wave has only drones, then chasers join, then tanks.
kind_for_wave :: (wave: s64, roll: u32) -> Enemy_Kind {
    if wave >= 5 && roll % 5 == 0 return .TANK;
    if wave >= 2 && roll % 3 == 0 return .CHASER;

    return .DRONE;
}
