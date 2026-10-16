/* Independent, CRT-free COFF control: both object inputs must be linked. */
int helper(void);
int mainCRTStartup(void) { return helper(); }
