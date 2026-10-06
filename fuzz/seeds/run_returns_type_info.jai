f :: () -> *Type_Info { return type_info(int); }
T :: #run f();
main :: () { t := T; }
