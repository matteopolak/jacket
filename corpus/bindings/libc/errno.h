/* Declarations only: errno for the corpus libraries, which read it as an int. */
#ifndef CORPUS_LIBC_ERRNO_H
#define CORPUS_LIBC_ERRNO_H
extern int errno;
#define EINVAL 22
#define ENOENT 2
#define EIO 5
#define ENOMEM 12
#define EEXIST 17
#define ENOSPC 28
#define EAGAIN 11
#define ENOTSUP 95
#endif
