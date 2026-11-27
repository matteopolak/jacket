// A library header that declares a variable of a record it never defines, as LVGL's label header
// does for `lv_label_class`. The record's body lives in a private header.
typedef struct Hidden Hidden;
struct Hidden *hidden_create(void);
extern const Hidden hidden_class;
extern struct Hidden hidden_tag_class;
extern const Hidden hidden_table[2];
extern struct Hidden *hidden_pointer;
extern const Hidden *hidden_table_pointer[2];
extern int visible_count;
