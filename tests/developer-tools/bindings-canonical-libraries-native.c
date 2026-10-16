/* Independently authored provider compiled three times with distinct symbols. */
#ifndef LIBRARY_SYMBOL
#error A concrete provider symbol is required
#endif
#ifndef LIBRARY_VALUE
#error A concrete provider value is required
#endif
int LIBRARY_SYMBOL(void) { return LIBRARY_VALUE; }
