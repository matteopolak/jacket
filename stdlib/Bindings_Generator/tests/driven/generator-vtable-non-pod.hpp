// A slot that returns a class with a user-provided constructor is not a plain record: it is refused.
struct NonPod {
    int a;
    NonPod() : a(0) {}
};
struct NonPodHolder {
    virtual NonPod make(int value) { NonPod result; result.a = value; return result; }
};
