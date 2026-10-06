// Structs, `using`, `#as`, operator overloading and unions.

Vec2 :: struct {
    x, y: float;
}

// Operators are ordinary procedures named `operator <symbol>`.
operator + :: (a: Vec2, b: Vec2) -> Vec2 {
    return .{ a.x + b.x, a.y + b.y };
}
// `#symmetric` lets the scalar go on either side: v * 2 and 2 * v both work.
operator * :: (v: Vec2, s: float) -> Vec2 #symmetric {
    return .{ v.x * s, v.y * s };
}

// Fields can have default values.
Creature :: struct {
    name := "nobody";
    position: Vec2;
    health := 10;
}

// `using` brings a member's fields into this struct, so `knight.name` works.
// `#as` also lets a Knight convert to a Creature implicitly.
Knight :: struct {
    #as using creature: Creature;
    sword := "a rusty spoon";
}

Dragon :: struct {
    #as using creature: Creature;
    fire_breath := 8;
}

// A union's members share the same bytes.
Float_Bits :: union {
    value: float32;
    bits: u32;
}

tour_structs :: () {
    // Struct literals: positional, named, or `.{}` when the type is known.
    knight: Knight;
    knight.name = "Sir Robin";
    knight.position = .{ 1, 2 };
    dragon := Dragon.{ creature = .{ name = "Smaug", health = 50 } };

    // Operator overloading in action.
    step := Vec2.{ 0.5, 0.25 };
    knight.position = knight.position + step * 4;
    print("% walks to (%, %) carrying %.\n",
        knight.name, knight.position.x, knight.position.y, knight.sword);

    // Both are Creatures thanks to `#as`, so `battle` takes either.
    battle(*knight, *dragon);
    battle(*dragon, *knight);
    print("Afterwards: % has % health, % has % health.\n",
        knight.name, knight.health, dragon.name, dragon.health);

    // `using` works on parameters too.
    report(dragon);

    // Peek at the bits of a float through a union.
    one: Float_Bits;
    one.value = 1.0;
    print("1.0 as raw IEEE-754 bits: 0x%\n", formatInt(one.bits, base = 16));

    print("A Knight takes % bytes, a Creature %.\n", size_of(Knight), size_of(Creature));
}

battle :: (attacker: *Creature, defender: *Creature) {
    damage := attacker.health / 5 + 1;
    defender.health -= damage;
    print("  % hits % for % damage!\n", attacker.name, defender.name, damage);
}

report :: (using creature: Creature) {
    // `name` and `health` here are creature.name and creature.health.
    print("  Status report: % (% hp) at (%, %)\n", name, health, position.x, position.y);
}
