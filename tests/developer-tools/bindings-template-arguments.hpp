struct OriginalTemplateType { int field; };
template<typename T, int N> struct OriginalTemplateBox { T values[N]; };
typedef OriginalTemplateBox<OriginalTemplateType, 3> OriginalApplication;
