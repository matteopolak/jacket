// The entities live in one `Soa`: an array per field, so the update loop streams through just the
// positions and velocities and can take four of them at a time with SIMD. Two jobs work on it
// every frame: `update_job` moves the entities, and `cull_job` turns each one into a model matrix
// (or marks it as outside the camera's view).

MAX_ENTITIES :: 1024;
ARENA_HALF :: 34.0;
ARENA_LOW :: 0.8;
ARENA_HIGH :: 16.0;
LOD_DISTANCE :: 26.0;
CHUNK :: 64; // entities per piece of a job: a multiple of the SIMD width

Entity :: struct {
    x: float;
    y: float;
    z: float;
    vx: float;
    vy: float;
    vz: float;
    yaw: float;
    pitch: float;
    spin_yaw: float;
    spin_pitch: float;
    scale: float;
    kind: s32;
    r: float;
    g: float;
    b: float;
}

World :: struct {
    entities: Soa(Entity, MAX_ENTITIES);
    count: s64; // a multiple of 4
    time: float;
    ticks: s64;
}

// What the GPU reads for every drawn entity: the model matrix, column by column, and a colour
// whose alpha says "checkered" (used by the floor).
Instance :: struct {
    model: [16] float;
    color: [4] float;
}

NOT_VISIBLE :: 255;

// What `cull_job` fills in. `lods[i]` is the level of detail of entity `i` or NOT_VISIBLE.
Cull_Job :: struct {
    world: *World;
    meshes: *Mesh_Set;
    planes: [6] Vector4;
    eye: Vector3;
    instances: *Instance;
    lods: *u8;
}

Update_Job :: struct {
    world: *World;
    dt: float;
}

world_init :: (world: *World, count: s64, seed: u32) {
    assert(count % 4 == 0 && count <= MAX_ENTITIES);

    world.count = count;
    world.time = 0;
    world.ticks = 0;

    rng: u64;

    random_seed(*rng, seed);

    for i: 0..count - 1 {
        e := *world.entities;
        speed := random_get_within_range(*rng, 1.5, 6.0);
        heading := random_get_within_range(*rng, 0, TAU);
        tilt := random_get_within_range(*rng, -0.4, 0.4);

        e.x[i] = random_get_within_range(*rng, -ARENA_HALF, ARENA_HALF);
        e.y[i] = random_get_within_range(*rng, ARENA_LOW, ARENA_HIGH);
        e.z[i] = random_get_within_range(*rng, -ARENA_HALF, ARENA_HALF);
        e.vx[i] = cos(heading) * speed;
        e.vy[i] = tilt * speed;
        e.vz[i] = sin(heading) * speed;
        e.yaw[i] = random_get_within_range(*rng, 0, TAU);
        e.pitch[i] = random_get_within_range(*rng, 0, TAU);
        e.spin_yaw[i] = random_get_within_range(*rng, -2, 2);
        e.spin_pitch[i] = random_get_within_range(*rng, -2, 2);
        e.scale[i] = random_get_within_range(*rng, 0.6, 1.5);
        e.kind[i] = cast(s32) (i % 3);
        e.r[i] = random_get_within_range(*rng, 0.25, 1.0);
        e.g[i] = random_get_within_range(*rng, 0.25, 1.0);
        e.b[i] = random_get_within_range(*rng, 0.25, 1.0);
    }
}

// Moves entities `first..first + count - 1`. Positions and velocities are processed four lanes at a
// time; entities bounce off the walls of the arena.
update_job :: (data: *void, first: s64, count: s64) {
    job := cast(*Update_Job) data;
    e := *job.world.entities;

    bounce(e.x.data, e.vx.data, first, count, job.dt, -ARENA_HALF, ARENA_HALF);
    bounce(e.y.data, e.vy.data, first, count, job.dt, ARENA_LOW, ARENA_HIGH);
    bounce(e.z.data, e.vz.data, first, count, job.dt, -ARENA_HALF, ARENA_HALF);

    for i: first..first + count - 1 {
        e.yaw[i] += e.spin_yaw[i] * job.dt;
        e.pitch[i] += e.spin_pitch[i] * job.dt;
    }
}

// One tick of the world: all entities, spread over the workers.
world_update :: (world: *World, jobs: *Job_System, dt: float) {
    job := Update_Job.{ world = world, dt = dt };

    run_job(jobs, update_job, *job, world.count, CHUNK);

    world.time += dt;
    world.ticks += 1;
}

