#import "Basic";

Holder :: struct(Elem: Type, chunk: int, mode := false) {
    storage: *Elem;
}

operator *[] :: (using holder: Holder, index: int) -> *holder.Elem {
    return storage + index;
}
