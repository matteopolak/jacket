/* Independently designed process-owned location set, shared by native hosts.
   Filenames are copied; callers may release or modify their input immediately.
   Entries live until process exit. The first attempt consumes the location,
   including when no debugger is attached. Allocation failure does not consume it. */
typedef struct Debug_Once_Location {
  struct Debug_Once_Location *next;
  int64_t count, line, character;
  unsigned char filename[];
} Debug_Once_Location;
static Debug_Once_Location *debug_once_locations;

DEBUG_EXPORT int32_t developer_debug_break_once(const unsigned char *filename,
                                               int64_t count, int64_t line,
                                               int64_t character) {
  if (count < 0 || (!filename && count) ||
      (uint64_t)count > SIZE_MAX - sizeof(Debug_Once_Location)) return -1;
  if (debug_once_lock() != 0) return -1;
  for (Debug_Once_Location *entry = debug_once_locations; entry; entry = entry->next) {
    if (entry->count == count && entry->line == line && entry->character == character &&
        (!count || memcmp(entry->filename, filename, (size_t)count) == 0)) {
      if (debug_once_unlock() != 0) return -1;
      return 0;
    }
  }
  Debug_Once_Location *entry = malloc(sizeof(*entry) + (size_t)count);
  if (!entry) { (void)debug_once_unlock(); return -1; }
  entry->count = count;
  entry->line = line;
  entry->character = character;
  if (count) memcpy(entry->filename, filename, (size_t)count);
  entry->next = debug_once_locations;
  debug_once_locations = entry;
  if (debug_once_unlock() != 0) return -1;
  return developer_debug_breakpoint();
}