// For each entity: is its bounding sphere inside the view, how far away is it, and where does it
// stand. Fills `instances[i]` and `lods[i]`.
cull_job :: (data: *void, first: s64, count: s64) {
    job := cast(*Cull_Job) data;
    e := *job.world.entities;

    for i: first..first + count - 1 {
        kind := e.kind[i];
        center := Vector3.{ e.x[i], e.y[i], e.z[i] };
        radius := job.meshes.radius[kind] * e.scale[i];

        if !sphere_in_view(job.planes, center, radius) {
            job.lods[i] = NOT_VISIBLE;

            continue;
        }

        job.lods[i] = ifx distance(center, job.eye) > LOD_DISTANCE then cast(u8) 1 else cast(u8) 0;

        rotation := rotation_y(e.yaw[i]) * rotation_x(e.pitch[i]);
        model := scale(rotation, e.scale[i]);

        model._14 = center.x;
        model._24 = center.y;
        model._34 = center.z;
        job.instances[i] = make_instance(model, .{ e.r[i], e.g[i], e.b[i] }, 0);
    }
}

// The model matrix as the GPU wants it (column-major), with a colour.
make_instance :: (model: Matrix4, color: Vector3, checkered: float) -> Instance {
    instance: Instance;

    instance.model = transpose(model).floats;
    instance.color = .[color.x, color.y, color.z, checkered];

    return instance;
}

// A hash of every entity's position, rotation and velocity, to compare runs.
world_checksum :: (world: *World) -> u64 {
    hash: u64 = 14695981039346656037;
    e := *world.entities;

    for i: 0..world.count - 1 {
        values := float.[e.x[i], e.y[i], e.z[i], e.vx[i], e.vy[i], e.vz[i], e.yaw[i], e.pitch[i]];

        for value: values {
            bits := (cast(*u32) *value).*;

            hash = (hash ^ bits) * 1099511628211;
        }
    }

    return hash;
}

rotation_y :: (angle: float) -> Matrix4 {
    c, s := cos(angle), sin(angle);

    return Matrix4.{ c, 0, s, 0, 0, 1, 0, 0, -s, 0, c, 0, 0, 0, 0, 1 };
}

rotation_x :: (angle: float) -> Matrix4 {
    c, s := cos(angle), sin(angle);

    return Matrix4.{ 1, 0, 0, 0, 0, c, -s, 0, 0, s, c, 0, 0, 0, 0, 1 };
}

// The six planes of the view frustum, from the combined view and projection matrix (Gribb and
// Hartmann). Each plane's normal points into the view; clip depth runs from 0 to 1.
frustum_planes :: (view_projection: Matrix4) -> [6] Vector4 {
    m := view_projection;
    planes: [6] Vector4;

    planes[0] = m.v[3] + m.v[0];
    planes[1] = m.v[3] - m.v[0];
    planes[2] = m.v[3] + m.v[1];
    planes[3] = m.v[3] - m.v[1];
    planes[4] = m.v[2];
    planes[5] = m.v[3] - m.v[2];

    for *plane: planes {
        inverse_length := 1.0 / length(plane.xyz);

        plane.* = plane.* * inverse_length;
    }

    return planes;
}

sphere_in_view :: (planes: [6] Vector4, center: Vector3, radius: float) -> bool {
    for plane: planes {
        if dot(plane.xyz, center) + plane.w < -radius return false;
    }

    return true;
}

#scope_file

// `pos += vel * dt` for `count` entities starting at `first`, four at a time, reversing the
// velocity of any lane that went past `low` or `high`.
bounce :: (position: *float, velocity: *float, first: s64, count: s64, dt: float, low: float, high: float) {
    step := SIMD.splat(SIMD.f32x4, dt);
    floor := SIMD.splat(SIMD.f32x4, low);
    ceiling := SIMD.splat(SIMD.f32x4, high);
    zero := SIMD.splat(SIMD.f32x4, 0);
    i := first;

    while i < first + count {
        p := SIMD.load(SIMD.f32x4, position + i);
        v := SIMD.load(SIMD.f32x4, velocity + i);

        p = p + v * step;

        away := zero - SIMD.abs(v);

        v = SIMD.select(SIMD.gt(p, ceiling), away, v);
        v = SIMD.select(SIMD.lt(p, floor), SIMD.abs(v), v);
        p = SIMD.min(SIMD.max(p, floor), ceiling);

        SIMD.store(p, position + i);
        SIMD.store(v, velocity + i);

        i += 4;
    }
}
