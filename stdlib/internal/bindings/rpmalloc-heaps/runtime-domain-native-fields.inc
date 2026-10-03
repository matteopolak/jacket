/* Generated complete pinned SDK globals; preserve conditional fields. */
//! Initialized flag
int sdk_rpmalloc_initialized;
//! Main thread ID
uintptr_t sdk_rpmalloc_main_thread_id;
//! Configuration
rpmalloc_config_t sdk_memory_config;
//! Memory page size
size_t sdk_memory_page_size;
//! Shift to divide by page size
size_t sdk_memory_page_size_shift;
//! Granularity at which memory pages are mapped by OS
size_t sdk_memory_map_granularity;
#if RPMALLOC_CONFIGURABLE
//! Size of a span of memory pages
size_t sdk_memory_span_size;
//! Shift to divide by span size
size_t sdk_memory_span_size_shift;
//! Mask to get to start of a memory span
uintptr_t sdk_memory_span_mask;
#else
//! Hardwired span size
#define sdk_memory_span_size _memory_default_span_size
#define sdk_memory_span_size_shift _memory_default_span_size_shift
#define sdk_memory_span_mask _memory_default_span_mask
#endif
//! Number of spans to map in each map call
size_t sdk_memory_span_map_count;
//! Number of spans to keep reserved in each heap
size_t sdk_memory_heap_reserve_count;
//! Global size classes
size_class_t sdk_memory_size_class[SIZE_CLASS_COUNT];
//! Run-time size limit of medium blocks
size_t sdk_memory_medium_size_limit;
//! Heap ID counter
atomic32_t sdk_memory_heap_id;
//! Huge page support
int sdk_memory_huge_pages;
#if ENABLE_GLOBAL_CACHE
//! Global span cache
global_cache_t sdk_memory_span_cache[LARGE_CLASS_COUNT];
#endif
//! Global reserved spans
span_t* sdk_memory_global_reserve;
//! Global reserved count
size_t sdk_memory_global_reserve_count;
//! Global reserved master
span_t* sdk_memory_global_reserve_master;
//! All heaps
#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION
heap_t** sdk_memory_heaps;
#else
heap_t* sdk_memory_heaps[HEAP_ARRAY_SIZE];
#endif
//! Used to restrict access to mapping memory for huge pages
atomic32_t sdk_memory_global_lock;
//! Orphaned heaps
heap_t* sdk_memory_orphan_heaps;
#if RPMALLOC_FIRST_CLASS_HEAPS
//! Orphaned heaps (first class heaps)
heap_t* sdk_memory_first_class_orphan_heaps;
#endif
#if ENABLE_STATISTICS
//! Allocations counter
atomic64_t sdk_allocation_counter;
//! Deallocations counter
atomic64_t sdk_deallocation_counter;
//! Active heap count
atomic32_t sdk_memory_active_heaps;
//! Number of currently mapped memory pages
atomic32_t sdk_mapped_pages;
//! Peak number of concurrently mapped memory pages
int32_t sdk_mapped_pages_peak;
//! Number of mapped master spans
atomic32_t sdk_master_spans;
//! Number of unmapped dangling master spans
atomic32_t sdk_unmapped_master_spans;
//! Running counter of total number of mapped memory pages since start
atomic32_t sdk_mapped_total;
//! Running counter of total number of unmapped memory pages since start
atomic32_t sdk_unmapped_total;
//! Number of currently mapped memory pages in OS calls
atomic32_t sdk_mapped_pages_os;
//! Number of currently allocated pages in huge allocations
atomic32_t sdk_huge_pages_current;
//! Peak number of currently allocated pages in huge allocations
int32_t sdk_huge_pages_peak;
#endif

#if defined(_WIN32) && (!defined(BUILD_DYNAMIC_LINK) || !BUILD_DYNAMIC_LINK)
DWORD sdk_fls_key;
#endif
