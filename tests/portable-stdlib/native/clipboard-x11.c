// Linux CI: a separate Xlib client validates actual selection traffic. Fork
// happens BEFORE any service thread exists; no test-owned fake clipboard.
#define _POSIX_C_SOURCE 200809L
#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <assert.h>
#include <poll.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

int portable_clipboard_text(unsigned long, const unsigned char *, size_t);
int portable_clipboard_bitmap(unsigned long, int32_t, int32_t,
                              const unsigned char *, int32_t, int);
int portable_clipboard_read(unsigned long, unsigned char **, size_t *);
void portable_clipboard_free(void *);

static const unsigned char sample[] = {'x', 0xc3, 0xa9, 0, 'z'};

static void exact(int fd, void *bytes, size_t size, int writing) {
  unsigned char *cursor = bytes;
  while (size) {
    ssize_t count = writing ? write(fd, cursor, size) : read(fd, cursor, size);
    assert(count > 0);
    cursor += count;
    size -= (size_t)count;
  }
}

static unsigned char *read_selection(Display *display, Window window,
                                     Atom target, size_t *size) {
  Atom selection = XInternAtom(display, "CLIPBOARD", False);
  Atom property = XInternAtom(display, "PORTABLE_CLIPBOARD_TEST", False);
  Atom incr = XInternAtom(display, "INCR", False);
  XSelectInput(display, window, PropertyChangeMask);
  XConvertSelection(display, selection, target, property, window, CurrentTime);
  XFlush(display);
  int notified = 0, incremental = 0;
  unsigned char *result = NULL;
  *size = 0;
  for (;;) {
    XEvent event;
    XNextEvent(display, &event);
    if (event.type == SelectionNotify) {
      assert(event.xselection.property == property);
      notified = 1;
    } else if (!(notified && incremental && event.type == PropertyNotify &&
                 event.xproperty.atom == property &&
                 event.xproperty.state == PropertyNewValue))
      continue;
    Atom type;
    int format;
    unsigned long count, remaining;
    unsigned char *data = NULL;
    int status = XGetWindowProperty(display, window, property, 0, 1048576, True,
                                    AnyPropertyType, &type, &format, &count,
                                    &remaining, &data);
    assert(status == Success && remaining == 0);
    if (type == incr) {
      assert(format == 32);
      incremental = 1;
      XDeleteProperty(display, window, property);
    } else {
      assert(type == target && format == 8);
      unsigned char *grown = realloc(result, *size + count + 1);
      assert(grown);
      result = grown;
      if (count)
        memcpy(result + *size, data, count);
      *size += count;
      if (data)
        XFree(data);
      if (!incremental || !count)
        return result;
      data = NULL;
    }
    if (data)
      XFree(data);
    XFlush(display);
  }
}

static void peer(int input, int output) {
  assert(XInitThreads());
  Display *display = XOpenDisplay(NULL);
  assert(display);
  Window window = XCreateSimpleWindow(display, DefaultRootWindow(display), 0, 0,
                                      1, 1, 0, 0, 0);
  Atom clipboard = XInternAtom(display, "CLIPBOARD", False);
  Atom utf8 = XInternAtom(display, "UTF8_STRING", False);
  int mode = 0;
  char ack = 'k';
  exact(output, &window, sizeof window, 1);
  for (;;) {
    while (XPending(display)) {
      XEvent event;
      XNextEvent(display, &event);
      if (event.type != SelectionRequest)
        continue;
      XSelectionRequestEvent *request = &event.xselectionrequest;
      XEvent reply = {0};
      reply.xselection.type = SelectionNotify;
      reply.xselection.requestor = request->requestor;
      reply.xselection.selection = request->selection;
      reply.xselection.target = request->target;
      reply.xselection.time = request->time;
      Atom property = request->property ? request->property : request->target;
      if (mode == 'J' && request->target == utf8) {
        unsigned long length = 300000;
        Atom incr = XInternAtom(display, "INCR", False);
        XSelectInput(display, request->requestor, PropertyChangeMask);
        XChangeProperty(display, request->requestor, property, incr, 32,
                        PropModeReplace, (unsigned char *)&length, 1);
        reply.xselection.property = property;
      } else if ((mode == 'U' && request->target == utf8) ||
                 (mode == 'L' && request->target == XA_STRING)) {
        const unsigned char latin[] = {'x', 0xe9, 0, 'z'};
        XChangeProperty(display, request->requestor, property, request->target,
                        8, PropModeReplace, mode == 'L' ? latin : sample,
                        mode == 'L' ? (int)sizeof latin : (int)sizeof sample);
        reply.xselection.property = property;
      }
      XSendEvent(display, request->requestor, False, 0, &reply);
      XFlush(display);
      if (mode == 'J' && reply.xselection.property) {
        size_t offset = 0;
        for (;;) {
          XEvent next;
          XNextEvent(display, &next);
          if (next.type != PropertyNotify ||
              next.xproperty.window != request->requestor ||
              next.xproperty.atom != property ||
              next.xproperty.state != PropertyDelete)
            continue;
          unsigned char chunk[4096];
          size_t count = 300000 - offset;
          if (count > sizeof chunk)
            count = sizeof chunk;
          for (size_t i = 0; i < count; ++i)
            chunk[i] = (unsigned char)('a' + (offset + i) % 26);
          XChangeProperty(display, request->requestor, property, utf8, 8,
                          PropModeReplace, chunk, (int)count);
          XFlush(display);
          offset += count;
          if (!count)
            break;
        }
      }
    }
    struct pollfd fds[2] = {{input, POLLIN, 0},
                            {ConnectionNumber(display), POLLIN, 0}};
    assert(poll(fds, 2, 5000) >= 0);
    if (!(fds[0].revents & POLLIN))
      continue;
    char command;
    exact(input, &command, 1, 0);
    if (command == 'Q')
      break;
    if (command == 'U' || command == 'L' || command == 'N' || command == 'J') {
      mode = command;
      XSetSelectionOwner(display, clipboard, window, CurrentTime);
      assert(XGetSelectionOwner(display, clipboard) == window);
    } else if (command == 'D') {
      XDestroyWindow(display, window);
      XSync(display, False);
      window = None;
    } else {
      size_t size;
      Atom target =
          command == 'B' ? XInternAtom(display, "image/bmp", False) : utf8;
      unsigned char *data = read_selection(display, window, target, &size);
      if (command == 'R')
        assert(size == sizeof sample && !memcmp(data, sample, size));
      else if (command == 'I') {
        assert(size == 300000);
        for (size_t i = 0; i < size; ++i)
          assert(data[i] == (unsigned char)('a' + i % 26));
      } else if (command == 'B') {
        const unsigned char expected[] = {6, 5, 4, 8, 3, 2, 1, 7};
        assert(size == 146 && data[0] == 'B' && data[1] == 'M');
        assert(!memcmp(data + 138, expected, sizeof expected));
      } else
        assert(0);
      free(data);
    }
    exact(output, &ack, 1, 1);
  }
  XCloseDisplay(display);
  _exit(0);
}

