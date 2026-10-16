#pragma once
#pragma clang system_header
typedef int RunHidden;
struct RunIncluded { int value; };
#define RUN_SYSTEM 5
struct RunRetainedSystem { int retained_member; };
namespace SystemNamespace {
struct NestedType { int nested_member; };
int suppressed;
}
