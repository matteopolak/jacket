// Native-ready SDK automation of real MessageBox retry/ignore/abort choices.
#define _WIN32_WINNT 0x0600
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>
#include <stdint.h>
#include <wchar.h>
__declspec(dllimport) int32_t developer_debug_dialog(const unsigned char *, int64_t);
typedef struct { DWORD thread; int choice; BOOL posted; } Choice;
static BOOL CALLBACK choose(HWND window, LPARAM opaque) {
  Choice *choice = (Choice *)opaque;
  WCHAR title[64];
  if (GetWindowTextW(window, title, 64) && wcscmp(title, L"Assertion failed") == 0) {
    choice->posted = PostMessageW(window, WM_COMMAND, (WPARAM)choice->choice, 0);
    return FALSE;
  }
  return TRUE;
}
static DWORD WINAPI chooser(void *opaque) {
  Choice *choice = opaque;
  for (int i = 0; i < 1000; ++i) {
    EnumThreadWindows(choice->thread, choose, (LPARAM)choice);
    if (choice->posted) return 0;
    Sleep(5);
  }
  // A failed fixture must not leave a modal test process hanging indefinitely.
  TerminateProcess(GetCurrentProcess(), 99);
  return 1;
}
int main(void) {
  int choices[3] = {IDIGNORE, IDRETRY, IDABORT};
  for (int i = 0; i < 3; ++i) {
    Choice choice = {GetCurrentThreadId(), choices[i], FALSE};
    HANDLE worker = CreateThread(NULL, 0, chooser, &choice, 0, NULL);
    assert(worker);
    static const unsigned char text[] = "original Windows assertion dialog";
    assert(developer_debug_dialog(text, sizeof(text) - 1) == i);
    assert(WaitForSingleObject(worker, 10000) == WAIT_OBJECT_0);
    DWORD status;
    assert(GetExitCodeThread(worker, &status) && status == 0 && choice.posted);
    assert(CloseHandle(worker));
  }
  assert(developer_debug_dialog(NULL, -1) == 2);
  return 0;
}
