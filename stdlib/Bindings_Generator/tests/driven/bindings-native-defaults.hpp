#include <stddef.h>
struct NativeDefaults {
    decltype(nullptr) absent = nullptr;
    size_t bytes = sizeof(void *);
};
#if __SIZEOF_LONG_DOUBLE__ == 8
struct NativeDouble { long double value; };
#endif
struct AbstractNative {
    virtual int apply(int value) = 0;
};
extern "C" { namespace Reopened { struct Earlier { int value; }; } }
namespace Reopened {
struct Forward { virtual int call(int value) = 0; };
}
template<int K> struct NativeGeneric {
    struct Inner { int value; };
};
static const unsigned NativeFlags = 9;
struct NativeIdentifier { unsigned flags = NativeFlags; };
extern "C" { namespace NativeLazy { struct Later; } }
namespace NativeLazy {
enum class Choice { Default };
struct Later { Choice choice = Choice::Default; };
}
