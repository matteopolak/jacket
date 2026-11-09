// Check core references, layouts and hierarchy. Example: error := gltf_validate(*asset);
gltf_validate :: (asset: *Gltf_Asset, options := Gltf_Options.{}) -> Gltf_Error {
    for validate: ((asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error).[validate_storage, validate_layout, validate_hierarchy, validate_meshes, validate_resources, validate_animations, validate_cameras] {
        e := validate(asset, options);
        if e.kind != .NONE return e;
    }

    return .{};
}

#scope_module
validate_storage :: (asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error {
    d := *asset.document;
    if d.asset.version != "2.0" || (d.asset.min_version.count && d.asset.min_version != "2.0") return failure(.INVALID, "unsupported asset version", "/asset/version");
    if options.max_count < 0 || options.max_buffer_bytes < 0 return failure(.LIMIT, "negative limits", "/");

    counts := s64.[d.scenes.count, d.nodes.count, d.meshes.count, d.accessors.count, d.buffer_views.count, d.buffers.count, d.materials.count, d.textures.count, d.images.count, d.samplers.count, d.skins.count, d.animations.count, d.cameras.count];
    total: s64;

    for counts {
        if it > options.max_count - total return failure(.LIMIT, "object count exceeded", "/");
        total += it;
    }

    if asset.buffers.count != d.buffers.count return failure(.INVALID, "loaded buffer count mismatch", "/buffers");

    total = 0;

    for b, i: d.buffers {
        if !b.byte_length || asset.buffers[i].count < b.byte_length return failure(.INVALID, "short buffer", tprint("/buffers/%/byteLength", i));
        if asset.buffers[i].count > options.max_buffer_bytes - total return failure(.LIMIT, "buffer budget exceeded", "/buffers");
        total += asset.buffers[i].count;
    }

    if !valid_ref(d.scene, d.scenes.count) return failure(.INVALID, "scene index out of range", "/scene");

    for v, i: d.buffer_views {
        p := tprint("/bufferViews/%", i);
        if v.buffer >= d.buffers.count return failure(.INVALID, "buffer index out of range", tprint("%/buffer", p));
        if !v.byte_length || cast(u64) v.byte_offset + v.byte_length > d.buffers[v.buffer].byte_length return failure(.INVALID, "buffer view exceeds buffer", p);

        stride := reference(v.byte_stride);
        if stride != -1 && (stride < 4 || stride > 252 || stride > v.byte_length || stride % 4 != 0 || target_is_index(v.target)) return failure(.INVALID, "invalid byte stride", tprint("%/byteStride", p));

        if v.target.state == .SET && !known(v.target.value) return failure(.INVALID, "invalid target", tprint("%/target", p));
    }

    return .{};
}

validate_layout :: (asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error {
    d := *asset.document;
    for a, i: d.accessors {
        p := tprint("/accessors/%", i);
        size, rows, cols := layout(a);

        if !size || !rows || !a.count || a.count > options.max_count return failure(.INVALID, "invalid accessor type or count", p);
        if a.normalized && (a.component_type == .UNSIGNED_INT || a.component_type == .FLOAT) return failure(.INVALID, "component cannot be normalized", tprint("%/normalized", p));
        if !valid_ref(a.buffer_view, d.buffer_views.count) return failure(.INVALID, "buffer view index out of range", tprint("%/bufferView", p));
        if a.buffer_view.state == .NULL return failure(.INVALID, "null buffer view", p);

        if a.buffer_view.state == .SET {
            e := region(asset, a.buffer_view.value, a.byte_offset, a.count, size, rows, cols, false, p);
            if e.kind != .NONE return e;
        } else if a.byte_offset != 0 return failure(.INVALID, "offset without buffer view", p);

        for bounds: [] float64.[a.min, a.max] {
            if bounds.count && bounds.count != rows * cols return failure(.INVALID, "bounds have wrong component count", p);
        }

        if a.min.count && a.max.count for j: 0..a.min.count - 1 if a.min[j] > a.max[j] return failure(.INVALID, "min exceeds max", p);

        if a.sparse {
            s := a.sparse;
            if !s.count || s.count > a.count || !unsigned_component(s.indices.component_type) return failure(.INVALID, "invalid sparse count or index type", tprint("%/sparse", p));

            isize := component_size(s.indices.component_type);
            e := region(asset, s.indices.buffer_view, s.indices.byte_offset, s.count, isize, 1, 1, true, tprint("%/sparse/indices", p));

            if e.kind != .NONE return e;

            e = region(asset, s.values.buffer_view, s.values.byte_offset, s.count, size, rows, cols, true, tprint("%/sparse/values", p));
            if e.kind != .NONE return e;

            bytes, start := view_data(asset, s.indices.buffer_view, s.indices.byte_offset);
            previous: s64 = -1;

            for j: 0..cast(s64) s.count - 1 {
                index := word(bytes, start + j * isize, isize);
                if index >= a.count || index <= previous return failure(.INVALID, "sparse indices must increase and fit accessor", tprint("%/sparse/indices", p));
                previous = index;
            }
        }
    }

    for a, i: d.accessors {
        cursor := Gltf_Cursor.{ asset = asset, accessor = i };
        size, rows, cols := layout(a);
        column := rows * size;

        if cols > 1 column = (column + 3) & ~3;

        for j: 0..cast(s64) a.count - 1 {
            bytes, start := next_position(*cursor);

            for c: 0..cols - 1 for r : 0..rows - 1 {
                raw: u32;
                if bytes.data raw = word(bytes, start + c * column + r * size, size);

                value, valid := scalar(raw, a.component_type, false);
                k := c * rows + r;

                if !valid || (a.min.count && value < bound(a.min[k], a.component_type)) || (a.max.count && value > bound(a.max[k], a.component_type)) return failure(.INVALID, "accessor data is non-finite or outside bounds", tprint("/accessors/%", i), start + c * column + r * size);
            }
        }
    }

    return .{};
}

validate_hierarchy :: (asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error {
    d := *asset.document;
    parents: [..] s64;

    array_resize(*parents, d.nodes.count);
    defer array_free(parents);
    for *parents it.* = -1;

    for n, i: d.nodes {
        p := tprint("/nodes/%", i);
        if !valid_ref(n.mesh, d.meshes.count) return failure(.INVALID, "mesh index out of range", tprint("%/mesh", p));
        if !valid_ref(n.skin, d.skins.count) return failure(.INVALID, "skin index out of range", tprint("%/skin", p));
        if !valid_ref(n.camera, d.cameras.count) return failure(.INVALID, "camera index out of range", tprint("%/camera", p));
        if (n.matrix.count && n.matrix.count != 16) || (n.translation.count && n.translation.count != 3) || (n.scale.count && n.scale.count != 3) || (n.rotation.count && n.rotation.count != 4) return failure(.INVALID, "invalid transform shape", p);
        if n.matrix.count && (n.translation.count || n.scale.count || n.rotation.count) return failure(.INVALID, "matrix and TRS are mutually exclusive", p);

        if n.matrix.count && !trs_matrix(n.matrix) return failure(.INVALID, "matrix cannot be decomposed to TRS", tprint("%/matrix", p));

        if n.skin.state == .SET && n.mesh.state != .SET return failure(.INVALID, "skin requires mesh", p);

        if n.skin.state == .SET && n.mesh.state == .SET {
            for prim: d.meshes[n.mesh.value].primitives {
                found, _ := table_find(*prim.attributes, "JOINTS_0");
                if !found return failure(.INVALID, "skinned primitive needs joint attributes", p);
            }
        }

        if n.weights.count && (n.mesh.state != .SET || !d.meshes[n.mesh.value].primitives.count || n.weights.count != d.meshes[n.mesh.value].primitives[0].targets.count) return failure(.INVALID, "node weights require matching morph targets", p);

        if n.rotation.count {
            length: float64;
            for n.rotation length += cast(float64) it * it;
            // Squared bounds correspond to a 0.001 error in vector length.
            if length < 0.998001 || length > 1.002001 return failure(.INVALID, "rotation must be a unit quaternion", p);
        }

        for child, j: n.children {
            if child >= d.nodes.count return failure(.INVALID, "node index out of range", tprint("%/children/%", p, j));
            if parents[child] != -1 return failure(.INVALID, "node has multiple parents", tprint("%/children/%", p, j));
            parents[child] = i;
        }
    }

    // Peel roots iteratively: no recursive call stack controlled by the input.
    queue: [..] s64;

    defer array_free(queue);
    for parents if it == -1 array_add(*queue, it_index);

    head: s64;
    while head < queue.count {
        node := queue[head];
        head += 1;
        for d.nodes[node].children array_add(*queue, cast(s64) it);
    }

    if head != d.nodes.count return failure(.INVALID, "node cycle", "/nodes");

    roots: [..] s64;

    array_resize(*roots, d.nodes.count);
    defer array_free(roots);
    for node: queue roots[node] = ifx parents[node] < 0 then node else roots[parents[node]];

    return validate_skins(asset, parents, roots);
}

validate_skins :: (asset: *Gltf_Asset, parents: [] s64, roots: [] s64) -> Gltf_Error {
    d := *asset.document;
    seen: [..] bool;

    array_resize(*seen, d.nodes.count);
    defer array_free(seen);

    for skin, i: d.skins {
        if !skin.joints.count || !valid_ref(skin.skeleton, d.nodes.count) || !valid_ref(skin.inverse_bind_matrices, d.accessors.count) return failure(.INVALID, "invalid skin reference", tprint("/skins/%", i));
        for joint: skin.joints if joint >= d.nodes.count return failure(.INVALID, "joint index out of range", tprint("/skins/%/joints", i));

        root := roots[skin.joints[0]];

        for joint: skin.joints if roots[joint] != root return failure(.INVALID, "skin joints have no common root", tprint("/skins/%/joints", i));

        duplicate := first_duplicate(skin.joints, seen);
        if duplicate >= 0 return failure(.INVALID, "skin joints must be unique", tprint("/skins/%/joints/%", i, duplicate));

        if skin.skeleton.state == .SET && !skeleton_covers(skin, parents) return failure(.INVALID, "skeleton is not an ancestor of every joint", tprint("/skins/%/skeleton", i));
    }

    return validate_scenes(asset, parents, roots);
}

// The position of the first joint that repeats an earlier one, or -1. `seen` is all false and is left that way.
first_duplicate :: (joints: [] u32, seen: [] bool) -> s64 {
    duplicate := -1;

    for joint, j: joints {
        if seen[joint] {
            duplicate = j;
            break;
        }

        seen[joint] = true;
    }

    for joint: joints seen[joint] = false;

    return duplicate;
}

// Whether every joint is the skin's skeleton node or one of its descendants. Parents must be acyclic.
skeleton_covers :: (skin: Gltf_Skin, parents: [] s64) -> bool {
    for joint: skin.joints {
        node: s64 = joint;
        while node >= 0 && node != skin.skeleton.value node = parents[node];
        if node < 0 return false;
    }

    return true;
}

validate_scenes :: (asset: *Gltf_Asset, parents: [] s64, roots: [] s64) -> Gltf_Error {
    d := *asset.document;
    queue: [..] u32;

    defer array_free(queue);

    for scene, i: d.scenes {
        queue.count = 0;
        seen: Table(u32, bool);

        defer table_reset(*seen);

        for node, j: scene.nodes {
            p := tprint("/scenes/%/nodes/%", i, j);
            if node >= d.nodes.count return failure(.INVALID, "node index out of range", p);

            found, _ := table_find(*seen, node);
            if found || parents[node] != -1 return failure(.INVALID, "scene must contain distinct roots", p);

            table_add(*seen, node, true);
            array_add(*queue, node);
        }

        head: s64;
        while head < queue.count {
            node := queue[head];

            head += 1;
            n := d.nodes[node];
            for n.children array_add(*queue, it);
            if n.skin.state != .SET continue;

            // The hierarchy pass proved all joints share this root.
            root := roots[d.skins[n.skin.value].joints[0]];
            reachable, _ := table_find(*seen, cast(u32) root);

            if !reachable return failure(.INVALID, "skin joint is absent from scene", tprint("/nodes/%/skin", node));
        }
    }

    return .{};
}

validate_meshes :: (asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error {
    d := *asset.document;
    joint_limits: [..] s64;

    array_resize(*joint_limits, d.meshes.count);
    defer array_free(joint_limits);
    for *joint_limits it.* = 65536;
    for n: d.nodes if n.skin.state == .SET joint_limits[n.mesh.value] = min(joint_limits[n.mesh.value], d.skins[n.skin.value].joints.count);

    for m, i: d.meshes {
        if !m.primitives.count return failure(.INVALID, "mesh has no primitives", tprint("/meshes/%/primitives", i));

        target_count := m.primitives[0].targets.count;
        if m.weights.count && m.weights.count != target_count return failure(.INVALID, "mesh weights must match target count", tprint("/meshes/%/weights", i));

        for prim, j: m.primitives {
            p := tprint("/meshes/%/primitives/%", i, j);
            if prim.targets.count != target_count return failure(.INVALID, "mesh target counts differ", p);
            if !known(prim.mode) || !valid_ref(prim.material, d.materials.count) return failure(.INVALID, "invalid mode or material", p);
            if !valid_ref(prim.indices, d.accessors.count) return failure(.INVALID, "accessor index out of range", tprint("%/indices", p));

            if prim.indices.state == .SET {
                a := d.accessors[prim.indices.value];
                if a.type != .SCALAR || !unsigned_component(a.component_type) || a.normalized return failure(.INVALID, "invalid index accessor", tprint("%/indices", p));
            }

            count: s64 = -1;

            for value, key: prim.attributes {
                if value >= d.accessors.count return failure(.INVALID, "attribute accessor out of range", tprint("%/attributes/%", p, key));

                a := d.accessors[value];
                if count != -1 && count != a.count return failure(.INVALID, "attribute counts differ", p);

                count = a.count;
                data_error := attribute_data(asset, value, key, p);
                if data_error.kind != .NONE return data_error;

                format_ok := attribute_format(a, key);
                if !format_ok return failure(.INVALID, "invalid attribute format or missing position bounds", tprint("%/attributes/%", p, key));

                if a.buffer_view.state == .SET {
                    v := d.buffer_views[a.buffer_view.value];
                    if a.byte_offset % 4 != 0 || (reference(v.byte_stride) == -1 && element_size(a) % 4 != 0) return failure(.INVALID, "vertex data must be four-byte aligned", p);
                }
            }

            skin_error := skin_attributes(asset, prim, joint_limits[i], count, p);
            if skin_error.kind != .NONE return skin_error;

            e := validate_indices(asset, prim.indices, count, p);
            if e.kind != .NONE return e;

            for target, k: prim.targets for value, key: target {
                if value >= d.accessors.count return failure(.INVALID, "morph accessor out of range", tprint("%/targets/%/%", p, k, key));

                a := d.accessors[value];
                found, _ := table_find(*prim.attributes, key);

                if !found || (key == "POSITION" && (a.min.count != 3 || a.max.count != 3)) return failure(.INVALID, "morph requires base attribute and position bounds", p);
                if a.count != count || a.type != .VEC3 || a.component_type != .FLOAT return failure(.INVALID, "invalid morph accessor", p);
            }
        }
    }

    return .{};
}

validate_resources :: (asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error {
    d := *asset.document;
    for t, i: d.textures {
        if !valid_ref(t.source, d.images.count) || !valid_ref(t.sampler, d.samplers.count) return failure(.INVALID, "texture reference out of range", tprint("/textures/%", i));
    }

    for sampler, i: d.samplers {
        if (sampler.mag_filter.state == .SET && !known(sampler.mag_filter.value)) || (sampler.min_filter.state == .SET && !known(sampler.min_filter.value)) || !known(sampler.wrap_s) || !known(sampler.wrap_t) return failure(.INVALID, "invalid sampler filter or wrap mode", tprint("/samplers/%", i));
    }

    for image, i: d.images {
        p := tprint("/images/%", i);
        if !valid_ref(image.buffer_view, d.buffer_views.count) return failure(.INVALID, "image buffer view out of range", p);
        if image.buffer_view.state != .SET && !image.uri.count && image.extensions.kind == .NULL return failure(.INVALID, "image has no source", p);
        if image.buffer_view.state == .SET && (image.uri.count || (image.mime_type != "image/png" && image.mime_type != "image/jpeg")) return failure(.INVALID, "invalid image source", p);
    }

    for m, i: d.materials {
        if !known(m.alpha_mode) return failure(.INVALID, "invalid alpha mode", tprint("/materials/%/alphaMode", i));

        infos := *Gltf_Texture_Info.[m.normal_texture, m.occlusion_texture, m.emissive_texture];
        for info, j: infos if info && info.index >= d.textures.count return failure(.INVALID, "texture index out of range", tprint("/materials/%/textures/%", i, j));

        if m.pbr_metallic_roughness {
            pbr := m.pbr_metallic_roughness;
            for info: *Gltf_Texture_Info.[pbr.base_color_texture, pbr.metallic_roughness_texture] if info && info.index >= d.textures.count return failure(.INVALID, "texture index out of range", tprint("/materials/%/pbrMetallicRoughness", i));
        }
    }

    for s, i: d.skins {
        p := tprint("/skins/%", i);
        if !s.joints.count || !valid_ref(s.skeleton, d.nodes.count) || !valid_ref(s.inverse_bind_matrices, d.accessors.count) return failure(.INVALID, "invalid skin reference", p);
        for joint, j: s.joints if joint >= d.nodes.count return failure(.INVALID, "joint index out of range", tprint("%/joints/%", p, j));

        if s.inverse_bind_matrices.state == .SET {
            a := d.accessors[s.inverse_bind_matrices.value];
            if a.type != .MAT4 || a.component_type != .FLOAT || a.count < s.joints.count return failure(.INVALID, "invalid inverse bind matrices", p);

            cursor := Gltf_Cursor.{ asset = asset, accessor = s.inverse_bind_matrices.value };

            for 0..cast(s64) a.count - 1 {
                matrix, _, e := gltf_next_floats(*cursor);
                if e.kind != .NONE return e;
                if !affine_matrix(matrix) return failure(.INVALID, "inverse bind matrix must be affine", p);
            }
        }
    }

    return .{};
}

validate_animations :: (asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error {
    d := *asset.document;
    for anim, i: d.animations {
        p := tprint("/animations/%", i);
        if !anim.channels.count || !anim.samplers.count return failure(.INVALID, "empty animation", p);

        for s, j: anim.samplers {
            if s.input >= d.accessors.count || s.output >= d.accessors.count return failure(.INVALID, "animation accessor out of range", tprint("%/samplers/%", p, j));

            a := d.accessors[s.input];
            if a.min.count != 1 || a.max.count != 1 || (s.interpolation == .CUBICSPLINE && a.count < 2) return failure(.INVALID, "animation input requires bounds and sufficient keyframes", p);

            cursor := Gltf_Cursor.{ asset = asset, accessor = s.input };
            previous: float32 = -1;

            for 0..cast(s64) a.count - 1 {
                v, _, e := gltf_next_floats(*cursor);
                if e.kind != .NONE return e;
                if v[0] <= previous || v[0] < 0 return failure(.INVALID, "animation times must be nonnegative and increase", p);
                previous = v[0];
            }

            if a.type != .SCALAR || a.component_type != .FLOAT || !known(s.interpolation) return failure(.INVALID, "invalid animation input or interpolation", p);
        }

        targets: Table(u64, bool);

        defer table_reset(*targets);
        if anim.channels.count > options.max_count return failure(.LIMIT, "animation channel limit exceeded", p);

        for channel, j: anim.channels {
            cp := tprint("%/channels/%", p, j);
            if channel.sampler >= anim.samplers.count || !valid_ref(channel.target.node, d.nodes.count) return failure(.INVALID, "animation reference out of range", cp);

            path := channel.target.path;
            if channel.target.node.state == .SET && known(path) && path != .POINTER {
                lane := cast(u64) path;
                key := cast(u64) channel.target.node.value * 4 + lane;
                found, _ := table_find(*targets, key);

                if found return failure(.INVALID, "duplicate animation target", cp);

                table_add(*targets, key, true);
                node := d.nodes[channel.target.node.value];
                if node.matrix.count return failure(.INVALID, "animated node cannot have matrix", cp);

                sampler := anim.samplers[channel.sampler];
                input := d.accessors[sampler.input];
                output := d.accessors[sampler.output];
                multiplier: s64 = ifx sampler.interpolation == .CUBICSPLINE then 3 else 1;
                expected_type := ifx path == .ROTATION then Gltf_Accessor_Type.VEC4 else .VEC3;

                if path == .WEIGHTS {
                    if node.mesh.state != .SET || !d.meshes[node.mesh.value].primitives.count return failure(.INVALID, "weights require morph mesh", cp);
                    multiplier *= d.meshes[node.mesh.value].primitives[0].targets.count;
                    expected_type = .SCALAR;
                    if multiplier == 0 return failure(.INVALID, "weights require morph targets", cp);
                }

                if output.type != expected_type || output.component_type != .FLOAT || output.count != cast(s64) input.count * multiplier return failure(.INVALID, "animation output format or count mismatch", cp);
            }

            if !known(path) || (path == .POINTER && channel.target.extensions.kind == .NULL) return failure(.INVALID, "invalid animation target path", cp);
        }
    }

    return .{};
}

validate_cameras :: (asset: *Gltf_Asset, options: Gltf_Options) -> Gltf_Error {
    d := *asset.document;
    for camera, i: d.cameras {
        valid := false;

        if camera.type == .PERSPECTIVE && camera.perspective && !camera.orthographic {
            c := camera.perspective;
            valid = c.yfov > 0 && c.yfov < PI && c.znear > 0 && (c.zfar.state == .MISSING || c.zfar.value > c.znear) && (c.aspect_ratio.state == .MISSING || c.aspect_ratio.value > 0);
        } else if camera.type == .ORTHOGRAPHIC && camera.orthographic && !camera.perspective {
            c := camera.orthographic;
            valid = c.xmag != 0 && c.ymag != 0 && c.znear >= 0 && c.zfar > c.znear;
        }

        if !valid return failure(.INVALID, "invalid camera projection", tprint("/cameras/%", i));
    }

    return .{};
}

valid_ref :: (r: Json_Maybe(u32), count: s64) -> bool {
    return r.state == .MISSING || (r.state == .SET && r.value < count);
}

// Whether `value` is a member of its enum; hand-built assets can hold others.
known :: (value: $T) -> bool {
    for (cast(*Type_Info_Enum) type_info(T)).values if it == cast(s64) value return true;
    return false;
}

unsigned_component :: (c: Gltf_Component) -> bool {
    return c == .UNSIGNED_BYTE || c == .UNSIGNED_SHORT || c == .UNSIGNED_INT;
}

component_size :: (c: Gltf_Component) -> s64 {
    if c == {
        case .BYTE; return 1;
        case .UNSIGNED_BYTE; return 1;
        case .SHORT; return 2;
        case .UNSIGNED_SHORT; return 2;
        case .UNSIGNED_INT; return 4;
        case .FLOAT; return 4;
    }

    return 0;
}

layout :: (a: Gltf_Accessor) -> size: s64, rows: s64, cols: s64 {
    size = component_size(a.component_type);
    cols = 1;

    if a.type == {
        case .SCALAR; rows = 1;
        case .VEC2; rows = 2;
        case .VEC3; rows = 3;
        case .VEC4; rows = 4;
        case .MAT2;
            rows = 2;
            cols = 2;
        case .MAT3;
            rows = 3;
            cols = 3;
        case .MAT4;
            rows = 4;
            cols = 4;
    }

    return size, rows, cols;
}

element_size :: (a: Gltf_Accessor) -> s64 {
    size, rows, cols := layout(a);
    column := rows * size;
    if cols > 1 column = (column + 3) & ~3;
    return column * cols;
}

region :: (asset: *Gltf_Asset, view: u32, offset: u32, count: u32, size: s64, rows: s64, cols: s64, sparse: bool, path: string) -> Gltf_Error {
    d := *asset.document;
    if view >= d.buffer_views.count return failure(.INVALID, "buffer view index out of range", path);

    v := d.buffer_views[view];
    if sparse && (v.byte_stride.state != .MISSING || v.target.state != .MISSING) return failure(.INVALID, "sparse view cannot have stride or target", path);
    if offset % size != 0 || (cast(s64) v.byte_offset + offset) % size != 0 return failure(.INVALID, "misaligned component", path);

    column := rows * size;

    if cols > 1 {
        column = (column + 3) & ~3;
        if (cast(u64) v.byte_offset + offset) % 4 != 0 return failure(.INVALID, "misaligned matrix column", path);
    }

    stride := column * cols;
    if v.byte_stride.state == .SET stride = v.byte_stride.value;
    if stride < column * cols || stride % size != 0 return failure(.INVALID, "stride smaller than element or misaligned", path);

    // The final column's padding need not be present after the last element.
    end := cast(u64) offset + (cast(u64) count - 1) * cast(u64) stride + cast(u64) ((cols - 1) * column + rows * size);
    if end > v.byte_length return failure(.INVALID, "accessor exceeds buffer view", path);

    return .{};
}

view_data :: (asset: *Gltf_Asset, view: u32, offset: u32) -> string, s64 {
    v := asset.document.buffer_views[view];
    return asset.buffers[v.buffer], cast(s64) v.byte_offset + offset;
}

target_is_index :: (target: Json_Maybe(Gltf_Target)) -> bool {
    return target.state == .SET && target.value == .ELEMENT_ARRAY_BUFFER;
}

bound :: (value: float64, kind: Gltf_Component) -> float64 {
    if kind == .FLOAT return cast(float64) cast(float32) value;
    return value;
}

validate_indices :: (asset: *Gltf_Asset, indices: Json_Maybe(u32), count: s64, p: string) -> Gltf_Error {
    d := *asset.document;

    if indices.state == .SET && count >= 0 {
        a := d.accessors[indices.value];
        cursor := Gltf_Cursor.{ asset = asset, accessor = indices.value };

        for 0..cast(s64) a.count - 1 {
            index, _, e := gltf_next_index(*cursor);
            if e.kind != .NONE return e;
            if index >= count || index == 0xffffffff || (a.component_type == .UNSIGNED_BYTE && index == 255) || (a.component_type == .UNSIGNED_SHORT && index == 65535) return failure(.INVALID, "index exceeds vertex count or is primitive restart", p);
        }
    }

    return .{};
}

attribute_format :: (a: Gltf_Accessor, key: string) -> bool {
    format_ok := a.component_type != .UNSIGNED_INT;
    if key == "POSITION" format_ok = a.type == .VEC3 && a.component_type == .FLOAT && a.min.count == 3 && a.max.count == 3;
    else if key == "NORMAL" format_ok = a.type == .VEC3 && a.component_type == .FLOAT;
    else if key == "TANGENT" format_ok = a.type == .VEC4 && a.component_type == .FLOAT;
    else if begins_with(key, "TEXCOORD_") format_ok = a.type == .VEC2 && (a.component_type == .FLOAT || ((a.component_type == .UNSIGNED_BYTE || a.component_type == .UNSIGNED_SHORT) && a.normalized));
    else if begins_with(key, "JOINTS_") format_ok = a.type == .VEC4 && (a.component_type == .UNSIGNED_BYTE || a.component_type == .UNSIGNED_SHORT) && !a.normalized;
    else if begins_with(key, "WEIGHTS_") || begins_with(key, "COLOR_") format_ok = (a.type == .VEC4 || (begins_with(key, "COLOR_") && a.type == .VEC3)) && (a.component_type == .FLOAT || ((a.component_type == .UNSIGNED_BYTE || a.component_type == .UNSIGNED_SHORT) && a.normalized));

    return format_ok;
}

attribute_data :: (asset: *Gltf_Asset, index: u32, key: string, p: string) -> Gltf_Error {
    a := asset.document.accessors[index];

    if key != "NORMAL" && key != "TANGENT" && !begins_with(key, "COLOR_") return .{};

    cursor := Gltf_Cursor.{ asset = asset, accessor = index };
    _, rows, cols := layout(a);

    for 0..cast(s64) a.count - 1 {
        v, _, e := gltf_next_floats(*cursor);
        if e.kind != .NONE return e;

        if key == "NORMAL" || key == "TANGENT" {
            length := cast(float64) v[0] * v[0] + cast(float64) v[1] * v[1] + cast(float64) v[2] * v[2];
            // Squared bounds correspond to a 0.001 error in vector length.
            if length < 0.998001 || length > 1.002001 || (key == "TANGENT" && v[3] != -1 && v[3] != 1) return failure(.INVALID, "invalid normal or tangent", p);
        } else if begins_with(key, "COLOR_") for k: 0..rows * cols - 1 if v[k] < 0 || v[k] > 1 return failure(.INVALID, "color outside unit interval", p);
    }

    return .{};
}

affine_matrix :: (m: [] float32) -> bool {
    return m[3] == 0 && m[7] == 0 && m[11] == 0 && m[15] == 1;
}

trs_matrix :: (m: [] float32) -> bool {
    if !affine_matrix(m) return false;

    // TRS columns are orthogonal, including reflections and zero scale axes.
    for a: 0..1 for b : a + 1..2 {
        dot, aa, bb: float64;

        for k: 0..2 {
            x := cast(float64) m[a * 4 + k];
            y := cast(float64) m[b * 4 + k];

            dot += x * y;
            aa += x * x;
            bb += y * y;
        }

        if dot * dot > 1.0e-10 * aa * bb return false;
    }

    return true;
}

skin_attributes :: (asset: *Gltf_Asset, prim: Gltf_Primitive, joint_limit: s64, count: s64, p: string) -> Gltf_Error {
    joints, weights: [..] Gltf_Cursor;

    defer array_free(joints);
    defer array_free(weights);
    integer_only := true;

    for value, key: prim.attributes {
        joint := begins_with(key, "JOINTS_");
        weight := begins_with(key, "WEIGHTS_");

        if !joint && !weight continue;

        prefix: s64 = ifx joint then 7 else 8;
        suffix := slice(key, prefix, key.count - prefix);
        paired, other := table_find(*prim.attributes, tprint("%%", ifx joint then "WEIGHTS_" else "JOINTS_", suffix));

        if !paired return failure(.INVALID, "joint and weight sets must match", p);
        if !joint continue;

        array_add(*joints, .{ asset = asset, accessor = value });
        array_add(*weights, .{ asset = asset, accessor = other });
        kind := asset.document.accessors[other].component_type;
        if kind == .FLOAT integer_only = false;
    }

    if !joints.count return .{};

    seen: [..] s64;

    array_resize(*seen, joint_limit);
    defer array_free(seen);
    for *seen it.* = -1;

    for vertex: 0..count - 1 {
        sum: float64;
        quantized_sum, nonzero: s64;

        for *jc, set: joints {
            jv, _, je := gltf_next_floats(jc);
            if je.kind != .NONE return je;

            wv, _, we := gltf_next_floats(*weights[set]);
            if we.kind != .NONE return we;

            for lane: 0..3 {
                index := cast(s64) jv[lane];
                if index >= joint_limit return failure(.INVALID, "joint index exceeds skin", p);
                if wv[lane] < 0 return failure(.INVALID, "negative skin weight", p);

                sum += wv[lane];
                // A float32 retains enough precision to recover either unorm integer exactly.
                if integer_only quantized_sum += cast(s64) (cast(float64) wv[lane] * 65535.0 + 0.5);
                if wv[lane] == 0 continue;

                nonzero += 1;
                if seen[index] == vertex return failure(.INVALID, "duplicate weighted joint", p);

                seen[index] = vertex;
            }
        }

        normalized := ifx integer_only then quantized_sum == 65535 else abs(sum - 1.0) <= 2.0e-7 * nonzero;
        if !normalized return failure(.INVALID, "skin weights must sum to one", p);
    }

    return .{};
}
