// The simulation: a fixed-step top-down arena. It knows nothing about windows, files or sound, so
// the smoke test can run it headless with scripted input and check that two runs agree.

VIEW_W :: 320;
VIEW_H :: 180;
TICK_RATE :: 60;
TICK_SECONDS :: 1.0 / 60.0;

PLAYER_SPEED :: 110.0;
PLAYER_RADIUS :: 5.0;
SHOT_SPEED :: 260.0;
SHOT_COOLDOWN_TICKS :: 8;
START_LIVES :: 3;
INVULNERABLE_SECONDS :: 2.0;

Controls :: struct {
    move: Vector2;
    aim: Vector2; // zero: face where the ship moves
    fire: bool;
    confirm: bool;
    pause: bool;
    back: bool;
}

Enemy :: struct {
    kind: Enemy_Kind;
    pos: Vector2;
    vel: Vector2;
    hp: s32;
    cooldown: float;
    age: float;
}

Bullet :: struct {
    pos: Vector2;
    vel: Vector2;
    from_player: bool;
    life: float;
}

Particle :: struct {
    pos: Vector2;
    vel: Vector2;
    life: float;
    max_life: float;
    size: float;
    color: Vector4;
    frame_cell: Cell;
}

Pickup :: struct {
    pos: Vector2;
    life: float;
}

Player :: struct {
    pos: Vector2;
    vel: Vector2;
    angle: float; // radians, 0 points up
    cooldown: s32;
    invulnerable: float;
}

World_Event :: enum u8 {
    SHOT;
    HIT;
    EXPLODE;
    PICKUP;
    PLAYER_HIT;
    WAVE;
    GAME_OVER;
}

World_State :: enum u8 {
    PLAYING;
    OVER;
}

World :: struct {
    seed: s64;
    rng: u64;
    tick: s64;
    state: World_State;

    player: Player;
    enemies: Bucket_Array(Enemy, 32);
    bullets: Bucket_Array(Bullet, 64);
    particles: Bucket_Array(Particle, 128);
    pickups: Bucket_Array(Pickup, 8);

    wave: s64;
    to_spawn: s64;
    spawn_timer: float;
    score: s64;
    kills: s64;
    lives: s64;
    shake: float;

    // Everything a wave plans lives in this pool and is dropped with it.
    wave_memory: Pool;
    wave_plan: [..] Enemy_Kind;

    events: [..] World_Event;
}

world_init :: (world: *World, seed: s64) {
    world_free(world);

    world.* = .{};
    world.seed = seed;
    random_seed(*world.rng, cast(u32) seed);
    world.player.pos = .{ VIEW_W / 2, VIEW_H - 30 };
    world.lives = START_LIVES;

    start_wave(world, 1);
}

world_free :: (world: *World) {
    bucket_array_reset(*world.enemies);
    bucket_array_reset(*world.bullets);
    bucket_array_reset(*world.particles);
    bucket_array_reset(*world.pickups);
    release(*world.wave_memory);
    array_free(world.events);
}

duration_ms :: (world: *World) -> s64 {
    return world.tick * 1000 / TICK_RATE;
}

random_unit :: (world: *World) -> float {
    return random_get_zero_to_one(*world.rng);
}

random_range :: (world: *World, low: float, high: float) -> float {
    return random_get_within_range(*world.rng, low, high);
}

start_wave :: (world: *World, wave: s64) {
    world.wave = wave;

    // The previous wave's plan is dropped all at once.
    reset(*world.wave_memory);

    plan: [..] Enemy_Kind;

    plan.allocator = .{ pool_allocator_proc, *world.wave_memory };

    for 1..wave_size(wave) array_add(*plan, kind_for_wave(wave, random_get(*world.rng)));

    world.wave_plan = plan;
    world.to_spawn = plan.count;
    world.spawn_timer = 0.8;

    array_add(*world.events, .WAVE);
}

spawn_enemy :: (world: *World, kind: Enemy_Kind) {
    enemy: Enemy;

    enemy.kind = kind;
    enemy.pos = .{ random_range(world, 16, VIEW_W - 16), -10 };

    if #complete kind == {
        case .DRONE;
            enemy.hp = 1;
            enemy.vel = .{ 0, 30 + 4 * cast(float) world.wave };
        case .CHASER;
            enemy.hp = 1;
            enemy.vel = .{ 0, 20 };
        case .TANK;
            enemy.hp = 4;
            enemy.vel = .{ 0, 14 };
            enemy.cooldown = 1.5;
    }

    bucket_array_add(*world.enemies, enemy);
}

