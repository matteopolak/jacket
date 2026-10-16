#ifndef RUN_ARGUMENT
#error exact configured arguments are required
#endif
#define RUN_VALUE (RUN_ARGUMENT + 2)
#define RUN_UTF8 "héllo"
#define RUN_BAD (1, 2)
// Original grouping metadata.
#define RUN_GROUP_ONE 3
#define RUN_UNWRAP_ONE 9
#define RUN_PREFIX___A 11
#define RUN_PREFIX___B 12
#define RUN_MIXED_NEGATIVE (-1)
#define RUN_MIXED_UNSIGNED 2ULL
enum RunExisting : unsigned short { RunOriginal = 4 };
#define RUN_EXISTING_ADDED 5
typedef int RunAlias;
namespace EmptyAfter { int hidden; }
namespace FlattenMe { struct Retained { int field; }; }
namespace KeepNamespace { struct Kept { int field; }; }
namespace First_Merged { struct A { int a; }; }
namespace Second_Merged { struct B { int b; }; }
