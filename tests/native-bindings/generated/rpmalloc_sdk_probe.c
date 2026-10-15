#include "rpmalloc.h"
#include <stddef.h>
_Static_assert(sizeof(void*) == 8 && sizeof(size_t) == 8 && sizeof(int) == 4, "64-bit C ABI");
unsigned long long binding_rpmalloc_size(unsigned index) {
    switch(index) {
        case 0: return sizeof(rpmalloc_config_t);
        case 1: return sizeof(rpmalloc_global_statistics_t);
        case 2: return sizeof(rpmalloc_thread_statistics_t);
        default: __builtin_trap();
    }
}
unsigned long long binding_rpmalloc_offset(unsigned index) {
    switch(index) {
        case 0: return offsetof(rpmalloc_config_t, memory_map);
        case 1: return offsetof(rpmalloc_config_t, memory_unmap);
        case 2: return offsetof(rpmalloc_config_t, error_callback);
        case 3: return offsetof(rpmalloc_config_t, map_fail_callback);
        case 4: return offsetof(rpmalloc_config_t, page_size);
        case 5: return offsetof(rpmalloc_config_t, span_size);
        case 6: return offsetof(rpmalloc_config_t, span_map_count);
        case 7: return offsetof(rpmalloc_config_t, enable_huge_pages);
        case 8: return offsetof(rpmalloc_config_t, page_name);
        case 9: return offsetof(rpmalloc_config_t, huge_page_name);
        case 10: return offsetof(rpmalloc_global_statistics_t, mapped);
        case 11: return offsetof(rpmalloc_global_statistics_t, mapped_peak);
        case 12: return offsetof(rpmalloc_global_statistics_t, cached);
        case 13: return offsetof(rpmalloc_global_statistics_t, huge_alloc);
        case 14: return offsetof(rpmalloc_global_statistics_t, huge_alloc_peak);
        case 15: return offsetof(rpmalloc_global_statistics_t, mapped_total);
        case 16: return offsetof(rpmalloc_global_statistics_t, unmapped_total);
        case 17: return offsetof(rpmalloc_thread_statistics_t, sizecache);
        case 18: return offsetof(rpmalloc_thread_statistics_t, spancache);
        case 19: return offsetof(rpmalloc_thread_statistics_t, thread_to_global);
        case 20: return offsetof(rpmalloc_thread_statistics_t, global_to_thread);
        case 21: return offsetof(rpmalloc_thread_statistics_t, span_use);
        case 22: return offsetof(rpmalloc_thread_statistics_t, span_use[0].current);
        case 23: return offsetof(rpmalloc_thread_statistics_t, span_use[0].peak);
        case 24: return offsetof(rpmalloc_thread_statistics_t, span_use[0].to_global);
        case 25: return offsetof(rpmalloc_thread_statistics_t, span_use[0].from_global);
        case 26: return offsetof(rpmalloc_thread_statistics_t, span_use[0].to_cache);
        case 27: return offsetof(rpmalloc_thread_statistics_t, span_use[0].from_cache);
        case 28: return offsetof(rpmalloc_thread_statistics_t, span_use[0].to_reserved);
        case 29: return offsetof(rpmalloc_thread_statistics_t, span_use[0].from_reserved);
        case 30: return offsetof(rpmalloc_thread_statistics_t, span_use[0].map_calls);
        case 31: return offsetof(rpmalloc_thread_statistics_t, size_use);
        case 32: return offsetof(rpmalloc_thread_statistics_t, size_use[0].alloc_current);
        case 33: return offsetof(rpmalloc_thread_statistics_t, size_use[0].alloc_peak);
        case 34: return offsetof(rpmalloc_thread_statistics_t, size_use[0].alloc_total);
        case 35: return offsetof(rpmalloc_thread_statistics_t, size_use[0].free_total);
        case 36: return offsetof(rpmalloc_thread_statistics_t, size_use[0].spans_to_cache);
        case 37: return offsetof(rpmalloc_thread_statistics_t, size_use[0].spans_from_cache);
        case 38: return offsetof(rpmalloc_thread_statistics_t, size_use[0].spans_from_reserved);
        case 39: return offsetof(rpmalloc_thread_statistics_t, size_use[0].map_calls);
        default: __builtin_trap();
    }
}
unsigned long long binding_rpmalloc_field_size(unsigned index) {
    switch(index) {
        case 0: return sizeof(((rpmalloc_config_t*)0)->memory_map);
        case 1: return sizeof(((rpmalloc_config_t*)0)->memory_unmap);
        case 2: return sizeof(((rpmalloc_config_t*)0)->error_callback);
        case 3: return sizeof(((rpmalloc_config_t*)0)->map_fail_callback);
        case 4: return sizeof(((rpmalloc_config_t*)0)->page_size);
        case 5: return sizeof(((rpmalloc_config_t*)0)->span_size);
        case 6: return sizeof(((rpmalloc_config_t*)0)->span_map_count);
        case 7: return sizeof(((rpmalloc_config_t*)0)->enable_huge_pages);
        case 8: return sizeof(((rpmalloc_config_t*)0)->page_name);
        case 9: return sizeof(((rpmalloc_config_t*)0)->huge_page_name);
        case 10: return sizeof(((rpmalloc_global_statistics_t*)0)->mapped);
        case 11: return sizeof(((rpmalloc_global_statistics_t*)0)->mapped_peak);
        case 12: return sizeof(((rpmalloc_global_statistics_t*)0)->cached);
        case 13: return sizeof(((rpmalloc_global_statistics_t*)0)->huge_alloc);
        case 14: return sizeof(((rpmalloc_global_statistics_t*)0)->huge_alloc_peak);
        case 15: return sizeof(((rpmalloc_global_statistics_t*)0)->mapped_total);
        case 16: return sizeof(((rpmalloc_global_statistics_t*)0)->unmapped_total);
        case 17: return sizeof(((rpmalloc_thread_statistics_t*)0)->sizecache);
        case 18: return sizeof(((rpmalloc_thread_statistics_t*)0)->spancache);
        case 19: return sizeof(((rpmalloc_thread_statistics_t*)0)->thread_to_global);
        case 20: return sizeof(((rpmalloc_thread_statistics_t*)0)->global_to_thread);
        case 21: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use);
        case 22: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].current);
        case 23: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].peak);
        case 24: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].to_global);
        case 25: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].from_global);
        case 26: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].to_cache);
        case 27: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].from_cache);
        case 28: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].to_reserved);
        case 29: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].from_reserved);
        case 30: return sizeof(((rpmalloc_thread_statistics_t*)0)->span_use[0].map_calls);
        case 31: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use);
        case 32: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].alloc_current);
        case 33: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].alloc_peak);
        case 34: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].alloc_total);
        case 35: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].free_total);
        case 36: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].spans_to_cache);
        case 37: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].spans_from_cache);
        case 38: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].spans_from_reserved);
        case 39: return sizeof(((rpmalloc_thread_statistics_t*)0)->size_use[0].map_calls);
        default: __builtin_trap();
    }
}
