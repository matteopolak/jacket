#include "bindings-generic-record.hpp"
typedef int NamedDefault;
#define MODEL_DEFAULT_TYPE const unsigned long long *
template <class T = MODEL_DEFAULT_TYPE> struct DefaultPointer {
  T value;
};
template <class T = int[3]> struct DefaultArray {
  T value;
};
template <class T = void (*)(int)> struct DefaultCallback {
  T value;
};
template <class T = int &> struct DefaultReference {
  T value;
};
template <class T = NamedDefault> struct DefaultAlias {
  T value;
};
template <class T = decltype(1)> struct DefaultDecltype {
  T value;
};
template <class T = __int128> struct DefaultSignedWide {
  T value;
};
template <class T = unsigned __int128> struct DefaultUnsignedWide {
  T value;
};
const __int128 *model_qualified_wide;
#define MODEL_REDEFINED_TYPE int
template <class T = MODEL_REDEFINED_TYPE> struct DefaultMacroBefore {
  T value;
};
#undef MODEL_REDEFINED_TYPE
#define MODEL_REDEFINED_TYPE double
int model_default_lookup;
namespace DefaultLookup {
double model_default_lookup;
template <class T = decltype(model_default_lookup)> struct Scoped {
  T value;
};
} // namespace DefaultLookup
#define MODEL_REPEAT_TYPE int
namespace DefaultFirst {
#include "generator-model-defaults-repeat.inc"
}
#undef MODEL_REPEAT_TYPE
#define MODEL_REPEAT_TYPE double
namespace DefaultSecond {
#include "generator-model-defaults-repeat.inc"
}
