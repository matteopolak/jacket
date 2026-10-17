// Independent ICCCM selection service. Only the service thread touches its
// Display. Commands borrow no Jai storage after returning; X transfers retain
// their own payload snapshots across subsequent clipboard replacements.
#define _POSIX_C_SOURCE 200809L
#include "data.h"
#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <poll.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

typedef struct Transfer {
  struct Transfer *next;
  Window window;
  Atom property, target;
  unsigned char *data;
  size_t size, offset;
  double deadline;
} Transfer;

typedef struct {
  Display *display;
  Window owner, parent;
  Atom clipboard, utf8, mime_text, targets, bmp, incr, property;
  unsigned char *data;
  size_t size, chunk;
  Atom format;
  Transfer *transfers;
  int failure;
} Service;

typedef struct {
  int operation, status, done;
  Window parent;
  unsigned char *data;
  size_t size;
  int bitmap;
} Command;

static pthread_mutex_t startup = PTHREAD_MUTEX_INITIALIZER;
static pthread_mutex_t queue = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t condition = PTHREAD_COND_INITIALIZER;
static pthread_t worker;
static int wakeup[2] = {-1, -1};
static int initialized, error_handler_installed;
static _Atomic int ready;
static _Atomic int service_error;
static Command *pending;
static _Atomic(Display *) private_display;
static _Atomic int native_error;
static _Atomic(XErrorHandler) previous_error_handler;

static int clipboard_x_error(Display *display, XErrorEvent *event) {
  if (display == atomic_load(&private_display)) {
    atomic_store(&native_error, event->error_code);
    fprintf(stderr,
            "X11 clipboard request failed: error=%u request=%u resource=%lu\n",
            event->error_code, event->request_code, event->resourceid);
    return 0;
  }
  XErrorHandler previous = atomic_load(&previous_error_handler);
  if (previous)
    return previous(display, event);
  fprintf(stderr, "unhandled X11 error on application display\n");
  abort();
}

static int checked_sync(Service *service) {
  XSync(service->display, False);
  return atomic_exchange(&native_error, 0) == 0;
}

static int now(Service *service, double *result) {
  if (service->failure)
    return 0;
  struct timespec value;
  if (clock_gettime(CLOCK_MONOTONIC, &value)) {
    int error = errno;
    service->failure = -error;
    fprintf(stderr, "X11 clipboard monotonic clock failed: %s (errno=%d)\n",
            strerror(error), error);
    return 0;
  }
  *result = (double)value.tv_sec + (double)value.tv_nsec / 1000000000.0;
  return 1;
}

static unsigned char *duplicate(const unsigned char *data, size_t size) {
  unsigned char *copy = malloc(size ? size : 1);
  if (copy && size)
    memcpy(copy, data, size);
  return copy;
}

static int expire_transfers(Service *service, Window destroyed) {
  double time;
  if (!now(service, &time))
    return 0;
  Transfer **cursor = &service->transfers;
  while (*cursor) {
    Transfer *transfer = *cursor;
    if (transfer->deadline <= time || transfer->window == destroyed) {
      *cursor = transfer->next;
      free(transfer->data);
      free(transfer);
    } else
      cursor = &transfer->next;
  }
  return 1;
}

