// Independent native C++ proof: optional leading global qualification has the
// same source identity; visible Jai namespace merging cannot rename this type.
#include "generator-run-pipeline.hpp"
Second_Merged::B *source_identity(::Second_Merged::B *value) { return value; }
int main() {
    ::Second_Merged::B value{};
    Second_Merged::B *pointer = source_identity(&value);
    return pointer->b;
}
