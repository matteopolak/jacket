#include "generator-model-defaults.hpp"
#include "generator-model-wide.hpp"
#include <cstddef>
#include <type_traits>
static_assert(sizeof(IndependentGeneric<>) == 12);
static_assert(sizeof(IndependentGeneric<unsigned char, 7>) == 7);
static_assert(std::is_same<decltype(DefaultPointer<>::value),
                           const unsigned long long *>::value);
static_assert(std::is_same<decltype(DefaultArray<>::value), int[3]>::value);
static_assert(
    std::is_same<decltype(DefaultCallback<>::value), void (*)(int)>::value);
static_assert(std::is_same<decltype(DefaultReference<>::value), int &>::value);
static_assert(std::is_same<decltype(DefaultMacroBefore<>::value), int>::value);
static_assert(
    std::is_same<decltype(DefaultLookup::Scoped<>::value), double>::value);
static_assert(
    std::is_same<decltype(DefaultFirst::RepeatedDefault<>::value), int>::value);
static_assert(std::is_same<decltype(DefaultSecond::RepeatedDefault<>::value),
                           double>::value);
static_assert(sizeof(ModelWideRecord) == 32 && alignof(ModelWideRecord) == 16);
static_assert(offsetof(ModelWideRecord, unsigned_value) == 16);
extern "C" void model_default_fill(ModelWideRecord *record) {
  record->signed_value = -7;
  record->unsigned_value = (static_cast<unsigned __int128>(9) << 64) | 17;
}
#ifndef MODEL_DEFAULT_LIBRARY
int main() {
  IndependentGeneric<> value{};
  value.data[2] = 42;
  ModelWideRecord wide{};
  model_default_fill(&wide);
  return value.data[2] != 42 || wide.signed_value != -7 ||
         (wide.unsigned_value >> 64) != 9;
}
#endif