static void request(Service *service, XSelectionRequestEvent *request) {
  XEvent response;
  memset(&response, 0, sizeof response);
  response.xselection.type = SelectionNotify;
  response.xselection.display = service->display;
  response.xselection.requestor = request->requestor;
  response.xselection.selection = request->selection;
  response.xselection.target = request->target;
  response.xselection.time = request->time;
  Atom property = request->property ? request->property : request->target;
  if (request->owner == service->owner &&
      request->selection == service->clipboard &&
      XGetSelectionOwner(service->display, service->clipboard) ==
          service->owner) {
    if (request->target == service->targets) {
      Atom targets[3] = {service->targets, service->format, service->mime_text};
      int count = service->format == service->utf8 ? 3 : 2;
      XChangeProperty(service->display, request->requestor, property, XA_ATOM,
                      32, PropModeReplace, (unsigned char *)targets, count);
      response.xselection.property = property;
    } else if (request->target == service->format ||
               (service->format == service->utf8 &&
                request->target == service->mime_text)) {
      if (service->size <= service->chunk) {
        XChangeProperty(service->display, request->requestor, property,
                        request->target, 8, PropModeReplace, service->data,
                        (int)service->size);
        response.xselection.property = property;
      } else {
        Transfer *transfer = calloc(1, sizeof *transfer);
        if (transfer)
          transfer->data = duplicate(service->data, service->size);
        double time;
        if (transfer && transfer->data && now(service, &time)) {
          transfer->window = request->requestor;
          transfer->property = property;
          transfer->target = request->target;
          transfer->size = service->size;
          transfer->deadline = time + 5;
          transfer->next = service->transfers;
          service->transfers = transfer;
          // Event masks belong to this connection, not the requester.
          XSelectInput(service->display, request->requestor,
                       PropertyChangeMask | StructureNotifyMask);
          unsigned long lower_bound =
              service->size > UINT32_MAX ? UINT32_MAX : service->size;
          XChangeProperty(service->display, request->requestor, property,
                          service->incr, 32, PropModeReplace,
                          (unsigned char *)&lower_bound, 1);
          response.xselection.property = property;
        } else {
          if (transfer) {
            free(transfer->data);
            free(transfer);
          }
        }
      }
    }
  }
  if (!checked_sync(service))
    response.xselection.property = None;
  XSendEvent(service->display, request->requestor, False, 0, &response);
  XFlush(service->display);
  checked_sync(service);
}

static void dispatch(Service *service, XEvent *event) {
  if (event->type == SelectionRequest)
    request(service, &event->xselectionrequest);
  else if (event->type == SelectionClear &&
           event->xselectionclear.window == service->owner &&
           XGetSelectionOwner(service->display, service->clipboard) !=
               service->owner) {
    free(service->data);
    service->data = NULL;
    service->size = 0;
    service->format = None;
  } else if (event->type == DestroyNotify) {
    if (event->xdestroywindow.window == service->owner) {
      service->owner = None;
      free(service->data);
      service->data = NULL;
      service->size = 0;
      service->format = None;
    }
    expire_transfers(service, event->xdestroywindow.window);
  } else if (event->type == PropertyNotify &&
             event->xproperty.state == PropertyDelete) {
    Transfer **cursor = &service->transfers;
    while (*cursor) {
      Transfer *transfer = *cursor;
      if (transfer->window == event->xproperty.window &&
          transfer->property == event->xproperty.atom) {
        double time;
        if (!now(service, &time))
          return;
        size_t size = transfer->size - transfer->offset;
        if (size > service->chunk)
          size = service->chunk;
        XChangeProperty(service->display, transfer->window, transfer->property,
                        transfer->target, 8, PropModeReplace,
                        transfer->data + transfer->offset, (int)size);
        transfer->offset += size;
        transfer->deadline = time + 5;
        int success = checked_sync(service);
        if (!size || !success) {
          *cursor = transfer->next;
          free(transfer->data);
          free(transfer);
        }
        XFlush(service->display);
        break;
      }
      cursor = &transfer->next;
    }
  }
}

static int wait_event(Service *service, XEvent *event, double deadline) {
  double time;
  if (!now(service, &time))
    return 0;
  while (!XPending(service->display)) {
    if (!now(service, &time))
      return 0;
    double remaining = deadline - time;
    if (remaining <= 0)
      return 0;
    struct pollfd fd = {ConnectionNumber(service->display), POLLIN, 0};
    int milliseconds = remaining >= 0.1 ? 100 : (int)(remaining * 1000) + 1;
    int result = poll(&fd, 1, milliseconds);
    if (result < 0 && errno != EINTR)
      return 0;
    if (result > 0 && (fd.revents & (POLLERR | POLLHUP | POLLNVAL)))
      return 0;
    if (!expire_transfers(service, None))
      return 0;
  }
  XNextEvent(service->display, event);
  return 1;
}

static int append(unsigned char **buffer, size_t *size,
                  const unsigned char *data, size_t count) {
  if (count > SIZE_MAX - *size - 1)
    return 0;
  unsigned char *grown = realloc(*buffer, *size + count + 1);
  if (!grown)
    return 0;
  if (count)
    memcpy(grown + *size, data, count);
  *size += count;
  grown[*size] = 0;
  *buffer = grown;
  return 1;
}

