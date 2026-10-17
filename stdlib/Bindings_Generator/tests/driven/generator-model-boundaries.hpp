#define MODEL_BOUNDARY_VALUE 17
#define MODEL_BOUNDARY_DEPENDENT (MODEL_BOUNDARY_VALUE + 1)
int boundary_call();
static int rejected_value = boundary_call();
typedef int AcceptedAfter;
typedef int LoggerAccepted;
namespace FilteredBoundary {
    struct Retained { int member; };
    enum RetainedEnum { Value = 3 };
    int excluded_value;
    void excluded_function();
}
struct BoundaryContainer { struct { int payload; } inner; int tail; };
struct BoundaryBits { unsigned first : 3; unsigned second : 5; };
struct BoundaryPolyBits { virtual int step(); unsigned field : 32; int tail; };
struct InvalidCycle;
struct DependentCycle { InvalidCycle *back; };
struct InvalidCycle { DependentCycle *back; int InvalidCycle::* member; };
template<class T> struct BoundaryBox;
template<> struct BoundaryBox<int> { int value; };
struct BoundaryUnsupportedField { BoundaryBox<int> value; int tail; };
struct BoundaryEnumOwner { enum Values { Value = 7 }; int BoundaryEnumOwner::* unsupported; };
void boundary_enum_default(int value = BoundaryEnumOwner::Value);
#ifdef __OBJC__
__attribute__((objc_root_class)) @interface BoundaryObjRoot
- (BoundaryEnumOwner*) owner;
@end
@interface BoundaryObjChild : BoundaryObjRoot
@end
#endif
