enum OriginalStatus { ORIGINAL_FAILED = -7, ORIGINAL_READY = 5 };
_Static_assert(sizeof(enum OriginalStatus) == 4, "fixture enum requires 32-bit storage");
int original_enum_value(enum OriginalStatus status) { return status; }
