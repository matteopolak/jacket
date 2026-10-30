// IntroSort: the comparator sorts that Sort is built from, for callers that want a specific algorithm.

intro_sort :: Impl.intro_sort;
// Sorts an array view in place by insertion with a three-way comparator.
//
// It is stable and quadratic in the worst case, but fast for short arrays and for arrays that are nearly
// sorted. The array can be empty. `intro_sort` already uses it for short ranges. The comparator
// has the shape described at `quick_sort`.
insertion_sort :: Impl.insertion_sort;

#scope_file
Impl :: #import "internal/sort";
