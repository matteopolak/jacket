// Original public SDK oracle: run a real child under the macOS tracing API.
#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/ptrace.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

int main(int argc, char **argv) {
  if (argc < 2) return 2;
  pid_t child = fork();
  if (child == -1) { perror("fork"); return 2; }
  if (!child) {
    if (ptrace(PT_TRACE_ME, 0, NULL, 0) == -1) { perror("PT_TRACE_ME"); _Exit(2); }
    execv(argv[1], argv + 1);
    perror("execv");
    _Exit(2);
  }
  int traps = 0;
  for (;;) {
    int status;
    pid_t waited;
    do { waited = waitpid(child, &status, 0); } while (waited == -1 && errno == EINTR);
    if (waited == -1) { perror("waitpid"); return 2; }
    if (WIFEXITED(status)) {
      if (WEXITSTATUS(status) || traps != 4) {
        fprintf(stderr, "child status=%d; SDK observed %d traps, expected exec+three locations\n", WEXITSTATUS(status), traps);
        return 1;
      }
      return 0;
    }
    if (!WIFSTOPPED(status) || WSTOPSIG(status) != SIGTRAP) {
      fprintf(stderr, "unexpected traced child status %d\n", status);
      (void)kill(child, SIGKILL);
      (void)waitpid(child, NULL, 0);
      return 1;
    }
    ++traps;
    if (ptrace(PT_CONTINUE, child, (caddr_t)1, 0) == -1) {
      perror("PT_CONTINUE");
      (void)kill(child, SIGKILL);
      (void)waitpid(child, NULL, 0);
      return 2;
    }
  }
}
