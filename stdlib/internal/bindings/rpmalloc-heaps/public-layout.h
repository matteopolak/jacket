#if BINDING_RPMALLOC_RAW
/* Generated reviewed public layouts, independently organized. */
#include <stdint.h>
#include <stddef.h>
typedef struct Public_Heap_Size_Class Public_Heap_Size_Class;
typedef struct Public_Span_Cache Public_Span_Cache;
typedef struct Public_Span_Large_Cache Public_Span_Large_Cache;
typedef struct Public_Span_Use Public_Span_Use;
typedef struct Public_Size_Class_Use Public_Size_Class_Use;
typedef struct Public_Span Public_Span;
typedef struct Public_Heap Public_Heap;
#if defined(_MSC_VER)
typedef volatile int32_t Public_Atomic32;
typedef volatile int64_t Public_Atomic64;
typedef volatile void* Public_AtomicPtr;
#else
typedef volatile _Atomic(int32_t) Public_Atomic32;
typedef volatile _Atomic(int64_t) Public_Atomic64;
typedef volatile _Atomic(void*) Public_AtomicPtr;
#endif
struct Public_Heap_Size_Class {
void *free_list;
Public_Span *partial_span;
Public_Span *cache;
};
struct Public_Span_Cache {
uint64_t count;
Public_Span *span[400];
};
struct Public_Span_Large_Cache {
uint64_t count;
Public_Span *span[100];
};
struct Public_Span_Use {
Public_Atomic32 current;
int32_t high;
#if ENABLE_STATISTICS
Public_Atomic32 spans_deferred;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_to_global;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_from_global;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_to_cache;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_from_cache;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_to_reserved;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_from_reserved;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_map_calls;
#endif
};
#if ENABLE_STATISTICS
struct Public_Size_Class_Use {
#if ENABLE_STATISTICS
Public_Atomic32 alloc_current;
#endif
#if ENABLE_STATISTICS
int32_t alloc_peak;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 alloc_total;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 free_total;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_current;
#endif
#if ENABLE_STATISTICS
int32_t spans_peak;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_to_cache;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_from_cache;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_from_reserved;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 spans_map_calls;
#endif
#if ENABLE_STATISTICS
Public_Atomic32 unused;
#endif
};
#endif
struct Public_Span {
void *free_list;
uint32_t block_count;
uint32_t size_class;
uint32_t free_list_limit;
uint32_t used_count;
uint32_t list_size;
uint32_t block_size;
uint32_t span_count;
uint32_t total_spans;
uint32_t offset_from_master;
Public_Atomic32 remaining_spans;
uint32_t align_offset;
Public_AtomicPtr free_list_deferred;
uint32_t flags;
Public_Heap *heap;
Public_Span *next;
Public_Span *prev;
};
struct Public_Heap {
uint64_t owner_thread;
Public_Heap_Size_Class size_class[126];
Public_AtomicPtr span_free_deferred;
uint64_t full_span_count;
Public_Span *span_reserve;
Public_Span *span_reserve_master;
uint32_t spans_reserved;
Public_Atomic32 child_count;
Public_Heap *next_heap;
Public_Heap *next_orphan;
int32_t id;
int32_t finalize;
Public_Heap *master_heap;
#if ENABLE_THREAD_CACHE
Public_Span_Cache span_cache;
#endif
#if ENABLE_THREAD_CACHE
Public_Span_Large_Cache span_large_cache[62];
#endif
#if BINDING_RPMALLOC_PUBLIC_HEAPS
Public_Span *full_span[126];
#endif
#if BINDING_RPMALLOC_PUBLIC_HEAPS
Public_Span *large_huge_span;
#endif
#if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS
Public_Span_Use span_use[63];
#endif
#if ENABLE_STATISTICS
Public_Size_Class_Use size_class_use[127];
#endif
#if ENABLE_STATISTICS
Public_Atomic64 thread_to_global;
#endif
#if ENABLE_STATISTICS
Public_Atomic64 global_to_thread;
#endif
};
_Static_assert(sizeof(Public_Span)==96,"reviewed public span");
#else
/* Generated reviewed public layouts, independently organized. */
#include <stdint.h>
#include <stddef.h>
typedef struct Public_Heap_Size_Class Public_Heap_Size_Class;
typedef struct Public_Span_Cache Public_Span_Cache;
typedef struct Public_Span_Large_Cache Public_Span_Large_Cache;
typedef struct Public_Span_Use Public_Span_Use;
typedef struct Public_Size_Class_Use Public_Size_Class_Use;
typedef struct Public_Span Public_Span;
typedef struct Public_Heap Public_Heap;
struct Public_Heap_Size_Class {
void *free_list;
Public_Span *partial_span;
Public_Span *cache;
};
struct Public_Span_Cache {
int64_t count;
Public_Span *span[400];
};
struct Public_Span_Large_Cache {
int64_t count;
Public_Span *span[100];
};
struct Public_Span_Use {
int32_t current;
int32_t high;
#if ENABLE_STATISTICS
int32_t spans_deferred;
#endif
#if ENABLE_STATISTICS
int32_t spans_to_global;
#endif
#if ENABLE_STATISTICS
int32_t spans_from_global;
#endif
#if ENABLE_STATISTICS
int32_t spans_to_cache;
#endif
#if ENABLE_STATISTICS
int32_t spans_from_cache;
#endif
#if ENABLE_STATISTICS
int32_t spans_to_reserved;
#endif
#if ENABLE_STATISTICS
int32_t spans_from_reserved;
#endif
#if ENABLE_STATISTICS
int32_t spans_map_calls;
#endif
};
#if ENABLE_STATISTICS
struct Public_Size_Class_Use {
#if ENABLE_STATISTICS
int32_t alloc_current;
#endif
#if ENABLE_STATISTICS
int32_t alloc_peak;
#endif
#if ENABLE_STATISTICS
int32_t alloc_total;
#endif
#if ENABLE_STATISTICS
int32_t free_total;
#endif
#if ENABLE_STATISTICS
int32_t spans_current;
#endif
#if ENABLE_STATISTICS
int32_t spans_peak;
#endif
#if ENABLE_STATISTICS
int32_t spans_to_cache;
#endif
#if ENABLE_STATISTICS
int32_t spans_from_cache;
#endif
#if ENABLE_STATISTICS
int32_t spans_from_reserved;
#endif
#if ENABLE_STATISTICS
int32_t spans_map_calls;
#endif
#if ENABLE_STATISTICS
int32_t unused;
#endif
};
#endif
struct Public_Span {
void *free_list;
uint32_t block_count;
uint32_t size_class;
uint32_t free_list_limit;
uint32_t used_count;
uint32_t list_size;
uint32_t block_size;
uint32_t span_count;
uint32_t total_spans;
uint32_t offset_from_master;
int32_t remaining_spans;
uint32_t align_offset;
void *free_list_deferred;
uint32_t flags;
Public_Heap *heap;
Public_Span *next;
Public_Span *prev;
};
struct Public_Heap {
uint64_t owner_thread;
Public_Heap_Size_Class size_class[126];
void *span_free_deferred;
int64_t full_span_count;
Public_Span *span_reserve;
Public_Span *span_reserve_master;
uint32_t spans_reserved;
int32_t child_count;
Public_Heap *next_heap;
Public_Heap *next_orphan;
int32_t id;
int32_t finalize;
Public_Heap *master_heap;
Public_Span *full_span[126];
Public_Span *large_huge_span;
#if ENABLE_THREAD_CACHE
Public_Span_Cache span_cache;
#endif
#if ENABLE_THREAD_CACHE
Public_Span_Large_Cache span_large_cache[62];
#endif
#if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS
Public_Span_Use span_use[63];
#endif
#if ENABLE_STATISTICS
Public_Size_Class_Use size_class_use[127];
#endif
#if ENABLE_STATISTICS
int64_t thread_to_global;
#endif
#if ENABLE_STATISTICS
int64_t global_to_thread;
#endif
};
_Static_assert(sizeof(Public_Span)==96,"reviewed public span");
#endif
