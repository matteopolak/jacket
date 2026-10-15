#ifndef INDEPENDENT_MULTI_BASE_H
#define INDEPENDENT_MULTI_BASE_H
typedef struct MultiRecord { int value; } MultiRecord;
#define MULTI_SCALE 3
int multi_base(const MultiRecord *record);
#endif
