// rules: modparam.5
// A program parameter cannot be given a new value once the shared instance has used it:
// the #run below calls Basic's assert, which depends on ENABLE_ASSERT, before the inserted
// import tries to turn it off.
#import "Basic";
READY :: #run -> bool { assert(true); return true; };
#insert #run -> string { assert(READY); return "#import \"Basic\"()(ENABLE_ASSERT = false);"; };
main :: () {}
