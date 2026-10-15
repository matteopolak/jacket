// Generated native ABI probe from complete pinned official C source.
#define rpaligned_alloc binding_storage_probe_rpaligned_alloc
#define rpaligned_calloc binding_storage_probe_rpaligned_calloc
#define rpaligned_realloc binding_storage_probe_rpaligned_realloc
#define rpcalloc binding_storage_probe_rpcalloc
#define rpfree binding_storage_probe_rpfree
#define rpmalloc binding_storage_probe_rpmalloc
#define rpmalloc_config binding_storage_probe_rpmalloc_config
#define rpmalloc_dump_statistics binding_storage_probe_rpmalloc_dump_statistics
#define rpmalloc_finalize binding_storage_probe_rpmalloc_finalize
#define rpmalloc_get_heap_for_ptr binding_storage_probe_rpmalloc_get_heap_for_ptr
#define rpmalloc_global_statistics binding_storage_probe_rpmalloc_global_statistics
#define rpmalloc_heap_acquire binding_storage_probe_rpmalloc_heap_acquire
#define rpmalloc_heap_aligned_alloc binding_storage_probe_rpmalloc_heap_aligned_alloc
#define rpmalloc_heap_aligned_calloc binding_storage_probe_rpmalloc_heap_aligned_calloc
#define rpmalloc_heap_aligned_realloc binding_storage_probe_rpmalloc_heap_aligned_realloc
#define rpmalloc_heap_alloc binding_storage_probe_rpmalloc_heap_alloc
#define rpmalloc_heap_calloc binding_storage_probe_rpmalloc_heap_calloc
#define rpmalloc_heap_free binding_storage_probe_rpmalloc_heap_free
#define rpmalloc_heap_free_all binding_storage_probe_rpmalloc_heap_free_all
#define rpmalloc_heap_realloc binding_storage_probe_rpmalloc_heap_realloc
#define rpmalloc_heap_release binding_storage_probe_rpmalloc_heap_release
#define rpmalloc_heap_thread_set_current binding_storage_probe_rpmalloc_heap_thread_set_current
#define rpmalloc_initialize binding_storage_probe_rpmalloc_initialize
#define rpmalloc_initialize_config binding_storage_probe_rpmalloc_initialize_config
#define rpmalloc_is_thread_initialized binding_storage_probe_rpmalloc_is_thread_initialized
#define rpmalloc_linker_reference binding_storage_probe_rpmalloc_linker_reference
#define rpmalloc_set_main_thread binding_storage_probe_rpmalloc_set_main_thread
#define rpmalloc_thread_collect binding_storage_probe_rpmalloc_thread_collect
#define rpmalloc_thread_finalize binding_storage_probe_rpmalloc_thread_finalize
#define rpmalloc_thread_initialize binding_storage_probe_rpmalloc_thread_initialize
#define rpmalloc_thread_statistics binding_storage_probe_rpmalloc_thread_statistics
#define rpmalloc_usable_size binding_storage_probe_rpmalloc_usable_size
#define rpmemalign binding_storage_probe_rpmemalign
#define rpposix_memalign binding_storage_probe_rpposix_memalign
#define rprealloc binding_storage_probe_rprealloc
#include "rpmalloc.c"
long long binding_rpmalloc_native_heap_counter(void* pointer,unsigned family,unsigned index,unsigned field) { heap_t* heap=pointer;
if(family==0 && index<(LARGE_CLASS_COUNT)) { switch(field) {
case 0:return atomic_load32(&heap->span_use[index].current);
case 1:return heap->span_use[index].high;
case 2:return atomic_load32(&heap->span_use[index].spans_deferred);
case 3:return atomic_load32(&heap->span_use[index].spans_to_global);
case 4:return atomic_load32(&heap->span_use[index].spans_from_global);
case 5:return atomic_load32(&heap->span_use[index].spans_to_cache);
case 6:return atomic_load32(&heap->span_use[index].spans_from_cache);
case 7:return atomic_load32(&heap->span_use[index].spans_to_reserved);
case 8:return atomic_load32(&heap->span_use[index].spans_from_reserved);
case 9:return atomic_load32(&heap->span_use[index].spans_map_calls);
} }
if(family==1 && index<(SIZE_CLASS_COUNT+1)) { switch(field) {
case 0:return atomic_load32(&heap->size_class_use[index].alloc_current);
case 1:return heap->size_class_use[index].alloc_peak;
case 2:return atomic_load32(&heap->size_class_use[index].alloc_total);
case 3:return atomic_load32(&heap->size_class_use[index].free_total);
case 4:return atomic_load32(&heap->size_class_use[index].spans_current);
case 5:return heap->size_class_use[index].spans_peak;
case 6:return atomic_load32(&heap->size_class_use[index].spans_to_cache);
case 7:return atomic_load32(&heap->size_class_use[index].spans_from_cache);
case 8:return atomic_load32(&heap->size_class_use[index].spans_from_reserved);
case 9:return atomic_load32(&heap->size_class_use[index].spans_map_calls);
case 10:return heap->size_class_use[index].unused;
} }
abort(); }
#undef rpaligned_alloc
#undef rpaligned_calloc
#undef rpaligned_realloc
#undef rpcalloc
#undef rpfree
#undef rpmalloc
#undef rpmalloc_config
#undef rpmalloc_dump_statistics
#undef rpmalloc_finalize
#undef rpmalloc_get_heap_for_ptr
#undef rpmalloc_global_statistics
#undef rpmalloc_heap_acquire
#undef rpmalloc_heap_aligned_alloc
#undef rpmalloc_heap_aligned_calloc
#undef rpmalloc_heap_aligned_realloc
#undef rpmalloc_heap_alloc
#undef rpmalloc_heap_calloc
#undef rpmalloc_heap_free
#undef rpmalloc_heap_free_all
#undef rpmalloc_heap_realloc
#undef rpmalloc_heap_release
#undef rpmalloc_heap_thread_set_current
#undef rpmalloc_initialize
#undef rpmalloc_initialize_config
#undef rpmalloc_is_thread_initialized
#undef rpmalloc_linker_reference
#undef rpmalloc_set_main_thread
#undef rpmalloc_thread_collect
#undef rpmalloc_thread_finalize
#undef rpmalloc_thread_initialize
#undef rpmalloc_thread_statistics
#undef rpmalloc_usable_size
#undef rpmemalign
#undef rpposix_memalign
#undef rprealloc
#include <stddef.h>
unsigned long long binding_rpmalloc_storage_size(unsigned index) {
    switch(index) {
        case 0: return sizeof(heap_size_class_t);
        case 1: return sizeof(span_cache_t);
        case 2: return sizeof(span_large_cache_t);
        case 3: return sizeof(span_use_t);
        case 4: return sizeof(size_class_use_t);
        case 5: return sizeof(span_t);
        case 6: return sizeof(heap_t);
        default: __builtin_trap();
    }
}
unsigned long long binding_rpmalloc_storage_offset(unsigned index) {
    switch(index) {
        case 0: return offsetof(heap_size_class_t, free_list);
        case 1: return offsetof(heap_size_class_t, partial_span);
        case 2: return offsetof(heap_size_class_t, cache);
        case 3: return offsetof(span_cache_t, count);
        case 4: return offsetof(span_cache_t, span);
        case 5: return offsetof(span_large_cache_t, count);
        case 6: return offsetof(span_large_cache_t, span);
        case 7: return offsetof(span_use_t, current);
        case 8: return offsetof(span_use_t, high);
        case 9: return offsetof(span_use_t, spans_deferred);
        case 10: return offsetof(span_use_t, spans_to_global);
        case 11: return offsetof(span_use_t, spans_from_global);
        case 12: return offsetof(span_use_t, spans_to_cache);
        case 13: return offsetof(span_use_t, spans_from_cache);
        case 14: return offsetof(span_use_t, spans_to_reserved);
        case 15: return offsetof(span_use_t, spans_from_reserved);
        case 16: return offsetof(span_use_t, spans_map_calls);
        case 17: return offsetof(size_class_use_t, alloc_current);
        case 18: return offsetof(size_class_use_t, alloc_peak);
        case 19: return offsetof(size_class_use_t, alloc_total);
        case 20: return offsetof(size_class_use_t, free_total);
        case 21: return offsetof(size_class_use_t, spans_current);
        case 22: return offsetof(size_class_use_t, spans_peak);
        case 23: return offsetof(size_class_use_t, spans_to_cache);
        case 24: return offsetof(size_class_use_t, spans_from_cache);
        case 25: return offsetof(size_class_use_t, spans_from_reserved);
        case 26: return offsetof(size_class_use_t, spans_map_calls);
        case 27: return offsetof(size_class_use_t, unused);
        case 28: return offsetof(span_t, free_list);
        case 29: return offsetof(span_t, block_count);
        case 30: return offsetof(span_t, size_class);
        case 31: return offsetof(span_t, free_list_limit);
        case 32: return offsetof(span_t, used_count);
        case 33: return offsetof(span_t, free_list_deferred);
        case 34: return offsetof(span_t, list_size);
        case 35: return offsetof(span_t, block_size);
        case 36: return offsetof(span_t, flags);
        case 37: return offsetof(span_t, span_count);
        case 38: return offsetof(span_t, total_spans);
        case 39: return offsetof(span_t, offset_from_master);
        case 40: return offsetof(span_t, remaining_spans);
        case 41: return offsetof(span_t, align_offset);
        case 42: return offsetof(span_t, heap);
        case 43: return offsetof(span_t, next);
        case 44: return offsetof(span_t, prev);
        case 45: return offsetof(heap_t, owner_thread);
        case 46: return offsetof(heap_t, size_class);
        case 47: return offsetof(heap_t, span_cache);
        case 48: return offsetof(heap_t, span_free_deferred);
        case 49: return offsetof(heap_t, full_span_count);
        case 50: return offsetof(heap_t, span_reserve);
        case 51: return offsetof(heap_t, span_reserve_master);
        case 52: return offsetof(heap_t, spans_reserved);
        case 53: return offsetof(heap_t, child_count);
        case 54: return offsetof(heap_t, next_heap);
        case 55: return offsetof(heap_t, next_orphan);
        case 56: return offsetof(heap_t, id);
        case 57: return offsetof(heap_t, finalize);
        case 58: return offsetof(heap_t, master_heap);
        case 59: return offsetof(heap_t, span_large_cache);
        case 60: return offsetof(heap_t, full_span);
        case 61: return offsetof(heap_t, large_huge_span);
        case 62: return offsetof(heap_t, span_use);
        case 63: return offsetof(heap_t, size_class_use);
        case 64: return offsetof(heap_t, thread_to_global);
        case 65: return offsetof(heap_t, global_to_thread);
        default: __builtin_trap();
    }
}
unsigned long long binding_rpmalloc_storage_field_size(unsigned index) {
    switch(index) {
        case 0: return sizeof(((heap_size_class_t*)0)->free_list);
        case 1: return sizeof(((heap_size_class_t*)0)->partial_span);
        case 2: return sizeof(((heap_size_class_t*)0)->cache);
        case 3: return sizeof(((span_cache_t*)0)->count);
        case 4: return sizeof(((span_cache_t*)0)->span);
        case 5: return sizeof(((span_large_cache_t*)0)->count);
        case 6: return sizeof(((span_large_cache_t*)0)->span);
        case 7: return sizeof(((span_use_t*)0)->current);
        case 8: return sizeof(((span_use_t*)0)->high);
        case 9: return sizeof(((span_use_t*)0)->spans_deferred);
        case 10: return sizeof(((span_use_t*)0)->spans_to_global);
        case 11: return sizeof(((span_use_t*)0)->spans_from_global);
        case 12: return sizeof(((span_use_t*)0)->spans_to_cache);
        case 13: return sizeof(((span_use_t*)0)->spans_from_cache);
        case 14: return sizeof(((span_use_t*)0)->spans_to_reserved);
        case 15: return sizeof(((span_use_t*)0)->spans_from_reserved);
        case 16: return sizeof(((span_use_t*)0)->spans_map_calls);
        case 17: return sizeof(((size_class_use_t*)0)->alloc_current);
        case 18: return sizeof(((size_class_use_t*)0)->alloc_peak);
        case 19: return sizeof(((size_class_use_t*)0)->alloc_total);
        case 20: return sizeof(((size_class_use_t*)0)->free_total);
        case 21: return sizeof(((size_class_use_t*)0)->spans_current);
        case 22: return sizeof(((size_class_use_t*)0)->spans_peak);
        case 23: return sizeof(((size_class_use_t*)0)->spans_to_cache);
        case 24: return sizeof(((size_class_use_t*)0)->spans_from_cache);
        case 25: return sizeof(((size_class_use_t*)0)->spans_from_reserved);
        case 26: return sizeof(((size_class_use_t*)0)->spans_map_calls);
        case 27: return sizeof(((size_class_use_t*)0)->unused);
        case 28: return sizeof(((span_t*)0)->free_list);
        case 29: return sizeof(((span_t*)0)->block_count);
        case 30: return sizeof(((span_t*)0)->size_class);
        case 31: return sizeof(((span_t*)0)->free_list_limit);
        case 32: return sizeof(((span_t*)0)->used_count);
        case 33: return sizeof(((span_t*)0)->free_list_deferred);
        case 34: return sizeof(((span_t*)0)->list_size);
        case 35: return sizeof(((span_t*)0)->block_size);
        case 36: return sizeof(((span_t*)0)->flags);
        case 37: return sizeof(((span_t*)0)->span_count);
        case 38: return sizeof(((span_t*)0)->total_spans);
        case 39: return sizeof(((span_t*)0)->offset_from_master);
        case 40: return sizeof(((span_t*)0)->remaining_spans);
        case 41: return sizeof(((span_t*)0)->align_offset);
        case 42: return sizeof(((span_t*)0)->heap);
        case 43: return sizeof(((span_t*)0)->next);
        case 44: return sizeof(((span_t*)0)->prev);
        case 45: return sizeof(((heap_t*)0)->owner_thread);
        case 46: return sizeof(((heap_t*)0)->size_class);
        case 47: return sizeof(((heap_t*)0)->span_cache);
        case 48: return sizeof(((heap_t*)0)->span_free_deferred);
        case 49: return sizeof(((heap_t*)0)->full_span_count);
        case 50: return sizeof(((heap_t*)0)->span_reserve);
        case 51: return sizeof(((heap_t*)0)->span_reserve_master);
        case 52: return sizeof(((heap_t*)0)->spans_reserved);
        case 53: return sizeof(((heap_t*)0)->child_count);
        case 54: return sizeof(((heap_t*)0)->next_heap);
        case 55: return sizeof(((heap_t*)0)->next_orphan);
        case 56: return sizeof(((heap_t*)0)->id);
        case 57: return sizeof(((heap_t*)0)->finalize);
        case 58: return sizeof(((heap_t*)0)->master_heap);
        case 59: return sizeof(((heap_t*)0)->span_large_cache);
        case 60: return sizeof(((heap_t*)0)->full_span);
        case 61: return sizeof(((heap_t*)0)->large_huge_span);
        case 62: return sizeof(((heap_t*)0)->span_use);
        case 63: return sizeof(((heap_t*)0)->size_class_use);
        case 64: return sizeof(((heap_t*)0)->thread_to_global);
        case 65: return sizeof(((heap_t*)0)->global_to_thread);
        default: __builtin_trap();
    }
}
long long binding_rpmalloc_native_heap_id(const void *heap) { return ((const heap_t*)heap)->id; }
unsigned long long binding_rpmalloc_native_heap_owner(const void *heap) { return ((const heap_t*)heap)->owner_thread; }
long long binding_rpmalloc_public_atomic32(void *value) { return atomic_load32((atomic32_t*)value); }
