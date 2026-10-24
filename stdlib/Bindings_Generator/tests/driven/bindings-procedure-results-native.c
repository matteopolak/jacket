typedef struct Display Display;
typedef int (*ErrorHandler)(Display *);

static int primary_handler(Display *display) { (void) display; return 11; }
static int replacement_handler(Display *display) { (void) display; return 22; }
static int forced_sync;
static int synchronized(Display *display) { (void) display; return forced_sync; }
static int (*after_function)(Display *) = primary_handler;
static ErrorHandler error_handler = primary_handler;
static void (*signal_handler)(int);

int (*set_after_function(Display *display, int (*callback)(Display *)))(Display *) {
    int (*previous)(Display *) = after_function;
    (void) display;
    after_function = callback;
    return previous;
}

int (*set_synchronize(Display *display, int enable))(Display *) {
    (void) display;
    forced_sync = enable;
    return enable ? synchronized : 0;
}

void (*set_signal(int number, void (*handler)(int)))(int) {
    void (*previous)(int) = signal_handler;
    (void) number;
    signal_handler = handler;
    return previous;
}

ErrorHandler set_error_handler(ErrorHandler handler) {
    ErrorHandler previous = error_handler;
    error_handler = handler ? handler : replacement_handler;
    return previous;
}
