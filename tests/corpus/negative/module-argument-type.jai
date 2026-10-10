// An import argument must have the parameter's kind: a bool flag takes no number.
Mod :: #import,string "#module_parameters(FLAG := false);\nvalue :: 1;"(FLAG = 3);

main :: () {}
