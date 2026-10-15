#include "bindings-cpp-scopes.h"
namespace alpha {
Sample adjust(Sample value, int delta) { value.value += delta; return value; }
int repeat(int value) { return value + 11; }
const char *identity(const char *text) { return text; }
long *long_identity(long *value) { return value; }
long invoke(long (*callback)(long), long value) { return callback(value) + 5; }
extern "C" int _source_identity(int value) { return value * 4; }
namespace detail { int scale(int value) { return value * 3; } }
}
namespace beta {
double adjust(Sample value, double factor) { return value.value * factor; }
}
