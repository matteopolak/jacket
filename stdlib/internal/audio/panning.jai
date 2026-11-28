SOUND_INNER_RADIUS_DEFAULT :: 10.0;
SOUND_OUTER_RADIUS_DEFAULT :: 150.0;
VERY_CLOSE :: 4.0;
ROOT_2_OVER_2: float : 0.7071067;

vbap_speaker_directions: [ACHANNEL_MAX] Vector2;

vbap_source_directions: [NUM_SOURCE_DIRECTIONS] Vector2;

// Writes the inverse of the 2x2 matrix `m` to `result`. Stops the program if the matrix is singular.
invert :: (using m: *Matrix2, result: *Matrix2) {
    determinant := cast(float64) _11 * _22 - cast(float64) _12 * _21;

    assert(determinant != 0, "singular audio panning matrix");
    inverse: Matrix2 = .{
        cast(float) (_22 / determinant), cast(float) (-_12 / determinant),
        cast(float) (-_21 / determinant), cast(float) (_11 / determinant)
    };

    result.* = inverse;
}

// Converts a perceptual volume of 0 to 1 into a linear gain with `x` to the power 1 / (2 log10 2), about
// 1.66, so a perceptual 0.5 gives a gain of about 0.32 (-10 dB).
perceptual_to_linear :: (x: float) -> float {
    exponent := cast(float) (1.0 / (2.0 * log(cast(float64) 2) / log(cast(float64) 10)));
    return pow(x, exponent);
}

// Sets the target gain of input channel `index` to zero for every output channel.
clear_scale_accumulation :: (stream: *Sound_Stream, index: s64) {
    assert(index >= 0 && index < MAX_CHANNELS_PER_SOUND);
    for *scale: stream.input_scale_mappings[index].source_scale_for_this_output_index scale.* = 0;
}

// Tells the player where the listener is, so sounds with a position can be panned and softened by distance.
//
// - camera_position: where the listener is.
// - camera_orientation: which way the listener faces.
//
// ```jai,no_run,skip_on(wasm)
// update_listener(.{ 0, 1.7, 0 }, .{ 0, 0, 0, 1 });
// ```
update_listener :: (camera_position: Vector3, camera_orientation: Quaternion) {
    lock_player();
    defer unlock_player();
    listener_position = camera_position;
    listener_orientation = camera_orientation;
    listener_ready = true;
}

// Returns the unit direction of `position` in the listener's horizontal plane: x ahead, y to the left.
//
// Returns (1, 0), straight ahead, when no listener is set or the stream is not `.SPATIALIZED`. A position
// directly above or below the listener has no horizontal direction and gives NaN.
get_plane_dir :: (stream: *Sound_Stream, position: Vector3) -> Vector2 {
    if !listener_ready || !(stream.user_flags & .SPATIALIZED) return .{ 1, 0 };
    local := rotate(position - listener_position, conjugate(listener_orientation));
    magnitude := sqrt(local.x * local.x + local.y * local.y);
    return .{ local.x / magnitude, local.y / magnitude };
}

// Returns how near the stream is to the listener horizontally, from 0 to 1.
//
// It is 1 on top of the listener and falls to 0 at the smaller of `VERY_CLOSE` and three quarters of
// `inner_radius` (world units).
get_target_2_closeness :: (stream: *Sound_Stream) -> float {
    local := rotate(stream.position - listener_position, conjugate(listener_orientation));
    radius := min(VERY_CLOSE, stream.inner_radius * 0.75);
    if radius <= 0 return 0;
    return clamp(1 - sqrt(local.x * local.x + local.y * local.y) / radius, 0.0, 1.0);
}

// Moves both values toward 1 by the fraction `closeness`, which is clamped to 0 to 1.
adjust_stereo_dots_by_closeness :: (dot_left: *float, dot_right: *float, closeness: float) {
    amount := clamp(closeness, 0.0, 1.0);
    dot_left.* += (1 - dot_left.*) * amount;
    dot_right.* += (1 - dot_right.*) * amount;
}

// Adds stereo gains for a source heard from `plane_dir` to outputs 0 (left) and 1 (right) of `target`.
//
// The pair is scaled to unit power, then multiplied by `gain_scale`. A `.SPATIALIZED` stream close to the
// listener spreads toward both speakers.
accum_scale_target_2_source_mono :: (stream: *Sound_Stream, plane_dir: Vector2, gain_scale: float, target: *Scale_Mapping) {
    left := max(plane_dir.x * ROOT_2_OVER_2 + plane_dir.y * ROOT_2_OVER_2, 0.0);
    right := max(plane_dir.x * ROOT_2_OVER_2 - plane_dir.y * ROOT_2_OVER_2, 0.0);

    if stream.user_flags & .SPATIALIZED adjust_stereo_dots_by_closeness(*left, *right, get_target_2_closeness(stream));

    // The shared base keeps uncovered directions audible; its private magnitude
    // is independently selected pending a numeric refinement.
    left += 0.5;
    right += 0.5;
    magnitude := sqrt(left * left + right * right);

    target.source_scale_for_this_output_index[0] += left * gain_scale / magnitude;
    target.source_scale_for_this_output_index[1] += right * gain_scale / magnitude;
}

