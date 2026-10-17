M :: #import "Alias_Facade";

#assert M.Public_Alias == M.Shared;
#assert M.Public_Alias.LOCAL == 7;
main :: () {}
