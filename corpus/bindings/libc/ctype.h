/* Declarations only: the character classes the corpus libraries call. */
#ifndef CORPUS_LIBC_CTYPE_H
#define CORPUS_LIBC_CTYPE_H
int isalnum(int c);
int isalpha(int c);
int isdigit(int c);
int isxdigit(int c);
int isspace(int c);
int isupper(int c);
int islower(int c);
int isprint(int c);
int iscntrl(int c);
int toupper(int c);
int tolower(int c);
#endif
