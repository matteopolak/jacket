#include "bindings-canonical-printing.h"
int original_global = 17;
int original_sum(const struct OriginalRecord *record, int increment) {
    return record->first + (int)record->second + increment + original_global;
}
void original_initialize(struct OriginalRecord *record, int first, double second) {
    record->first = first;
    record->second = second;
}
void original_container_initialize(struct OriginalContainer *record, int payload, int tail) {
    record->inner.payload = payload;
    record->tail = tail;
}
int original_container_read(const struct OriginalContainer *record) {
    return record->inner.payload + record->tail;
}
