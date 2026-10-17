#ifndef ORIGINAL_ANONYMOUS_TYPES_H
#define ORIGINAL_ANONYMOUS_TYPES_H
typedef struct { int value; double weight; } Anonymous_Record;
typedef union { unsigned long long bits; double number; } Anonymous_Union;
typedef enum { ORIGINAL_MODE_NEGATIVE = -3, ORIGINAL_MODE_POSITIVE = 7 } Anonymous_Mode;
typedef Anonymous_Record Record_Alias;
typedef Anonymous_Mode Mode_Alias;
typedef struct Forward_Record Forward_Record;
struct Forward_Record { int count; };
int original_anonymous(Anonymous_Record *record, Anonymous_Union *value, Anonymous_Mode mode);
int original_alias(Record_Alias *record, Mode_Alias mode, Forward_Record *forward);
#endif
