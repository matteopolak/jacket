struct OriginalAggregate { int count; double weight; };
struct OriginalNestedAggregate { OriginalAggregate child; };
OriginalAggregate original_aggregate = { 4, 2.5 };
OriginalNestedAggregate original_nested = { { 7, 3.5 } };
OriginalAggregate original_empty = {};
void original_default(const OriginalAggregate &value = OriginalAggregate{ 9, 4.5 });
