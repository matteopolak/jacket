#import "Basic";

// A creature in the tour's little world.
Creature :: struct {
    name := "nobody";
    health: s32 = 10;
    alive := true;
    position: [2] float;
}

heal :: (creature: *Creature, amount: s32) {
    creature.health += amount;
}

main :: () {
    knight := Creature.{ name = "Sir Robin" };
    heal(*knight, 5);
    if knight.alive == true print("% has % health\n", knight.name, knight.health);
}
