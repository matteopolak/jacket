#include "bindings-generic-record.hpp"
extern "C" int original_generic_default(IndependentGeneric<> *value) {
    value->data[0] += 7;
    value->data[2] = 42;
    return sizeof(*value);
}
extern "C" int original_generic_bytes(IndependentGeneric<unsigned char, 7> *value) {
    value->data[6] += 25;
    return sizeof(*value);
}
