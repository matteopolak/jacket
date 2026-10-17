typedef struct member_allocate { void *pointer; unsigned size; } member_allocate;
typedef struct member_free { void *pointer; } member_free;

typedef struct member_op {
    unsigned kind;
    union {
        member_allocate allocate;
        member_free release;
    };
    struct {
        char tag;
        signed char delta;
    };
} member_op;

int member_text(const char *text, char letter, signed char small);
