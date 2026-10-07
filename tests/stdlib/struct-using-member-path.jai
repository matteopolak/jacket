// `using,only(...) field.path;` in a struct makes those members of a nested field reachable directly.
#import "Basic";

Desc :: struct {
    width: int;
    height: int;
    depth: int;
}

Tex :: struct {
    desc: Desc;
    id: int;
}

Tex2D :: struct {
    #as texture: Tex;
    using,only(width, height) texture.desc;
}

main :: () -> s64 {
    t: Tex2D;
    t.texture.desc.width = 4;
    t.height = 7;
    if t.width != 4 || t.height != 7 || t.texture.desc.height != 7 return 1;
    return 0;
}

// The path may go through a pointer field.
Mesh :: struct {
    armature: int;
    other: int;
}

Pose :: struct {
    mesh: *Mesh;
    using,only(armature) mesh;
}

#run {
    m := Mesh.{ armature = 4 };
    p := Pose.{ mesh = *m };
    assert(p.armature == 4);
    p.armature = 6;
    assert(m.armature == 6);
}
