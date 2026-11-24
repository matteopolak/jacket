typedef __SIZE_TYPE__ size_t;
typedef __UINTPTR_TYPE__ uintptr_t;

struct Mixed {
    char tag;
    long long wide;
    void *pointer;
    size_t count;
    short tail;
};

struct Names {
    const char *text;
    int (*callback)(void *context);
    long counts[3];
};

int mixed_total(struct Mixed *value, size_t count);
