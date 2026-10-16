struct OriginalRecord { int first; double second; };
enum OriginalEnum { ORIGINAL_FAILED = -7, ORIGINAL_READY = 5 };
typedef struct OriginalRecord OriginalAlias;
extern int original_global;
int original_sum(const struct OriginalRecord *record, int increment);
void original_initialize(struct OriginalRecord *record, int first, double second);
struct OriginalContainer { struct { int payload; } inner; int tail; };
void original_container_initialize(struct OriginalContainer *record, int payload, int tail);
int original_container_read(const struct OriginalContainer *record);
