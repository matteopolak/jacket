namespace OriginalLeft { struct Shared { int field; }; }
namespace OriginalRight {
typedef OriginalLeft::Shared Shared;
Shared *value;
}
template<typename T> struct OriginalUnsupported;
template<> struct OriginalUnsupported<int> { int field; };
template<> struct OriginalUnsupported<double> { double field; };
struct OriginalAfterUnsupported { int field; };
struct OriginalBase { int value; };
struct OriginalDerived : public OriginalBase { double field; };
enum OriginalWideEnum : unsigned __int128 { ORIGINAL_WIDE_ENUM_VALUE = ((unsigned __int128)1) << 100 };
