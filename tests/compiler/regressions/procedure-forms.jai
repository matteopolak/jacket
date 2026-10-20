// Procedure features: arguments, defaults, multiple results, locals, lambdas, overloads,
// inlining, recursion, polymorphism and reflection.
#import "Basic";
#import "Math";

shouted := false;

times :: (a: int, b: int) -> int {
    return a * b;
}

floor_at_zero :: (x: int) -> int {
    if x < 0 return 0;
    return x;
}

Window :: struct {
    x, y, width, height: float;
}

make_window :: (x: float, y: float, width: float, height: float) -> Window {
    return .{ x, y, width, height };
}

sum :: (values: ..int) -> int {
    total := 0;
    for values total += it;
    return total;
}

Launch :: struct {
    altitude, heading: float;
    thrust: int;
}

launch :: (altitude := 0.0, heading := 0.0, thrust := 100) -> Launch {
    return .{ altitude, heading, thrust };
}

divide :: (a: int, b: int) -> int, int {
    return a / b, a % b;
}

fetch :: (key: string) -> data: int = -1, exists: bool = false {
    if key == "secret" return 42, true;
    return;
}

clear :: (value: *int) {
    value.* = 0;
}

try_modify :: (value: int) -> int {
    value += 1;
    return value;
}

Point :: struct {
    x, y: float;
}

translate :: (p: *Point, dx: float, dy: float) {
    p.x += dx;
    p.y += dy;
}

unit_direction :: (p: Point) -> Point {
    result := p;
    len := sqrt(result.x * result.x + result.y * result.y);

    result.x /= len;
    result.y /= len;

    return result;
}

subtract :: (a, b) => a - b;

show :: (n: int) -> string {
    return "int";
}

show :: (s: string) -> string {
    return "string";
}

show :: (f: float) -> string {
    return "float";
}

quick_negate :: inline (x: int) -> int {
    return -x;
}

cube_it :: (x: int) -> int {
    return x * x * x;
}

factorial :: (n: int) -> int {
    if n <= 1 return 1;
    return n * factorial(n - 1);
}

countdown :: (n: int) -> int {
    if n <= 0 return 0;
    return 1 + #this(n - 1);
}

set_opacity :: (level: float) -> float {
    return level;
}

Actor :: struct {
    x, y: float;
    label: string;
}

nudge_right :: (using a: *Actor, amount: float) {
    x += amount;
}

identity :: (x: $T) -> T {
    return x;
}

sum_array :: (values: [$N] int) -> int {
    total := 0;
    for values total += it;
    return total;
}

array_length :: (values: [$N] int) -> int {
    return N;
}

Box :: struct(T: Type, N: int) {
    items: [N] T;
}

first_item :: (b: Box) -> b.T {
    return b.items[0];
}

fill :: (b: *Box($T, $N), value: T) {
    for 0..N - 1 b.items[it] = value;
}

pair_count :: (b: Box($T, 2)) -> int {
    return 2;
}

int_box_size :: (b: Box(int, $N)) -> int {
    return N;
}

exact_box :: (_b: Box(int, 2)) -> int {
    return 22;
}

box_length :: (b: $T/Box) -> int {
    return b.N;
}

Named :: struct {
    name: string;
}

Player :: struct {
    name: string;
    score: int;
}

greeting :: (x: $T/interface Named) -> string {
    return x.name;
}

whoami :: () -> string {
    return #procedure_name();
}