// Adds stereo gains for input channel `n` of a multichannel stream, turning the direction by that
// channel's source direction. Stops the program if `n` is not a channel of the stream.
accum_scale_target_2_source_n :: (stream: *Sound_Stream, plane_dir: Vector2, n: s64, gain_scale: float) {
    assert(n >= 0 && n < stream.num_channels);
    direction := channel_plane_direction(stream, plane_dir, n);
    accum_scale_target_2_source_mono(stream, direction, gain_scale, *stream.input_scale_mappings[n]);
}

// Adds gains for `plane_dir` to the two speakers of the output arc that fits it best (vector base
// amplitude panning), scaled to unit power then by `gain_scale`.
//
// Does nothing when the output has no arcs (stereo) or the direction matches none.
add_vbap_gain_for_direction :: (target: *Scale_Mapping, plane_dir: Vector2, gain_scale: float) {
    best_score := -FLOAT32_INFINITY;
    selected: *Vbap_Arc;
    selected_a: float;
    selected_b: float;

    for *arc: arcs {
        assert(cast(s64) arc.speaker1 < OUTPUT_CHANNELS_MAX && cast(s64) arc.speaker2 < OUTPUT_CHANNELS_MAX);
        first := vbap_speaker_directions[cast(s64) arc.speaker1];
        second := vbap_speaker_directions[cast(s64) arc.speaker2];
        matrix: Matrix2 = .{ first.x, second.x, first.y, second.y };

        invert(*matrix, *arc.l_inverse);
        a := arc.l_inverse._11 * plane_dir.x + arc.l_inverse._12 * plane_dir.y;
        b := arc.l_inverse._21 * plane_dir.x + arc.l_inverse._22 * plane_dir.y;
        score := min(a, b);

        if !selected || score > best_score {
            best_score = score;
            selected = arc;
            selected_a = a;
            selected_b = b;
        }
    }

    if !selected return;

    a := max(selected_a, 0.0);
    b := max(selected_b, 0.0);
    power := sqrt(a * a + b * b);

    if power == 0 return;

    target.source_scale_for_this_output_index[cast(s64) selected.speaker1] += gain_scale * a / power;
    target.source_scale_for_this_output_index[cast(s64) selected.speaker2] += gain_scale * b / power;
}

// Returns whether `stream` keeps `index` silent: footsteps do not play from the rear speakers.
skip_this_output_channel :: (index: Audio_Channel, stream: *Sound_Stream) -> bool {
    return (stream.user_flags & .IS_FOOTSTEP) != 0 && (index == .REAR_LEFT || index == .REAR_RIGHT);
}

// Adds gains for a mono source heard from `plane_dir` to every output channel of `target`.
//
// Stereo output uses [accum_scale_target_2_source_mono]. Otherwise the direction is panned between the
// nearest speakers, and a stream near the listener spreads to all speakers but the subwoofer. The result
// is scaled to unit power, then by `gain_scale`.
accum_scale_target_all_source_mono :: (stream: *Sound_Stream, plane_dir: Vector2, gain_scale: float, target: *Scale_Mapping) {
    if output_channel_count() == 2 {
        accum_scale_target_2_source_mono(stream, plane_dir, gain_scale, target);
        return;
    }

    coefficients: Scale_Mapping;

    add_vbap_gain_for_direction(*coefficients, plane_dir, 1);
    closeness := get_target_2_closeness(stream);
    power: float;

    for output: 0..output_channel_count() - 1 {
        value := coefficients.source_scale_for_this_output_index[output];

        if output != cast(s64) Audio_Channel.SUBWOOFER {
            value *= 1 - closeness;
            if !skip_this_output_channel(cast(Audio_Channel) output, stream) value += closeness;
        }

        coefficients.source_scale_for_this_output_index[output] = value;
        power += value * value;
    }

    if power == 0 return;

    power = sqrt(power);
    for output: 0..output_channel_count() - 1
        target.source_scale_for_this_output_index[output] += coefficients.source_scale_for_this_output_index[output] * gain_scale / power;
}

// Adds gains for input channel `n` of a multichannel stream to every output channel, turning the
// direction by that channel's source direction. Stops the program if `n` is not a channel of the stream.
accum_scale_target_all_source_n :: (stream: *Sound_Stream, plane_dir: Vector2, n: s64, gain_scale: float) {
    assert(n >= 0 && n < stream.num_channels);
    direction := channel_plane_direction(stream, plane_dir, n);
    accum_scale_target_all_source_mono(stream, direction, gain_scale, *stream.input_scale_mappings[n]);
}

