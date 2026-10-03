/* AeDebug's documented placeholders are PID then inherited event handle.
   Parse them as data, never as a printf format. Reject unknown/incomplete or
   third substitutions and commands exceeding CreateProcess's UTF-16 limit. */
static wchar_t *debug_format_command(const wchar_t *source, uint32_t pid,
                                     uintptr_t event) {
  size_t length = 0;
  while (length < 32768 && source[length]) ++length;
  if (!length || length >= 32768) return NULL;
  wchar_t *output = malloc(32768 * sizeof(*output));
  if (!output) return NULL;
  size_t cursor = 0;
  int placeholders = 0;
  for (size_t i = 0; i < length; ++i) {
    if (source[i] != L'%') {
      if (cursor >= 32767) { free(output); return NULL; }
      output[cursor++] = source[i];
    } else if (source[i + 1] == L'%') {
      if (cursor >= 32767) { free(output); return NULL; }
      output[cursor++] = L'%'; ++i;
    } else if (i + 2 < length && source[i + 1] == L'l' &&
               source[i + 2] == L'd' && placeholders < 2) {
      wchar_t value[32];
      unsigned long long number = placeholders ? (unsigned long long)event : pid;
      int count = swprintf(value, 32, L"%llu", number);
      if (count < 0 || (size_t)count > 32767 - cursor) { free(output); return NULL; }
      memcpy(output + cursor, value, (size_t)count * sizeof(*output));
      cursor += (size_t)count; ++placeholders; i += 2;
    } else { free(output); return NULL; }
  }
  if (!placeholders) { free(output); return NULL; }
  output[cursor] = 0;
  return output;
}
