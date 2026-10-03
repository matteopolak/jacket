#ifndef BINDING_RPMALLOC_CALLBACK_STATE_H
#define BINDING_RPMALLOC_CALLBACK_STATE_H
#include <rpmalloc.h>
#include <stdatomic.h>
typedef struct BindingMap {
  void *address;
  size_t size;
  size_t offset;
  struct BindingMap *next;
} BindingMap;
typedef struct BindingCallbackState {
  rpmalloc_config_t config;
  BindingMap *maps;
  atomic_flag map_lock;
  char *page_name;
  char *huge_page_name;
  int active;
} BindingCallbackState;
#define BINDING_JOIN_INNER(a, b) a##b
#define BINDING_JOIN(a, b) BINDING_JOIN_INNER(a, b)
BindingCallbackState *BINDING_JOIN(BINDING_RPMALLOC_FACADE_PREFIX,
                                   callback_storage)(void);
#endif
