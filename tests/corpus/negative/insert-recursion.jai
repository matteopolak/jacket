// Found by fuzzing: the inserted string inserts itself, forever.
X :: "#insert X;";

main :: () {
    #insert X;
}
