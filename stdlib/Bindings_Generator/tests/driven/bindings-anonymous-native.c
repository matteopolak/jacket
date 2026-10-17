#include "bindings-anonymous.h"
int original_anonymous(Anonymous_Record *record, Anonymous_Union *value, Anonymous_Mode mode) {
  record->value += mode;
  value->number = record->weight;
  return record->value;
}
int original_alias(Record_Alias *record, Mode_Alias mode, Forward_Record *forward) {
  forward->count += record->value + mode;
  return forward->count;
}