// Xlib returns format-32 properties as native longs; ordinary text remains
// bytes.
static int read_bytes(Service *service, Window window, Atom target,
                      unsigned char **buffer, size_t *size) {
  unsigned long remaining = 1;
  long offset = 0;
  while (remaining) {
    Atom type;
    int format;
    unsigned long count;
    unsigned char *data = NULL;
    int result = XGetWindowProperty(service->display, window, service->property,
                                    offset, 16384, True, AnyPropertyType, &type,
                                    &format, &count, &remaining, &data);
    int valid = result == Success && type == target && format == 8 &&
                append(buffer, size, data, count);
    if (data)
      XFree(data);
    if (!valid || (remaining && (!count || count % 4)))
      return 0;
    if (count / 4 > (unsigned long)(LONG_MAX - offset))
      return 0;
    offset += (long)(count / 4);
  }
  return 1;
}

static int receive(Service *service, Command *command) {
  if (service->failure)
    return service->failure;
  if (XGetSelectionOwner(service->display, service->clipboard) == None)
    return 0;
  Window parent =
      command->parent ? command->parent : DefaultRootWindow(service->display);
  Window window =
      XCreateSimpleWindow(service->display, parent, 0, 0, 1, 1, 0, 0, 0);
  if (!checked_sync(service) || !window)
    return -1;
  XSelectInput(service->display, window,
               PropertyChangeMask | StructureNotifyMask);
  int status = -1;
  // The owner refused both text targets: it holds something else (an image), and the text is empty.
  int no_text = 0;
  Atom target = service->utf8;
  for (int attempt = 0; attempt < 2; ++attempt) {
    XDeleteProperty(service->display, window, service->property);
    XConvertSelection(service->display, service->clipboard, target,
                      service->property, window, CurrentTime);
    XFlush(service->display);
    double time;
    if (!now(service, &time))
      break;
    double deadline = time + 5;
    int incremental = 0, notified = 0, refused = 0;
    XEvent event;
    while (wait_event(service, &event, deadline)) {
      if (event.type == DestroyNotify &&
          event.xdestroywindow.window == window) {
        window = None;
        break;
      }
      if (event.type == SelectionNotify &&
          event.xselection.requestor == window &&
          event.xselection.selection == service->clipboard &&
          event.xselection.target == target) {
        if (!event.xselection.property) {
          refused = 1;
          break;
        }
        if (event.xselection.property != service->property)
          break;
        Atom type;
        int format;
        unsigned long count, remaining;
        unsigned char *data = NULL;
        int result = XGetWindowProperty(
            service->display, window, service->property, 0, 0, False,
            AnyPropertyType, &type, &format, &count, &remaining, &data);
        if (data)
          XFree(data);
        if (result != Success)
          break;
        notified = 1;
        if (type == service->incr && format == 32) {
          incremental = 1;
          XDeleteProperty(service->display, window, service->property);
          XFlush(service->display);
        } else {
          if (read_bytes(service, window, target, &command->data,
                         &command->size))
            status = 0;
          break;
        }
      } else if (notified && incremental && event.type == PropertyNotify &&
                 event.xproperty.window == window &&
                 event.xproperty.atom == service->property &&
                 event.xproperty.state == PropertyNewValue) {
        size_t before = command->size;
        if (!read_bytes(service, window, target, &command->data,
                        &command->size))
          break;
        XFlush(service->display);
        if (before == command->size) {
          status = 0;
          break;
        }
        if (!now(service, &time))
          break;
        deadline = time + 5;
      } else
        dispatch(service, &event);
    }
    if (service->failure || !refused || target == XA_STRING) {
      no_text = refused && target == XA_STRING;
      break;
    }
    target = XA_STRING;
  }
  if (window)
    XDestroyWindow(service->display, window);
  if (!checked_sync(service))
    status = -1;
  if (service->failure)
    return service->failure;
  if (status && no_text)
    return 0;
  if (!status && target == XA_STRING) {
    unsigned char *utf8 = NULL;
    size_t size = 0;
    for (size_t i = 0; i < command->size; ++i) {
      unsigned char byte = command->data[i];
      unsigned char encoded[2] = {(unsigned char)(0xc0 | (byte >> 6)),
                                  (unsigned char)(0x80 | (byte & 63))};
      if (!append(&utf8, &size, byte < 128 ? &byte : encoded,
                  byte < 128 ? 1 : 2)) {
        free(utf8);
        return -1;
      }
    }
    free(command->data);
    command->data = utf8;
    command->size = size;
  }
  if (!status && !pcb_utf8(command->data, command->size))
    status = -1;
  return status;
}

