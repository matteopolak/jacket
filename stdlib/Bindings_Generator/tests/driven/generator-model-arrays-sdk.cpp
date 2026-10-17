#include "generator-model-arrays.hpp"
#include <cstddef>
#include <cstdio>
#include <type_traits>
using Instance = ModelArrayBounds<int, 3, 4>;
static_assert(std::extent_v<decltype(Instance::single)> == 3);
static_assert(std::extent_v<decltype(Instance::matrix), 0> == 3);
static_assert(std::extent_v<decltype(Instance::matrix), 1> == 4);
static_assert(std::extent_v<decltype(Instance::mixed), 0> == 2);
static_assert(std::extent_v<decltype(Instance::mixed), 1> == 4);
static_assert(std::extent_v<decltype(Instance::mixed), 2> == 4);
static_assert(std::is_same_v<decltype(Instance::pointer), int (*)[3]>);
static_assert(offsetof(Instance, single) == 0);
static_assert(offsetof(Instance, matrix) == 12);
static_assert(offsetof(Instance, mixed) == 60);
static_assert(offsetof(Instance, pointer) == 192);
static_assert(std::is_same_v<decltype(Instance::callbacks), int (*(*[3])())[4]>);
static_assert(offsetof(Instance, callbacks) == 200);
static_assert(std::is_same_v<decltype(Instance::callback_arguments), int (*[3])(int (*)[4])>);
static_assert(offsetof(Instance, callback_arguments) == 224);
static_assert(std::is_same_v<decltype(Instance::callback_both), int (*(*[3])(int (*)[4]))[4]>);
static_assert(offsetof(Instance, callback_both) == 248);
static_assert(std::is_same_v<decltype(Instance::reference_callback), int (*)(const int&)>);
static_assert(offsetof(Instance, reference_callback) == 272);
static_assert(sizeof(Instance) == 280 && alignof(Instance) == 8);
int main() { Instance value{}; return value.pointer == nullptr ? (std::puts("ok"), 0) : 1; }
