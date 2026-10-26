# Memory and the context

`defer`, `New` and `free`, the temporary allocator, the implicit `context`, and replacing its
allocator and logger for a stretch of code.

## Map

- [main.jai](main.jai): Calls the section.
- [memory.jai](memory.jai): A hand-made linked list, temporary storage, a counting allocator and a custom logger.

## Things to try

- Remove the `free` of a node and see the counting allocator report fewer frees.
- Change what the logger prints.
