#if BINDING_RPMALLOC_RAW
/* Generated reviewed public C mirror layout measurements. */
uint64_t API(layout_size)(unsigned index) { switch(index) {
case 0:return sizeof(Public_Heap_Size_Class);
case 1:return sizeof(Public_Span_Cache);
case 2:return sizeof(Public_Span_Large_Cache);
case 3:return sizeof(Public_Span_Use);
#if ENABLE_STATISTICS
case 4:return sizeof(Public_Size_Class_Use);
#endif
case 5:return sizeof(Public_Span);
case 6:return sizeof(Public_Heap);
default: abort(); } }
uint64_t API(layout_offset)(unsigned index) { switch(index) {
case 0:return offsetof(Public_Heap_Size_Class,free_list);
case 1:return offsetof(Public_Heap_Size_Class,partial_span);
case 2:return offsetof(Public_Heap_Size_Class,cache);
case 3:return offsetof(Public_Span_Cache,count);
case 4:return offsetof(Public_Span_Cache,span);
case 5:return offsetof(Public_Span_Large_Cache,count);
case 6:return offsetof(Public_Span_Large_Cache,span);
case 7:return offsetof(Public_Span_Use,current);
case 8:return offsetof(Public_Span_Use,high);
#if ENABLE_STATISTICS
case 9:return offsetof(Public_Span_Use,spans_deferred);
#endif
#if ENABLE_STATISTICS
case 10:return offsetof(Public_Span_Use,spans_to_global);
#endif
#if ENABLE_STATISTICS
case 11:return offsetof(Public_Span_Use,spans_from_global);
#endif
#if ENABLE_STATISTICS
case 12:return offsetof(Public_Span_Use,spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 13:return offsetof(Public_Span_Use,spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 14:return offsetof(Public_Span_Use,spans_to_reserved);
#endif
#if ENABLE_STATISTICS
case 15:return offsetof(Public_Span_Use,spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 16:return offsetof(Public_Span_Use,spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 17:return offsetof(Public_Size_Class_Use,alloc_current);
#endif
#if ENABLE_STATISTICS
case 18:return offsetof(Public_Size_Class_Use,alloc_peak);
#endif
#if ENABLE_STATISTICS
case 19:return offsetof(Public_Size_Class_Use,alloc_total);
#endif
#if ENABLE_STATISTICS
case 20:return offsetof(Public_Size_Class_Use,free_total);
#endif
#if ENABLE_STATISTICS
case 21:return offsetof(Public_Size_Class_Use,spans_current);
#endif
#if ENABLE_STATISTICS
case 22:return offsetof(Public_Size_Class_Use,spans_peak);
#endif
#if ENABLE_STATISTICS
case 23:return offsetof(Public_Size_Class_Use,spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 24:return offsetof(Public_Size_Class_Use,spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 25:return offsetof(Public_Size_Class_Use,spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 26:return offsetof(Public_Size_Class_Use,spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 27:return offsetof(Public_Size_Class_Use,unused);
#endif
case 28:return offsetof(Public_Span,free_list);
case 29:return offsetof(Public_Span,block_count);
case 30:return offsetof(Public_Span,size_class);
case 31:return offsetof(Public_Span,free_list_limit);
case 32:return offsetof(Public_Span,used_count);
case 33:return offsetof(Public_Span,list_size);
case 34:return offsetof(Public_Span,block_size);
case 35:return offsetof(Public_Span,span_count);
case 36:return offsetof(Public_Span,total_spans);
case 37:return offsetof(Public_Span,offset_from_master);
case 38:return offsetof(Public_Span,remaining_spans);
case 39:return offsetof(Public_Span,align_offset);
case 40:return offsetof(Public_Span,free_list_deferred);
case 41:return offsetof(Public_Span,flags);
case 42:return offsetof(Public_Span,heap);
case 43:return offsetof(Public_Span,next);
case 44:return offsetof(Public_Span,prev);
case 45:return offsetof(Public_Heap,owner_thread);
case 46:return offsetof(Public_Heap,size_class);
case 47:return offsetof(Public_Heap,span_free_deferred);
case 48:return offsetof(Public_Heap,full_span_count);
case 49:return offsetof(Public_Heap,span_reserve);
case 50:return offsetof(Public_Heap,span_reserve_master);
case 51:return offsetof(Public_Heap,spans_reserved);
case 52:return offsetof(Public_Heap,child_count);
case 53:return offsetof(Public_Heap,next_heap);
case 54:return offsetof(Public_Heap,next_orphan);
case 55:return offsetof(Public_Heap,id);
case 56:return offsetof(Public_Heap,finalize);
case 57:return offsetof(Public_Heap,master_heap);
#if ENABLE_THREAD_CACHE
case 58:return offsetof(Public_Heap,span_cache);
#endif
#if ENABLE_THREAD_CACHE
case 59:return offsetof(Public_Heap,span_large_cache);
#endif
#if BINDING_RPMALLOC_PUBLIC_HEAPS
case 60:return offsetof(Public_Heap,full_span);
#endif
#if BINDING_RPMALLOC_PUBLIC_HEAPS
case 61:return offsetof(Public_Heap,large_huge_span);
#endif
#if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS
case 62:return offsetof(Public_Heap,span_use);
#endif
#if ENABLE_STATISTICS
case 63:return offsetof(Public_Heap,size_class_use);
#endif
#if ENABLE_STATISTICS
case 64:return offsetof(Public_Heap,thread_to_global);
#endif
#if ENABLE_STATISTICS
case 65:return offsetof(Public_Heap,global_to_thread);
#endif
default: abort(); } }
uint64_t API(layout_width)(unsigned index) { switch(index) {
case 0:return sizeof(((Public_Heap_Size_Class*)0)->free_list);
case 1:return sizeof(((Public_Heap_Size_Class*)0)->partial_span);
case 2:return sizeof(((Public_Heap_Size_Class*)0)->cache);
case 3:return sizeof(((Public_Span_Cache*)0)->count);
case 4:return sizeof(((Public_Span_Cache*)0)->span);
case 5:return sizeof(((Public_Span_Large_Cache*)0)->count);
case 6:return sizeof(((Public_Span_Large_Cache*)0)->span);
case 7:return sizeof(((Public_Span_Use*)0)->current);
case 8:return sizeof(((Public_Span_Use*)0)->high);
#if ENABLE_STATISTICS
case 9:return sizeof(((Public_Span_Use*)0)->spans_deferred);
#endif
#if ENABLE_STATISTICS
case 10:return sizeof(((Public_Span_Use*)0)->spans_to_global);
#endif
#if ENABLE_STATISTICS
case 11:return sizeof(((Public_Span_Use*)0)->spans_from_global);
#endif
#if ENABLE_STATISTICS
case 12:return sizeof(((Public_Span_Use*)0)->spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 13:return sizeof(((Public_Span_Use*)0)->spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 14:return sizeof(((Public_Span_Use*)0)->spans_to_reserved);
#endif
#if ENABLE_STATISTICS
case 15:return sizeof(((Public_Span_Use*)0)->spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 16:return sizeof(((Public_Span_Use*)0)->spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 17:return sizeof(((Public_Size_Class_Use*)0)->alloc_current);
#endif
#if ENABLE_STATISTICS
case 18:return sizeof(((Public_Size_Class_Use*)0)->alloc_peak);
#endif
#if ENABLE_STATISTICS
case 19:return sizeof(((Public_Size_Class_Use*)0)->alloc_total);
#endif
#if ENABLE_STATISTICS
case 20:return sizeof(((Public_Size_Class_Use*)0)->free_total);
#endif
#if ENABLE_STATISTICS
case 21:return sizeof(((Public_Size_Class_Use*)0)->spans_current);
#endif
#if ENABLE_STATISTICS
case 22:return sizeof(((Public_Size_Class_Use*)0)->spans_peak);
#endif
#if ENABLE_STATISTICS
case 23:return sizeof(((Public_Size_Class_Use*)0)->spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 24:return sizeof(((Public_Size_Class_Use*)0)->spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 25:return sizeof(((Public_Size_Class_Use*)0)->spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 26:return sizeof(((Public_Size_Class_Use*)0)->spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 27:return sizeof(((Public_Size_Class_Use*)0)->unused);
#endif
case 28:return sizeof(((Public_Span*)0)->free_list);
case 29:return sizeof(((Public_Span*)0)->block_count);
case 30:return sizeof(((Public_Span*)0)->size_class);
case 31:return sizeof(((Public_Span*)0)->free_list_limit);
case 32:return sizeof(((Public_Span*)0)->used_count);
case 33:return sizeof(((Public_Span*)0)->list_size);
case 34:return sizeof(((Public_Span*)0)->block_size);
case 35:return sizeof(((Public_Span*)0)->span_count);
case 36:return sizeof(((Public_Span*)0)->total_spans);
case 37:return sizeof(((Public_Span*)0)->offset_from_master);
case 38:return sizeof(((Public_Span*)0)->remaining_spans);
case 39:return sizeof(((Public_Span*)0)->align_offset);
case 40:return sizeof(((Public_Span*)0)->free_list_deferred);
case 41:return sizeof(((Public_Span*)0)->flags);
case 42:return sizeof(((Public_Span*)0)->heap);
case 43:return sizeof(((Public_Span*)0)->next);
case 44:return sizeof(((Public_Span*)0)->prev);
case 45:return sizeof(((Public_Heap*)0)->owner_thread);
case 46:return sizeof(((Public_Heap*)0)->size_class);
case 47:return sizeof(((Public_Heap*)0)->span_free_deferred);
case 48:return sizeof(((Public_Heap*)0)->full_span_count);
case 49:return sizeof(((Public_Heap*)0)->span_reserve);
case 50:return sizeof(((Public_Heap*)0)->span_reserve_master);
case 51:return sizeof(((Public_Heap*)0)->spans_reserved);
case 52:return sizeof(((Public_Heap*)0)->child_count);
case 53:return sizeof(((Public_Heap*)0)->next_heap);
case 54:return sizeof(((Public_Heap*)0)->next_orphan);
case 55:return sizeof(((Public_Heap*)0)->id);
case 56:return sizeof(((Public_Heap*)0)->finalize);
case 57:return sizeof(((Public_Heap*)0)->master_heap);
#if ENABLE_THREAD_CACHE
case 58:return sizeof(((Public_Heap*)0)->span_cache);
#endif
#if ENABLE_THREAD_CACHE
case 59:return sizeof(((Public_Heap*)0)->span_large_cache);
#endif
#if BINDING_RPMALLOC_PUBLIC_HEAPS
case 60:return sizeof(((Public_Heap*)0)->full_span);
#endif
#if BINDING_RPMALLOC_PUBLIC_HEAPS
case 61:return sizeof(((Public_Heap*)0)->large_huge_span);
#endif
#if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS
case 62:return sizeof(((Public_Heap*)0)->span_use);
#endif
#if ENABLE_STATISTICS
case 63:return sizeof(((Public_Heap*)0)->size_class_use);
#endif
#if ENABLE_STATISTICS
case 64:return sizeof(((Public_Heap*)0)->thread_to_global);
#endif
#if ENABLE_STATISTICS
case 65:return sizeof(((Public_Heap*)0)->global_to_thread);
#endif
default: abort(); } }
#else
/* Generated reviewed public C mirror layout measurements. */
uint64_t API(layout_size)(unsigned index) { switch(index) {
case 0:return sizeof(Public_Heap_Size_Class);
case 1:return sizeof(Public_Span_Cache);
case 2:return sizeof(Public_Span_Large_Cache);
case 3:return sizeof(Public_Span_Use);
#if ENABLE_STATISTICS
case 4:return sizeof(Public_Size_Class_Use);
#endif
case 5:return sizeof(Public_Span);
case 6:return sizeof(Public_Heap);
default: abort(); } }
uint64_t API(layout_offset)(unsigned index) { switch(index) {
case 0:return offsetof(Public_Heap_Size_Class,free_list);
case 1:return offsetof(Public_Heap_Size_Class,partial_span);
case 2:return offsetof(Public_Heap_Size_Class,cache);
case 3:return offsetof(Public_Span_Cache,count);
case 4:return offsetof(Public_Span_Cache,span);
case 5:return offsetof(Public_Span_Large_Cache,count);
case 6:return offsetof(Public_Span_Large_Cache,span);
case 7:return offsetof(Public_Span_Use,current);
case 8:return offsetof(Public_Span_Use,high);
#if ENABLE_STATISTICS
case 9:return offsetof(Public_Span_Use,spans_deferred);
#endif
#if ENABLE_STATISTICS
case 10:return offsetof(Public_Span_Use,spans_to_global);
#endif
#if ENABLE_STATISTICS
case 11:return offsetof(Public_Span_Use,spans_from_global);
#endif
#if ENABLE_STATISTICS
case 12:return offsetof(Public_Span_Use,spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 13:return offsetof(Public_Span_Use,spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 14:return offsetof(Public_Span_Use,spans_to_reserved);
#endif
#if ENABLE_STATISTICS
case 15:return offsetof(Public_Span_Use,spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 16:return offsetof(Public_Span_Use,spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 17:return offsetof(Public_Size_Class_Use,alloc_current);
#endif
#if ENABLE_STATISTICS
case 18:return offsetof(Public_Size_Class_Use,alloc_peak);
#endif
#if ENABLE_STATISTICS
case 19:return offsetof(Public_Size_Class_Use,alloc_total);
#endif
#if ENABLE_STATISTICS
case 20:return offsetof(Public_Size_Class_Use,free_total);
#endif
#if ENABLE_STATISTICS
case 21:return offsetof(Public_Size_Class_Use,spans_current);
#endif
#if ENABLE_STATISTICS
case 22:return offsetof(Public_Size_Class_Use,spans_peak);
#endif
#if ENABLE_STATISTICS
case 23:return offsetof(Public_Size_Class_Use,spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 24:return offsetof(Public_Size_Class_Use,spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 25:return offsetof(Public_Size_Class_Use,spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 26:return offsetof(Public_Size_Class_Use,spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 27:return offsetof(Public_Size_Class_Use,unused);
#endif
case 28:return offsetof(Public_Span,free_list);
case 29:return offsetof(Public_Span,block_count);
case 30:return offsetof(Public_Span,size_class);
case 31:return offsetof(Public_Span,free_list_limit);
case 32:return offsetof(Public_Span,used_count);
case 33:return offsetof(Public_Span,list_size);
case 34:return offsetof(Public_Span,block_size);
case 35:return offsetof(Public_Span,span_count);
case 36:return offsetof(Public_Span,total_spans);
case 37:return offsetof(Public_Span,offset_from_master);
case 38:return offsetof(Public_Span,remaining_spans);
case 39:return offsetof(Public_Span,align_offset);
case 40:return offsetof(Public_Span,free_list_deferred);
case 41:return offsetof(Public_Span,flags);
case 42:return offsetof(Public_Span,heap);
case 43:return offsetof(Public_Span,next);
case 44:return offsetof(Public_Span,prev);
case 45:return offsetof(Public_Heap,owner_thread);
case 46:return offsetof(Public_Heap,size_class);
case 47:return offsetof(Public_Heap,span_free_deferred);
case 48:return offsetof(Public_Heap,full_span_count);
case 49:return offsetof(Public_Heap,span_reserve);
case 50:return offsetof(Public_Heap,span_reserve_master);
case 51:return offsetof(Public_Heap,spans_reserved);
case 52:return offsetof(Public_Heap,child_count);
case 53:return offsetof(Public_Heap,next_heap);
case 54:return offsetof(Public_Heap,next_orphan);
case 55:return offsetof(Public_Heap,id);
case 56:return offsetof(Public_Heap,finalize);
case 57:return offsetof(Public_Heap,master_heap);
case 58:return offsetof(Public_Heap,full_span);
case 59:return offsetof(Public_Heap,large_huge_span);
#if ENABLE_THREAD_CACHE
case 60:return offsetof(Public_Heap,span_cache);
#endif
#if ENABLE_THREAD_CACHE
case 61:return offsetof(Public_Heap,span_large_cache);
#endif
#if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS
case 62:return offsetof(Public_Heap,span_use);
#endif
#if ENABLE_STATISTICS
case 63:return offsetof(Public_Heap,size_class_use);
#endif
#if ENABLE_STATISTICS
case 64:return offsetof(Public_Heap,thread_to_global);
#endif
#if ENABLE_STATISTICS
case 65:return offsetof(Public_Heap,global_to_thread);
#endif
default: abort(); } }
uint64_t API(layout_width)(unsigned index) { switch(index) {
case 0:return sizeof(((Public_Heap_Size_Class*)0)->free_list);
case 1:return sizeof(((Public_Heap_Size_Class*)0)->partial_span);
case 2:return sizeof(((Public_Heap_Size_Class*)0)->cache);
case 3:return sizeof(((Public_Span_Cache*)0)->count);
case 4:return sizeof(((Public_Span_Cache*)0)->span);
case 5:return sizeof(((Public_Span_Large_Cache*)0)->count);
case 6:return sizeof(((Public_Span_Large_Cache*)0)->span);
case 7:return sizeof(((Public_Span_Use*)0)->current);
case 8:return sizeof(((Public_Span_Use*)0)->high);
#if ENABLE_STATISTICS
case 9:return sizeof(((Public_Span_Use*)0)->spans_deferred);
#endif
#if ENABLE_STATISTICS
case 10:return sizeof(((Public_Span_Use*)0)->spans_to_global);
#endif
#if ENABLE_STATISTICS
case 11:return sizeof(((Public_Span_Use*)0)->spans_from_global);
#endif
#if ENABLE_STATISTICS
case 12:return sizeof(((Public_Span_Use*)0)->spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 13:return sizeof(((Public_Span_Use*)0)->spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 14:return sizeof(((Public_Span_Use*)0)->spans_to_reserved);
#endif
#if ENABLE_STATISTICS
case 15:return sizeof(((Public_Span_Use*)0)->spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 16:return sizeof(((Public_Span_Use*)0)->spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 17:return sizeof(((Public_Size_Class_Use*)0)->alloc_current);
#endif
#if ENABLE_STATISTICS
case 18:return sizeof(((Public_Size_Class_Use*)0)->alloc_peak);
#endif
#if ENABLE_STATISTICS
case 19:return sizeof(((Public_Size_Class_Use*)0)->alloc_total);
#endif
#if ENABLE_STATISTICS
case 20:return sizeof(((Public_Size_Class_Use*)0)->free_total);
#endif
#if ENABLE_STATISTICS
case 21:return sizeof(((Public_Size_Class_Use*)0)->spans_current);
#endif
#if ENABLE_STATISTICS
case 22:return sizeof(((Public_Size_Class_Use*)0)->spans_peak);
#endif
#if ENABLE_STATISTICS
case 23:return sizeof(((Public_Size_Class_Use*)0)->spans_to_cache);
#endif
#if ENABLE_STATISTICS
case 24:return sizeof(((Public_Size_Class_Use*)0)->spans_from_cache);
#endif
#if ENABLE_STATISTICS
case 25:return sizeof(((Public_Size_Class_Use*)0)->spans_from_reserved);
#endif
#if ENABLE_STATISTICS
case 26:return sizeof(((Public_Size_Class_Use*)0)->spans_map_calls);
#endif
#if ENABLE_STATISTICS
case 27:return sizeof(((Public_Size_Class_Use*)0)->unused);
#endif
case 28:return sizeof(((Public_Span*)0)->free_list);
case 29:return sizeof(((Public_Span*)0)->block_count);
case 30:return sizeof(((Public_Span*)0)->size_class);
case 31:return sizeof(((Public_Span*)0)->free_list_limit);
case 32:return sizeof(((Public_Span*)0)->used_count);
case 33:return sizeof(((Public_Span*)0)->list_size);
case 34:return sizeof(((Public_Span*)0)->block_size);
case 35:return sizeof(((Public_Span*)0)->span_count);
case 36:return sizeof(((Public_Span*)0)->total_spans);
case 37:return sizeof(((Public_Span*)0)->offset_from_master);
case 38:return sizeof(((Public_Span*)0)->remaining_spans);
case 39:return sizeof(((Public_Span*)0)->align_offset);
case 40:return sizeof(((Public_Span*)0)->free_list_deferred);
case 41:return sizeof(((Public_Span*)0)->flags);
case 42:return sizeof(((Public_Span*)0)->heap);
case 43:return sizeof(((Public_Span*)0)->next);
case 44:return sizeof(((Public_Span*)0)->prev);
case 45:return sizeof(((Public_Heap*)0)->owner_thread);
case 46:return sizeof(((Public_Heap*)0)->size_class);
case 47:return sizeof(((Public_Heap*)0)->span_free_deferred);
case 48:return sizeof(((Public_Heap*)0)->full_span_count);
case 49:return sizeof(((Public_Heap*)0)->span_reserve);
case 50:return sizeof(((Public_Heap*)0)->span_reserve_master);
case 51:return sizeof(((Public_Heap*)0)->spans_reserved);
case 52:return sizeof(((Public_Heap*)0)->child_count);
case 53:return sizeof(((Public_Heap*)0)->next_heap);
case 54:return sizeof(((Public_Heap*)0)->next_orphan);
case 55:return sizeof(((Public_Heap*)0)->id);
case 56:return sizeof(((Public_Heap*)0)->finalize);
case 57:return sizeof(((Public_Heap*)0)->master_heap);
case 58:return sizeof(((Public_Heap*)0)->full_span);
case 59:return sizeof(((Public_Heap*)0)->large_huge_span);
#if ENABLE_THREAD_CACHE
case 60:return sizeof(((Public_Heap*)0)->span_cache);
#endif
#if ENABLE_THREAD_CACHE
case 61:return sizeof(((Public_Heap*)0)->span_large_cache);
#endif
#if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS
case 62:return sizeof(((Public_Heap*)0)->span_use);
#endif
#if ENABLE_STATISTICS
case 63:return sizeof(((Public_Heap*)0)->size_class_use);
#endif
#if ENABLE_STATISTICS
case 64:return sizeof(((Public_Heap*)0)->thread_to_global);
#endif
#if ENABLE_STATISTICS
case 65:return sizeof(((Public_Heap*)0)->global_to_thread);
#endif
default: abort(); } }
#endif
