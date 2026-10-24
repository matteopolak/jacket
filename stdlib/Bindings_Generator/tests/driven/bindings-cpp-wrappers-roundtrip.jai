// Calls C++ objects through the generated slot wrappers.
#import "Basic";
#import "String";
#load "developer-tools-cpp-wrappers-shapes.jai";
#load "developer-tools-cpp-wrappers-sdk.jai";

native :: #library "cppwrappers";
wrappers_create_cube :: (side: s32, x: s32, y: s32) -> *ICube #foreign native;
wrappers_create_global_session :: () -> *IGlobalSession #foreign native;
wrappers_live_objects :: () -> s32 #foreign native;

check_shapes :: () {
    cube := wrappers_create_cube(4, 10, 20);

    assert(cube != null);

    // A method declared by the record itself.
    assert(ICube_volume(cube) == 64);

    // Methods of the ancestors, called through a derived pointer by the declaring class's name.
    box := cast(*IBox) cube;
    shape := cast(*IShape) cube;

    assert(IBox_side(cube) == 4 && IBox_side(box) == 4);
    assert(IShape_area(cube) == 96 && IShape_area(box) == 96 && IShape_area(shape) == 96);

    // Overloads: a value and a reference parameter.
    assert(IShape_scale(cube, 2) == 8);

    point := Point.{ 2, 1 };

    assert(IShape_scale_1(box, *point) == 16);
    assert(ICube_volume(cube) == 4096);

    // References come back as pointers into the object.
    origin := IShape_origin(cube);

    assert(origin.x == 10 && origin.y == 20);

    cursor := IShape_cursor(shape);

    cursor.x = 5;
    cursor.y = 6;
    assert(IShape_cursor(cube).x == 5 && IShape_cursor(box).y == 6);

    // An aggregate result, a double and a float, and a method named like a Jai keyword.
    moved := IShape_offset(cube, 3);

    assert(moved.x == 13 && moved.y == 23);
    assert(IShape_ratio(cube, 1.5, 2) == 19.0);
    assert(IShape_cast(cube, 4) == 20);

    // The wrappers reach the same slots as a direct call.
    assert(cube.__vtable.side(box) == IBox_side(cube));
    assert(ICube.vtable(cube) == cube.__vtable);

    // A method of the base interface, and out parameters.
    assert(IRefCounted_addRef(cube) == 2);
    assert(IRefCounted_release(shape) == 1);

    found: *void;

    assert(IRefCounted_queryInterface(cube, 3, *found) == 0 && found == cube);
    assert(IRefCounted_queryInterface(cube, 9, *found) == -1);
    assert(IRefCounted_release(box) == 1);

    first: *IShape;
    second: *IShape;

    assert(IBox_split(box, *first, *second) == 0);
    assert(IShape_area(first) == 6 * 8 * 8 && IShape_area(second) == 6 * 8 * 8);
    assert(IRefCounted_release(first) == 0);
    assert(IRefCounted_release(second) == 0);

    assert(IRefCounted_release(cube) == 0);
}

check_sdk :: () {
    global := wrappers_create_global_session();

    assert(global != null);
    assert(to_string(IGlobalSession_getBuildTagString(global)) == "wrappers-1");

    desc := SessionDesc.{ flags = 3, targetCount = 2 };
    session: *ISession;

    assert(IGlobalSession_createSession(global, *desc, *session) == 0);

    described: SessionDesc;

    assert(ISession_describe(session, *described) == 0);
    assert(described.flags == 3 && described.targetCount == 2);
    assert(ISession_getIndex(session) == 1);

    // The base interface's wrappers accept the derived pointers.
    uuid: SlangUUID;

    uuid.data1 = 7;
    found: *void;

    assert(ISlangUnknown_queryInterface(session, *uuid, *found) == 0 && found == session);
    assert(ISlangUnknown_addRef(session) == 3);
    assert(ISlangUnknown_release(session) == 2);
    assert(ISlangUnknown_release(session) == 1);
    assert(ISlangUnknown_release(session) == 0);

    uuid.data1 = 1;
    assert(ISlangUnknown_queryInterface(global, *uuid, *found) != 0);
    assert(ISlangUnknown_release(global) == 0);
}

main :: () {
    check_shapes();
    check_sdk();
    assert(wrappers_live_objects() == 0);
    print("ok\n");
}