static int replace(Service *service, Command *command) {
  if (service->failure)
    return service->failure;
  Window parent =
      command->parent ? command->parent : DefaultRootWindow(service->display);
  Window owner = service->owner;
  if (!owner || parent != service->parent) {
    owner = XCreateSimpleWindow(service->display, parent, 0, 0, 1, 1, 0, 0, 0);
    if (!checked_sync(service) || !owner)
      return -1;
    XSelectInput(service->display, owner, StructureNotifyMask);
  }
  XSetSelectionOwner(service->display, service->clipboard, owner, CurrentTime);
  Window actual_owner =
      XGetSelectionOwner(service->display, service->clipboard);
  if (!checked_sync(service) || actual_owner != owner) {
    if (owner != service->owner)
      XDestroyWindow(service->display, owner);
    return -1;
  }
  if (service->owner && owner != service->owner)
    XDestroyWindow(service->display, service->owner);
  free(service->data);
  service->data = command->data;
  command->data = NULL;
  service->size = command->size;
  service->format = command->bitmap ? service->bmp : service->utf8;
  service->owner = owner;
  service->parent = parent;
  XFlush(service->display);
  return 0;
}

static void finish_service(Service *service) {
  free(service->data);
  service->data = NULL;
  while (service->transfers) {
    Transfer *next = service->transfers->next;
    free(service->transfers->data);
    free(service->transfers);
    service->transfers = next;
  }
  if (service->display)
    XCloseDisplay(service->display);
  service->display = NULL;
  atomic_store(&private_display, NULL);
  pthread_mutex_lock(&queue);
  service_error = service->failure ? service->failure : -1;
  ready = -1;
  if (pending) {
    pending->status = service->failure ? service->failure : -1;
    pending->done = 1;
    pending = NULL;
  }
  pthread_cond_broadcast(&condition);
  pthread_mutex_unlock(&queue);
}

static void *serve(void *unused) {
  (void)unused;
  Service service = {0};
  service.display = XOpenDisplay(NULL);
  if (service.display) {
    atomic_store(&private_display, service.display);
    service.clipboard = XInternAtom(service.display, "CLIPBOARD", False);
    service.utf8 = XInternAtom(service.display, "UTF8_STRING", False);
    service.mime_text =
        XInternAtom(service.display, "text/plain;charset=utf-8", False);
    service.targets = XInternAtom(service.display, "TARGETS", False);
    service.bmp = XInternAtom(service.display, "image/bmp", False);
    service.incr = XInternAtom(service.display, "INCR", False);
    service.property =
        XInternAtom(service.display, "_JAI_PORTABLE_CLIPBOARD", False);
    long maximum = XMaxRequestSize(service.display);
    service.chunk = maximum > 128 ? (size_t)(maximum - 128) * 4 : 1024;
    if (service.chunk > 65536)
      service.chunk = 65536;
  }
  pthread_mutex_lock(&queue);
  ready = service.display ? 1 : -1;
  pthread_cond_broadcast(&condition);
  pthread_mutex_unlock(&queue);
  if (!service.display)
    return NULL;
  int running = 1;
  while (running) {
    // Drain Xlib's private queue before polling its socket.
    while (!service.failure && XPending(service.display)) {
      XEvent event;
      XNextEvent(service.display, &event);
      dispatch(&service, &event);
    }
    if (service.failure || !expire_transfers(&service, None))
      break;
    pthread_mutex_lock(&queue);
    Command *command = pending;
    pthread_mutex_unlock(&queue);
    if (command) {
      int status = 0;
      if (command->operation == 1)
        status = replace(&service, command);
      else if (command->operation == 2)
        status = receive(&service, command);
      else
        running = 0;
      if (service.failure)
        break;
      pthread_mutex_lock(&queue);
      command->status = status;
      command->done = 1;
      pending = NULL;
      pthread_cond_broadcast(&condition);
      pthread_mutex_unlock(&queue);
      continue;
    }
    struct pollfd fds[2] = {{ConnectionNumber(service.display), POLLIN, 0},
                            {wakeup[0], POLLIN, 0}};
    int result = poll(fds, 2, 100);
    if (result < 0 && errno == EINTR)
      continue;
    if (result < 0 || (fds[0].revents & (POLLERR | POLLHUP | POLLNVAL)))
      break;
    if (fds[1].revents & POLLIN) {
      unsigned char bytes[64];
      while (read(wakeup[0], bytes, sizeof bytes) > 0) {
      }
    }
  }
  finish_service(&service);
  return NULL;
}

