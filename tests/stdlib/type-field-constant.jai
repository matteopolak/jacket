// `Type.field.Constant`: a field named through its struct type reaches its type's constants.
#import "Basic";

Map :: struct(K: Type) {
    Load_Factor :: 70;
    Entry :: struct {
        key: K;
        hash: u32;
    }
}
Anim :: struct {
    joints: Map(int);
    count: int;
}

main :: () {
    if Anim.joints.Load_Factor != 70 exit(1);
    if size_of(Anim.joints.Entry) != 16 exit(2);
    print("ok\n");
}