main :: () {
    assert(times(3, 4) == 12);
    assert(times(times(2, 3), times(4, 5)) == 120);
    assert(floor_at_zero(-5) == 0 && floor_at_zero(5) == 5);

    // Named arguments in any order.
    w := make_window(width = 800, height = 600, x = 50, y = 100);

    assert(w.x == 50 && w.y == 100 && w.width == 800 && w.height == 600);
    w2 := make_window(1, 2, height = 4, width = 3);

    assert(w2.width == 3 && w2.height == 4);

    // Variable arguments and expansion.
    assert(sum() == 0);
    assert(sum(10, 20, 30) == 60);
    numbers := int.[10, 20, 30];

    assert(sum(..numbers) == 60);
    assert(sum(values = ..numbers) == 60);

    // Defaults, skipping one with a name.
    l := launch();

    assert(l.altitude == 0 && l.heading == 0 && l.thrust == 100);
    l = launch(500.0);
    assert(l.altitude == 500 && l.thrust == 100);
    l = launch(500.0, 90.0);
    assert(l.heading == 90 && l.thrust == 100);
    l = launch(thrust = 50);
    assert(l.altitude == 0 && l.thrust == 50);
    l = launch(200.0, thrust = 75);
    assert(l.altitude == 200 && l.heading == 0 && l.thrust == 75);

    // Several results, some of them dropped.
    quotient, remainder := divide(17, 5);

    assert(quotient == 3 && remainder == 2);
    only_quotient := divide(17, 5);

    assert(only_quotient == 3);
    _, rem := divide(17, 5);

    assert(rem == 2);

    // Named results with defaults.
    d, ok := fetch("secret");

    assert(d == 42 && ok);
    d2, ok2 := fetch("missing");

    assert(d2 == -1 && !ok2);
    _, exists := fetch("secret");

    assert(exists);

    // Arguments are copies unless a pointer is passed.
    health := 75;

    assert(try_modify(health) == 76);
    assert(health == 75);
    clear(*health);
    assert(health == 0);

    cursor := Point.{ 10, 20 };

    translate(*cursor, 5, -3);
    assert(cursor.x == 15 && cursor.y == 17);
    dir := unit_direction(Point.{ 3, 4 });

    assert(dir.x == 0.6 && dir.y == 0.8);

    // Local procedures.
    cube :: (n: int) -> int {
        return n * n * n;
    }

    assert(cube(4) == 64);

    // Procedure values and types.
    op: (int, int) -> int = times;

    assert(op(6, 7) == 42);
    assert(type_of(times) == (int, int) -> int);
    callbacks: [2] (int) -> int;
    callbacks[0] = cube_it;
    callbacks[1] = quick_negate;
    assert(callbacks[0](3) == 27 && callbacks[1](3) == -3);

    // Lambdas and anonymous procedures.
    assert(subtract(10, 3) == 7);

    result := (x: int, y: int) -> int {
        return x + y;
    } (15, 27);

    assert(result == 42);

    shout := () {
        shouted = true;
    };

    shout();
    assert(shouted);

    // Overloads pick the best match.
    assert(show(99) == "int");
    assert(show("world") == "string");
    assert(show(2.718) == "float");

    // Inlining at the declaration and at the call.
    assert(quick_negate(7) == -7);
    assert(no_inline quick_negate(7) == -7);
    assert(inline cube_it(3) == 27);

    assert(factorial(10) == 3628800);
    assert(countdown(6) == 6);

    // `xx` converts at the call.
    val: int = 75;

    assert(set_opacity(xx val) == 75.0);

    // `using` on a parameter.
    hero := Actor.{ 0, 0, "Hero" };

    nudge_right(*hero, 5);
    assert(hero.x == 5);

    // Polymorphism.
    assert(identity(42) == 42);
    assert(identity("hello") == "hello");
    assert(type_of(identity(3.5)) == float);
    nums := int.[10, 20, 30, 40];

    assert(sum_array(nums) == 100);
    assert(array_length(nums) == 4);

    b: Box(int, 4);
    b.items[0] = 99;
    assert(first_item(b) == 99);
    f: Box(float, 3);

    fill(*f, 1.5);
    assert(f.items[0] == 1.5 && f.items[2] == 1.5);

    a2: Box(float, 2);
    b5: Box(int, 5);
    c2: Box(int, 2);

    assert(pair_count(a2) == 2 && pair_count(c2) == 2);
    assert(int_box_size(b5) == 5 && int_box_size(c2) == 2);
    assert(exact_box(c2) == 22);

    assert(box_length(b5) == 5);

    p := Player.{ name = "Ada", score = 1200 };

    assert(greeting(p) == "Ada");

    assert(whoami() == "whoami");

    // Reflection on a procedure type.
    info := cast(*Type_Info_Procedure) type_info(type_of(times));

    assert(info.type == .PROCEDURE);
    assert(info.argument_types.count == 2);
    assert(info.argument_types[0].type == .INTEGER);
    assert(info.return_types.count == 1);
    assert(info.return_types[0].type == .INTEGER);

    print("ok\n");
}