static int submit(Command *command) {
  pthread_mutex_lock(&queue);
  while (pending && ready == 1)
    pthread_cond_wait(&condition, &queue);
  if (ready != 1) {
    pthread_mutex_unlock(&queue);
    return service_error ? service_error : -1;
  }
  pending = command;
  unsigned char byte = 1;
  ssize_t written;
  do {
    written = write(wakeup[1], &byte, 1);
  } while (written < 0 && errno == EINTR);
  if (written != 1 && errno != EAGAIN) {
    pending = NULL;
    pthread_cond_broadcast(&condition);
    pthread_mutex_unlock(&queue);
    return -1;
  }
  while (!command->done)
    pthread_cond_wait(&condition, &queue);
  pthread_mutex_unlock(&queue);
  return command->status;
}

static void shutdown_service(void) {
  Command command = {0};
  submit(&command);
  pthread_join(worker, NULL);
  close(wakeup[0]);
  close(wakeup[1]);
}

static int initialize(void) {
  pthread_mutex_lock(&startup);
  if (service_error) {
    int error = service_error;
    pthread_mutex_unlock(&startup);
    return error;
  }
  if (!initialized) {
    // Must precede every display in an application using concurrent Xlib.
    // Window's shared provider initialization has the same prerequisite.
    if (!XInitThreads() || pipe(wakeup)) {
      pthread_mutex_unlock(&startup);
      return -1;
    }
    for (int i = 0; i < 2; ++i) {
      if (fcntl(wakeup[i], F_SETFD, FD_CLOEXEC) == -1 ||
          fcntl(wakeup[i], F_SETFL, O_NONBLOCK) == -1) {
        close(wakeup[0]);
        close(wakeup[1]);
        pthread_mutex_unlock(&startup);
        return -1;
      }
    }
    if (!error_handler_installed) {
      previous_error_handler = XSetErrorHandler(clipboard_x_error);
      error_handler_installed = 1;
    }
    pthread_mutex_lock(&queue);
    ready = 0;
    int result = pthread_create(&worker, NULL, serve, NULL);
    if (!result)
      while (!ready)
        pthread_cond_wait(&condition, &queue);
    pthread_mutex_unlock(&queue);
    if (result || ready < 0) {
      if (!result)
        pthread_join(worker, NULL);
      close(wakeup[0]);
      close(wakeup[1]);
      pthread_mutex_unlock(&startup);
      return service_error ? service_error : -1;
    }
    initialized = 1;
    if (atexit(shutdown_service)) {
      shutdown_service();
      initialized = 0;
      pthread_mutex_unlock(&startup);
      return -1;
    }
  }
  pthread_mutex_unlock(&startup);
  return 0;
}

__attribute__((visibility("default"))) int
portable_clipboard_text(unsigned long parent, const unsigned char *text,
                        size_t size) {
  if (!pcb_utf8(text, size))
    return -1;
  int status = initialize();
  if (status)
    return status;
  Command command = {.operation = 1, .parent = parent, .size = size};
  command.data = duplicate(text, size);
  if (!command.data)
    return -1;
  status = submit(&command);
  free(command.data);
  return status;
}

__attribute__((visibility("default"))) int
portable_clipboard_bitmap(unsigned long parent, int32_t width, int32_t height,
                          const unsigned char *pixels, int32_t channels,
                          int flip) {
  Command command = {.operation = 1, .parent = parent, .bitmap = 1};
  command.data =
      pcb_bitmap(width, height, pixels, channels, flip, &command.size);
  if (!command.data)
    return -1;
  int status = initialize();
  if (!status)
    status = submit(&command);
  free(command.data);
  return status;
}

__attribute__((visibility("default"))) int
portable_clipboard_read(unsigned long parent, unsigned char **text,
                        size_t *size) {
  *text = NULL;
  *size = 0;
  int status = initialize();
  if (status)
    return status;
  Command command = {.operation = 2, .parent = parent};
  status = submit(&command);
  if (!status) {
    *text = command.data;
    *size = command.size;
  } else
    free(command.data);
  return status;
}

__attribute__((visibility("default"))) void
portable_clipboard_free(void *text) {
  free(text);
}
