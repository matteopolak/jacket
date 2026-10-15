struct Forward;
struct Opaque;
struct Forward { int value; };
struct Opaque *forward_call(struct Forward *value);
