#include "developer-tools-support.h"
int64_t support_sum(JaiSupport_User *value) {
  return value->text.count + value->view.count + value->dynamic.count +
         *(s32 *)value->boxed.value_pointer + (value->type != NULL);
}
