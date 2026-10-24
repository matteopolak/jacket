typedef struct Display Display;
typedef int (*ErrorHandler)(Display *);

int (*set_after_function(Display *display, int (*callback)(Display *)))(Display *);
int (*set_synchronize(Display *display, int enable))(Display *);
void (*set_signal(int number, void (*handler)(int)))(int);
ErrorHandler set_error_handler(ErrorHandler handler);