static void command(int output, int input, char byte) {
  exact(output, &byte, 1, 1);
  char ack;
  exact(input, &ack, 1, 0);
  assert(ack == 'k');
}

int main(void) {
  alarm(40); // fail blocked selection/worker tests rather than hang CI
  int to_peer[2], from_peer[2];
  assert(!pipe(to_peer) && !pipe(from_peer));
  pid_t child = fork();
  assert(child >= 0);
  if (!child) {
    close(to_peer[1]);
    close(from_peer[0]);
    peer(to_peer[0], from_peer[1]);
  }
  close(to_peer[0]);
  close(from_peer[1]);
  Window window;
  exact(from_peer[0], &window, sizeof window, 0);
  unsigned char borrowed[sizeof sample];
  memcpy(borrowed, sample, sizeof sample);
  assert(!portable_clipboard_text(0, borrowed, sizeof borrowed));
  memset(borrowed, '!', sizeof borrowed);
  command(to_peer[1], from_peer[0], 'R');
  unsigned char *large = malloc(300000);
  assert(large);
  for (size_t i = 0; i < 300000; ++i)
    large[i] = (unsigned char)('a' + i % 26);
  assert(!portable_clipboard_text(0, large, 300000));
  free(large);
  command(to_peer[1], from_peer[0], 'I');
  const unsigned char pixels[] = {1, 2, 3, 7, 4, 5, 6, 8};
  assert(!portable_clipboard_bitmap(window, 1, 2, pixels, 4, 1));
  command(to_peer[1], from_peer[0], 'B');
  assert(!portable_clipboard_text(window, sample, sizeof sample));
  command(to_peer[1], from_peer[0], 'R');
  for (int mode = 0; mode < 2; ++mode) {
    command(to_peer[1], from_peer[0], mode ? 'L' : 'U');
    unsigned char *text;
    size_t size;
    assert(!portable_clipboard_read(window, &text, &size));
    assert(size == sizeof sample && !memcmp(text, sample, size));
    portable_clipboard_free(text);
  }
  command(to_peer[1], from_peer[0], 'N');
  unsigned char *text;
  size_t size;
  // The owner holds something that is not text: the text is empty, not an
  // error.
  assert(!portable_clipboard_read(0, &text, &size) && size == 0);
  portable_clipboard_free(text);
  command(to_peer[1], from_peer[0], 'J');
  assert(!portable_clipboard_read(0, &text, &size) && size == 300000);
  for (size_t i = 0; i < size; ++i)
    assert(text[i] == (unsigned char)('a' + i % 26));
  portable_clipboard_free(text);
  assert(!portable_clipboard_text(window, sample, sizeof sample));
  command(to_peer[1], from_peer[0], 'D');
  assert(!portable_clipboard_read(0, &text, &size) && size == 0);
  portable_clipboard_free(text);
  char quit = 'Q';
  exact(to_peer[1], &quit, 1, 1);
  int status;
  assert(waitpid(child, &status, 0) == child && WIFEXITED(status) &&
         !WEXITSTATUS(status));
  return 0;
}
