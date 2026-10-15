#include "bindings-multi-base.h"
#include "bindings-multi-extra.h"
int multi_base(const MultiRecord *record) { return record->value * MULTI_SCALE; }
int multi_sum(const MultiRecord *record, int value) { return multi_base(record) + value + MULTI_OFFSET; }
