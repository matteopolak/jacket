f :: (l := #caller_location) {}
X :: #run f();
main :: () { f(); }