add_burst :: (world: *World, pos: Vector2, count: s64, speed: float, color: Vector4) {
    for 1..count {
        angle := random_range(world, 0, 2 * PI);
        magnitude := random_range(world, 0.3, 1) * speed;
        particle: Particle;

        particle.pos = pos;
        particle.vel = .{ cos(angle) * magnitude, sin(angle) * magnitude };
        particle.max_life = random_range(world, 0.25, 0.6);
        particle.life = particle.max_life;
        particle.size = random_range(world, 3, 7);
        particle.color = color;
        particle.frame_cell = .DOT;

        bucket_array_add(*world.particles, particle);
    }
}

add_explosion :: (world: *World, pos: Vector2) {
    particle: Particle;

    particle.pos = pos;
    particle.life = 0.3;
    particle.max_life = 0.3;
    particle.size = 26;
    particle.color = .{ 1, 1, 1, 1 };
    particle.frame_cell = .EXPLOSION_0;

    bucket_array_add(*world.particles, particle);
    add_burst(world, pos, 8, 90, .{ 1, 0.7, 0.3, 1 });
}

fire_bullet :: (world: *World, pos: Vector2, velocity: Vector2, from_player: bool) {
    bullet := Bullet.{ pos, velocity, from_player, 2.5 };

    bucket_array_add(*world.bullets, bullet);
}

world_step :: (world: *World, controls: Controls) {
    if world.state == .OVER return;

    world.tick += 1;
    world.shake = max(0.0, world.shake - 0.6);

    step_player(world, controls);
    step_spawning(world);
    step_enemies(world);
    step_bullets(world);
    step_particles(world);
    step_pickups(world);

    if world.to_spawn == 0 && world.enemies.count == 0 start_wave(world, world.wave + 1);

    if world.lives <= 0 {
        world.state = .OVER;
        array_add(*world.events, .GAME_OVER);
    }
}

step_player :: (world: *World, controls: Controls) {
    player := *world.player;
    move := controls.move;

    if length(move) > 1 move = normalize(move);

    player.vel = move * PLAYER_SPEED;
    player.pos += player.vel * TICK_SECONDS;
    player.pos.x = clamp(player.pos.x, 8.0, VIEW_W - 8.0);
    player.pos.y = clamp(player.pos.y, 8.0, VIEW_H - 8.0);
    player.invulnerable = max(0.0, player.invulnerable - TICK_SECONDS);

    facing := controls.aim;

    if length(facing) < 0.2 facing = move;

    if length(facing) > 0.2 player.angle = atan2(facing.x, -facing.y);

    if player.cooldown > 0 player.cooldown -= 1;

    if controls.fire && player.cooldown == 0 {
        direction := Vector2.{ sin(player.angle), -cos(player.angle) };

        fire_bullet(world, player.pos + direction * 8, direction * SHOT_SPEED, true);
        player.cooldown = SHOT_COOLDOWN_TICKS;
        array_add(*world.events, .SHOT);
    }
}

step_spawning :: (world: *World) {
    if world.to_spawn == 0 return;

    world.spawn_timer -= TICK_SECONDS;

    if world.spawn_timer > 0 return;

    index := world.wave_plan.count - world.to_spawn;

    spawn_enemy(world, world.wave_plan[index]);
    world.to_spawn -= 1;
    world.spawn_timer = max(0.25, 0.9 - 0.05 * cast(float) world.wave);
}

step_enemies :: (world: *World) {
    player := *world.player;

    for *enemy: world.enemies {
        enemy.age += TICK_SECONDS;

        if #complete enemy.kind == {
            case .DRONE;
                enemy.pos.x += sin(enemy.age * 3 + enemy.pos.y * 0.05) * 40 * TICK_SECONDS;
                enemy.pos.y += enemy.vel.y * TICK_SECONDS;
            case .CHASER;
                to_player := player.pos - enemy.pos;

                if length(to_player) > 1 {
                    enemy.vel = enemy.vel * 0.96 + normalize(to_player) * (60 + 3 * cast(float) world.wave) * 0.04;
                }

                enemy.pos += enemy.vel * TICK_SECONDS;
            case .TANK;
                enemy.pos += enemy.vel * TICK_SECONDS;
                enemy.cooldown -= TICK_SECONDS;

                if enemy.cooldown <= 0 && enemy.pos.y > 0 {
                    aim := player.pos - enemy.pos;

                    if length(aim) > 1 fire_bullet(world, enemy.pos, normalize(aim) * 70, false);

                    enemy.cooldown = 2.0;
                }
        }

        if enemy.pos.y > VIEW_H + 20 || enemy.pos.x < -30 || enemy.pos.x > VIEW_W + 30 {
            // An enemy that gets past is not a kill; it comes back on top.
            enemy.pos.y = -10;
        }

        if player.invulnerable <= 0 && distance_squared(enemy.pos, player.pos) < 10 * 10 {
            hurt_player(world);
            kill_enemy(world, enemy, false);
            remove;
        }
    }
}