// Returns the output speaker that plays a source direction.
//
// `.EXTEND_LEFT` and `.EXTEND_RIGHT` use the extra side speakers only on an 8-channel output, and the rear
// speakers otherwise.
source_dir_to_target_dir :: (source_dir_index: Source_Direction) -> Audio_Channel, success := true {
    if source_dir_index == {
        case .MONO; return .CENTER, true;
        case .LEFT; return .FRONT_LEFT, true;
        case .FRONT_LEFT; return .FRONT_LEFT, true;
        case .RIGHT; return .FRONT_RIGHT, true;
        case .FRONT_RIGHT; return .FRONT_RIGHT, true;
        case .REAR_LEFT; return .REAR_LEFT, true;
        case .REAR_RIGHT; return .REAR_RIGHT, true;
        case .CENTER_FORWARD; return .CENTER, true;
        case .SUBWOOFER_CENTER_FORWARD; return .SUBWOOFER, true;
        case .EXTEND_LEFT;
            if output_channel_count() == 8 return .EXTEND_LEFT, true;

            return .REAR_LEFT, true;
        case .EXTEND_RIGHT;
            if output_channel_count() == 8 return .EXTEND_RIGHT, true;

            return .REAR_RIGHT, true;
    }

    assert(false, "unsupported source direction");

    return .CENTER, false;
}

// Adds gains for a music stream of one or two channels on an output of more than two: front speakers,
// centre and subwoofer carry it, and the other speakers get half of `gain_scale`.
//
// A stereo stream's channels stay on their own side. Stereo output uses
// [update_panning_helper_for_locked_output].
update_panning_helper_for_many_channel_music :: (stream: *Sound_Stream, gain_scale: float) {
    assert(stream.num_channels == 1 || stream.num_channels == 2);
    channels := output_channel_count();
    if channels == 2 {
        update_panning_helper_for_locked_output(stream, gain_scale);
        return;
    }

    for input: 0..stream.num_channels - 1 {
        mapping := *stream.input_scale_mappings[input];
        front := gain_scale;
        center := 0.7 * gain_scale;

        if stream.num_channels == 1 {
            front = 0.8 * gain_scale;
            center = gain_scale;
        }

        mapping.source_scale_for_this_output_index[2] += center;
        mapping.source_scale_for_this_output_index[3] += center;

        for output: 0..channels - 1 {
            if output == 2 || output == 3 continue;

            left := output % 2 == 0;
            if stream.num_channels == 2 && ((input == 0) != left) continue;

            amount := 0.5 * gain_scale;
            if output < 2 amount = front;

            mapping.source_scale_for_this_output_index[output] += amount;
        }
    }
}

// Adds gains that do not depend on the listener: each input channel goes to the speaker its source
// direction names.
//
// On stereo output a mono stream is split evenly, a stereo stream passes through, and other layouts go
// to the side their source direction points to.
update_panning_helper_for_locked_output :: (stream: *Sound_Stream, gain_scale: float) {
    channels := output_channel_count();

    for input: 0..stream.num_channels - 1 {
        mapping := *stream.input_scale_mappings[input];

        if channels == 2 {
            if stream.num_channels == 1 {
                mapping.source_scale_for_this_output_index[0] += 0.5 * gain_scale;
                mapping.source_scale_for_this_output_index[1] += 0.5 * gain_scale;
            } else if stream.num_channels == 2 {
                mapping.source_scale_for_this_output_index[input] += gain_scale;
            } else {
                direction := vbap_source_directions[cast(s64) stream.sound_data.source_directions[input]];
                output := 0;
                if direction.y < 0 output = 1;
                mapping.source_scale_for_this_output_index[output] += gain_scale;
            }
        } else {
            channel, success := source_dir_to_target_dir(stream.sound_data.source_directions[input]);
            if success mapping.source_scale_for_this_output_index[cast(s64) channel] += gain_scale;
        }
    }
}

// Adds the target gains of `stream`, scaled by `gain_scale`, choosing how it is panned.
//
// Music on more than two speakers, and streams without a position (no listener, not `.SPATIALIZED`,
// `.AMBIENT`, `.MUSIC` or the `.UI` category) get fixed gains. Others are panned by where they are
// relative to the listener, which is stored in `stream.last_plane_dir`.
update_panning_helper :: (stream: *Sound_Stream, gain_scale: float) {
    if (stream.user_flags & .MUSIC) && output_channel_count() > 2 {
        update_panning_helper_for_many_channel_music(stream, gain_scale);
        return;
    }

    locked := !listener_ready || !(stream.user_flags & .SPATIALIZED)
        || (stream.user_flags & .AMBIENT) != 0 || (stream.user_flags & .MUSIC) != 0 || stream.category == .UI;
    if locked {
        update_panning_helper_for_locked_output(stream, gain_scale);
        return;
    }

    plane := get_plane_dir(stream, stream.position);

    stream.last_plane_dir = plane;
    if stream.num_channels == 1 {
        accum_scale_target_all_source_mono(stream, plane, gain_scale, *stream.input_scale_mappings[0]);
    } else for input: 0..stream.num_channels - 1 {
        if output_channel_count() == 2 accum_scale_target_2_source_n(stream, plane, input, gain_scale);
        else accum_scale_target_all_source_n(stream, plane, input, gain_scale);
    }
}

