namespace alpha {
struct Sample { int value; };
Sample adjust(Sample value, int delta);
int repeat(int value);
const char *identity(const char *text);
long *long_identity(long *value);
long invoke(long (*callback)(long), long value);
extern "C" int _source_identity(int value);
}
namespace beta {
struct Sample { double value; };
double adjust(Sample value, double factor);
}
namespace alpha {
int repeat(int value);
namespace detail { int scale(int value); }
}
