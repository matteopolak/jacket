#include <stdarg.h>
inline void* operator new(__SIZE_TYPE__, void* memory) noexcept { return memory; }
inline int global_cpp(int value = 4) { return value + 38; }
namespace NativeBridge {
#define DEEP_TWELVE * * * * * * * * * * * *
inline long DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE deep(
    long DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE DEEP_TWELVE value) { return value; }
#undef DEEP_TWELVE
struct Pair { int first; int second; };
inline int sum(int first, int second = 7) { return first + second; }
inline long choose(long value) { return value + 100; }
inline long long choose(long long value) { return value + 200; }
inline Pair make_pair(int out_return_value) { return {out_return_value, out_return_value + 1}; }
inline int& shared_value() { static int value = 9; return value; }
inline int bump(int& value) { value += 4; return value; }
inline int sum_array(const int values[3]) { return values[0] + values[1] + values[2]; }
inline int sum_matrix(const int values[2][3]) {
    return values[0][0] + values[0][1] + values[0][2] + values[1][0] + values[1][1] + values[1][2];
}
inline int destroyed = 0;
struct Box {
    int value;
    explicit Box(int initial) : value(initial) {}
    int add(int amount) const { return value + amount; }
    ~Box() { ++destroyed; }
};
inline int destruction_count() { return destroyed; }
inline int message(const char* format, ...) {
    va_list arguments;
    va_start(arguments, format);
    int count = va_arg(arguments, int);
    const char* data = va_arg(arguments, const char*);
    va_end(arguments);
    if (format[0] != '%' || format[1] != '.' || format[2] != '*' || format[3] != 's' || format[4] != 0) return -1;
    int sum = 0;
    for (int index = 0; index < count; ++index) sum += static_cast<unsigned char>(data[index]);
    return sum;
}
}