// The loudness a stream should have for the listener's position, from the master volume, the stream's own scales and distance.
get_desired_volume_perceptual :: (stream: *Sound_Stream, position: Vector3) -> float {
    if !stream || !stream.sound_data return 0;

    volume := master_volume * stream.universe_volume_scale * stream.user_volume_scale * stream.sound_data.volume_scale;

    if stream.user_flags & .SPATIALIZED {
        distance := length(stream.position - position);
        if distance >= stream.outer_radius return 0;
        if distance > stream.inner_radius && stream.outer_radius > stream.inner_radius
            volume *= (stream.outer_radius - distance) / (stream.outer_radius - stream.inner_radius);
    }

    return volume;
}

// Sets the volume of everything the player plays, as a factor of the original: 1 is unchanged, 0 is silent.
set_master_volume :: (value: float) {
    lock_player();
    defer unlock_player();
    master_volume = value;
}

// Recomputes `stream`'s loudness and per-channel gains for the listener's current position.
//
// Sets `max_gain`, `inaudible` and `debug_display_volume`, rebuilds the target gains, and sets how fast
// each current gain ramps toward its target: the whole range in 1/30 s, a gain being a factor of 0 to 1.
// The first update after the stream starts jumps to the target. Needs an initialized backend.
update_panning :: (stream: *Sound_Stream) {
    lock_player();
    defer unlock_player();
    for i: 0..MAX_CHANNELS_PER_SOUND - 1 clear_scale_accumulation(stream, i);

    volume := get_desired_volume_perceptual(stream, listener_position);

    stream.debug_display_volume = volume;
    stream.max_gain = perceptual_to_linear(volume);
    stream.inaudible = volume <= 0;
    update_panning_helper(stream, stream.max_gain);
    first := (stream.internal_flags & .FIRST_VOLUME_UPDATE) != 0;

    rate := output_sampling_rate();

    assert(rate > 0, "audio output sampling rate requires backend initialization");
    maximum_step := 1.0 / (30.0 * cast(float) rate);

    for *mapping: stream.input_scale_mappings {
        for output: 0..OUTPUT_CHANNELS_MAX - 1 {
            desired := mapping.source_scale_for_this_output_index[output];
            if first mapping.interpolated_source_scale_for_this_output_index[output] = desired;
            mapping.dvolume_dsample[output] = clamp(desired - mapping.interpolated_source_scale_for_this_output_index[output], -maximum_step, maximum_step);
        }
    }

    stream.internal_flags &= ~ .FIRST_VOLUME_UPDATE;
}

#scope_module
initialize_panning_vectors :: () {
    vbap_speaker_directions = .[
    .{ ROOT_2_OVER_2, ROOT_2_OVER_2 }, .{ ROOT_2_OVER_2, -ROOT_2_OVER_2 },
    .{ 1, 0 }, .{ 0, 0 }, .{ -ROOT_2_OVER_2, ROOT_2_OVER_2 },
    .{ -ROOT_2_OVER_2, -ROOT_2_OVER_2 }, .{ -0.9659258, 0.2588190 },
    .{ -0.9659258, -0.2588190 }
];
    vbap_source_directions = .[
    .{ 1, 0 }, .{ 0, 1 }, .{ 0, -1 },
    .{ ROOT_2_OVER_2, ROOT_2_OVER_2 }, .{ ROOT_2_OVER_2, -ROOT_2_OVER_2 },
    .{ -ROOT_2_OVER_2, ROOT_2_OVER_2 }, .{ -ROOT_2_OVER_2, -ROOT_2_OVER_2 },
    .{ 1, 0 }, .{ 1, 0 }, .{ -0.9659258, 0.2588190 }, .{ -0.9659258, -0.2588190 }
];
}

#scope_file
channel_plane_direction :: (stream: *Sound_Stream, plane: Vector2, input: s64) -> Vector2 {
    source := vbap_source_directions[cast(s64) stream.sound_data.source_directions[input]];
    return .{ plane.x * source.x - plane.y * source.y, plane.y * source.x + plane.x * source.y };
}

listener_ready := false;
listener_position: Vector3;
listener_orientation: Quaternion;
master_volume := 1.0;

#import "Basic";
#import "Math";