hurt_player :: (world: *World) {
    world.lives -= 1;
    world.player.invulnerable = INVULNERABLE_SECONDS;
    world.shake = 8;
    add_explosion(world, world.player.pos);
    array_add(*world.events, .PLAYER_HIT);
}

// Scores the kill (when `scored`), starts the explosion and maybe drops a shield.
kill_enemy :: (world: *World, enemy: *Enemy, scored: bool) {
    if scored {
        world.score += ENEMY_POINTS[cast(s64) enemy.kind];
        world.kills += 1;

        if random_get_within_bound(*world.rng, 12) == 0 {
            bucket_array_add(*world.pickups, Pickup.{ enemy.pos, 6 });
        }
    }

    add_explosion(world, enemy.pos);
    array_add(*world.events, .EXPLODE);
}

step_bullets :: (world: *World) {
    player := *world.player;

    for *bullet: world.bullets {
        bullet.pos += bullet.vel * TICK_SECONDS;
        bullet.life -= TICK_SECONDS;

        outside := bullet.pos.x < -8 || bullet.pos.x > VIEW_W + 8 || bullet.pos.y < -8 || bullet.pos.y > VIEW_H + 8;

        if bullet.life <= 0 || outside {
            remove;
            continue;
        }

        if bullet.from_player {
            hit := false;

            for *enemy: world.enemies {
                reach := ifx enemy.kind == .TANK then 9.0 else 7.0;

                if distance_squared(enemy.pos, bullet.pos) > reach * reach continue;

                enemy.hp -= 1;
                hit = true;
                array_add(*world.events, .HIT);
                add_burst(world, bullet.pos, 3, 60, .{ 1, 1, 0.6, 1 });

                if enemy.hp <= 0 {
                    kill_enemy(world, enemy, true);
                    remove;
                }

                break;
            }

            if hit remove;
        } else if player.invulnerable <= 0 && distance_squared(bullet.pos, player.pos) < 6 * 6 {
            hurt_player(world);
            remove;
        }
    }
}

step_particles :: (world: *World) {
    for *particle: world.particles {
        particle.life -= TICK_SECONDS;
        particle.pos += particle.vel * TICK_SECONDS;
        particle.vel *= 0.95;

        if particle.life <= 0 remove;
    }
}

step_pickups :: (world: *World) {
    for *pickup: world.pickups {
        pickup.life -= TICK_SECONDS;
        pickup.pos.y += 10 * TICK_SECONDS;

        if distance_squared(pickup.pos, world.player.pos) < 10 * 10 {
            world.player.invulnerable = max(world.player.invulnerable, 3.0);
            array_add(*world.events, .PICKUP);
            remove;
        } else if pickup.life <= 0 {
            remove;
        }
    }
}

// A hash of everything the simulation decides, for comparing two runs.
world_hash :: (world: *World) -> u64 {
    state: XXH64_state_t;

    XXH64_reset(*state, 7);

    numbers := float.[world.player.pos.x, world.player.pos.y, world.player.angle];

    XXH64_update(*state, numbers.data, cast(u32) (numbers.count * size_of(float)));

    scalars := s64.[world.tick, world.score, world.kills, world.lives, world.wave, world.enemies.count, world.bullets.count, world.particles.count];

    XXH64_update(*state, scalars.data, cast(u32) (scalars.count * size_of(s64)));

    for world.enemies {
        XXH64_update(*state, *it.pos, size_of(Vector2));
    }

    return XXH64_digest(*state);
}

match_result :: (world: *World) -> New_Match {
    return .{ seed = world.seed, score = world.score, kills = world.kills, duration_ms = duration_ms(world) };
}
