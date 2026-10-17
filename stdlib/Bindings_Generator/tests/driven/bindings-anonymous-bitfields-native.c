struct OriginalContainer {
    struct { unsigned first : 3; unsigned second : 5; } inner;
    unsigned tail;
};
void original_initialize(struct OriginalContainer* value) {
    value->inner.first = 5;
    value->inner.second = 17;
    value->tail = 99;
}
unsigned original_read(const struct OriginalContainer* value) {
    return value->inner.first + value->inner.second + value->tail;
}
unsigned long original_size(void) { return sizeof(struct OriginalContainer); }
unsigned long original_alignment(void) { return _Alignof(struct OriginalContainer); }
