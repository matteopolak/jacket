#import "Basic";

Creature :: struct {
    name := "nobody";
    health: s32 = 10;
}

main :: () {
    knight := Creature.{ name = "Sir Robin" };
    knight.health = "full";
    print("%\n", knight.name);
}
