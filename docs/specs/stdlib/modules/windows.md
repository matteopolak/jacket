# Windows


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- [OS == .WINDOWS] `LARGE_INTEGER_S64 :: s64`
- [OS == .WINDOWS] `QueryPerformanceCounter :: (pc: *s64) -> s32 #deprecated #foreign kernel32`
- [OS == .WINDOWS] `QueryPerformanceFrequency :: (pc: *s64) -> s32 #deprecated #foreign kernel32`
- [OS == .WINDOWS] `SetFilePointerEx :: (hFile: HANDLE, liDistanceToMove: LARGE_INTEGER_S64, lpNewFilePointer: *LARGE_INTEGER_S64, dwMoveMethod: SetFilePointer_MoveMethod) -> BOOL #deprecated #foreign kernel32`
- [OS == .WINDOWS] `DWORD :: u32`
- [OS == .WINDOWS] `WORD :: u16`
- [OS == .WINDOWS] `BOOL :: enum s32`
- [OS == .WINDOWS] `BOOLEAN :: enum u8`
- [OS == .WINDOWS] `SIZE_T :: u64`
- [OS == .WINDOWS] `size_t :: u64`
- [OS == .WINDOWS] `LPSTR :: *u8`
- [OS == .WINDOWS] `LPCSTR :: *u8`
- [OS == .WINDOWS] `LPCWSTR :: *u16`
- [OS == .WINDOWS] `WPARAM :: u64`
- [OS == .WINDOWS] `LPARAM :: s64`
- [OS == .WINDOWS] `LRESULT :: s64`
- [OS == .WINDOWS] `HANDLE :: *void`
- [OS == .WINDOWS] `HKEY :: HANDLE`
- [OS == .WINDOWS] `HMETAFILE :: HANDLE`
- [OS == .WINDOWS] `HINSTANCE :: HANDLE`
- [OS == .WINDOWS] `HMODULE :: HINSTANCE`
- [OS == .WINDOWS] `HWND :: HANDLE`
- [OS == .WINDOWS] `HDC :: HANDLE`
- [OS == .WINDOWS] `HMONITOR :: HANDLE`
- [OS == .WINDOWS] `HBRUSH :: HANDLE`
- [OS == .WINDOWS] `HMENU :: HANDLE`
- [OS == .WINDOWS] `HICON :: HANDLE`
- [OS == .WINDOWS] `HCURSOR :: HANDLE`
- [OS == .WINDOWS] `HRAWINPUT :: HANDLE`
- [OS == .WINDOWS] `HBITMAP :: HANDLE`
- [OS == .WINDOWS] `HGDIOBJ :: HANDLE`
- [OS == .WINDOWS] `HLOCAL :: HANDLE`
- [OS == .WINDOWS] `HFILE :: s32`
- [OS == .WINDOWS] `HRESULT :: s32`
- [OS == .WINDOWS] `INFINITE : u32 : 0xffff_ffff`
- [OS == .WINDOWS] `VARTYPE :: u16`
- [OS == .WINDOWS] `LARGE_INTEGER :: union`
- [OS == .WINDOWS] `ULARGE_INTEGER :: union`
- [OS == .WINDOWS] `RECT :: struct`
- [OS == .WINDOWS] `SMALL_RECT :: struct`
- [OS == .WINDOWS] `COORD :: struct`
- [OS == .WINDOWS] `POINT :: struct`
- [OS == .WINDOWS] `GUID :: struct`
- [OS == .WINDOWS] `UUID :: GUID`
- [OS == .WINDOWS] `LUID :: struct`
- [OS == .WINDOWS] `SIZE :: struct`
- [OS == .WINDOWS] `MEMORY_BASIC_INFORMATION :: struct`
- [OS == .WINDOWS] `operator == :: (a: GUID, b: GUID) -> bool`
- [OS == .WINDOWS] `REFGUID :: *GUID`
- [OS == .WINDOWS] `IID :: GUID`
- [OS == .WINDOWS] `REFIID :: *IID`
- [OS == .WINDOWS] `S_OK :HRESULT: 0x00000000`
- [OS == .WINDOWS] `E_NOTIMPL :HRESULT: 0x80004001`
- [OS == .WINDOWS] `E_NOINTERFACE :HRESULT: 0x80004002`
- [OS == .WINDOWS] `E_POINTER :HRESULT: 0x80004003`
- [OS == .WINDOWS] `E_ABORT :HRESULT: 0x80004004`
- [OS == .WINDOWS] `E_FAIL :HRESULT: 0x80004005`
- [OS == .WINDOWS] `E_ACCESSDENIED :HRESULT: 0x80070005`
- [OS == .WINDOWS] `E_HANDLE :HRESULT: 0x80070006`
- [OS == .WINDOWS] `E_OUTOFMEMORY :HRESULT: 0x8007000E`
- [OS == .WINDOWS] `E_INVALIDARG :HRESULT: 0x80070057`
- [OS == .WINDOWS] `E_UNEXPECTED :HRESULT: 0x8000FFFF`
- [OS == .WINDOWS] `SUCCEEDED :: inline (hr: HRESULT) -> bool`
- [OS == .WINDOWS] `FAILED :: inline (hr: HRESULT) -> bool`
- [OS == .WINDOWS] `WM_NULL ::0x0000`
- [OS == .WINDOWS] `WM_CREATE :: 0x0001`
- [OS == .WINDOWS] `WM_DESTROY :: 0x0002`
- [OS == .WINDOWS] `WM_MOVE :: 0x0003`
- [OS == .WINDOWS] `WM_SIZE :: 0x0005`
- [OS == .WINDOWS] `WM_SETFOCUS :: 0x0007`
- [OS == .WINDOWS] `WM_KILLFOCUS :: 0x0008`
- [OS == .WINDOWS] `WM_ENABLE :: 0x000A`
- [OS == .WINDOWS] `WM_SETREDRAW :: 0x000B`
- [OS == .WINDOWS] `WM_SETTEXT :: 0x000C`
- [OS == .WINDOWS] `WM_GETTEXT :: 0x000D`
- [OS == .WINDOWS] `WM_GETTEXTLENGTH :: 0x000E`
- [OS == .WINDOWS] `WM_PAINT :: 0x000F`
- [OS == .WINDOWS] `WM_CLOSE :: 0x0010`
- [OS == .WINDOWS] `WM_QUERYENDSESSION :: 0x0011`
- [OS == .WINDOWS] `WM_QUERYOPEN :: 0x0013`
- [OS == .WINDOWS] `WM_ENDSESSION :: 0x0016`
- [OS == .WINDOWS] `WM_QUIT :: 0x0012`
- [OS == .WINDOWS] `WM_ERASEBKGND :: 0x0014`
- [OS == .WINDOWS] `WM_SYSCOLORCHANGE :: 0x0015`
- [OS == .WINDOWS] `WM_SHOWWINDOW :: 0x0018`
- [OS == .WINDOWS] `WM_WININICHANGE :: 0x001A`
- [OS == .WINDOWS] `WM_SETTINGCHANGE :: WM_WININICHANGE`
- [OS == .WINDOWS] `WM_DEVMODECHANGE :: 0x001B`
- [OS == .WINDOWS] `WM_ACTIVATEAPP :: 0x001C`
- [OS == .WINDOWS] `WM_FONTCHANGE :: 0x001D`
- [OS == .WINDOWS] `WM_TIMECHANGE :: 0x001E`
- [OS == .WINDOWS] `WM_CANCELMODE :: 0x001F`
- [OS == .WINDOWS] `WM_SETCURSOR :: 0x0020`
- [OS == .WINDOWS] `WM_MOUSEACTIVATE :: 0x0021`
- [OS == .WINDOWS] `WM_CHILDACTIVATE :: 0x0022`
- [OS == .WINDOWS] `WM_QUEUESYNC :: 0x0023`
- [OS == .WINDOWS] `WM_GETMINMAXINFO :: 0x0024`
- [OS == .WINDOWS] `WM_WINDOWPOSCHANGED :: 0x0047`
- [OS == .WINDOWS] `WM_NOTIFY :: 0x004E`
- [OS == .WINDOWS] `WM_INPUTLANGCHANGEREQUEST :: 0x0050`
- [OS == .WINDOWS] `WM_INPUTLANGCHANGE :: 0x0051`
- [OS == .WINDOWS] `WM_TCARD :: 0x0052`
- [OS == .WINDOWS] `WM_HELP :: 0x0053`
- [OS == .WINDOWS] `WM_USERCHANGED :: 0x0054`
- [OS == .WINDOWS] `WM_NOTIFYFORMAT :: 0x0055`
- [OS == .WINDOWS] `WM_CONTEXTMENU :: 0x007B`
- [OS == .WINDOWS] `WM_STYLECHANGING :: 0x007C`
- [OS == .WINDOWS] `WM_STYLECHANGED :: 0x007D`
- [OS == .WINDOWS] `WM_DISPLAYCHANGE :: 0x007E`
- [OS == .WINDOWS] `WM_GETICON :: 0x007F`
- [OS == .WINDOWS] `WM_SETICON :: 0x0080`
- [OS == .WINDOWS] `WM_NCCREATE :: 0x0081`
- [OS == .WINDOWS] `WM_NCDESTROY :: 0x0082`
- [OS == .WINDOWS] `WM_NCCALCSIZE :: 0x0083`
- [OS == .WINDOWS] `WM_NCHITTEST :: 0x0084`
- [OS == .WINDOWS] `WM_NCPAINT :: 0x0085`
- [OS == .WINDOWS] `WM_NCACTIVATE :: 0x0086`
- [OS == .WINDOWS] `WM_GETDLGCODE :: 0x0087`
- [OS == .WINDOWS] `WM_SYNCPAINT :: 0x0088`
- [OS == .WINDOWS] `WM_NCMOUSEMOVE :: 0x00A0`
- [OS == .WINDOWS] `WM_NCLBUTTONDOWN :: 0x00A1`
- [OS == .WINDOWS] `WM_NCLBUTTONUP :: 0x00A2`
- [OS == .WINDOWS] `WM_NCLBUTTONDBLCLK :: 0x00A3`
- [OS == .WINDOWS] `WM_NCRBUTTONDOWN :: 0x00A4`
- [OS == .WINDOWS] `WM_NCRBUTTONUP :: 0x00A5`
- [OS == .WINDOWS] `WM_NCRBUTTONDBLCLK :: 0x00A6`
- [OS == .WINDOWS] `WM_NCMBUTTONDOWN :: 0x00A7`
- [OS == .WINDOWS] `WM_NCMBUTTONUP :: 0x00A8`
- [OS == .WINDOWS] `WM_NCMBUTTONDBLCLK :: 0x00A9`
- [OS == .WINDOWS] `WM_NCXBUTTONDOWN :: 0x00AB`
- [OS == .WINDOWS] `WM_NCXBUTTONUP :: 0x00AC`
- [OS == .WINDOWS] `WM_NCXBUTTONDBLCLK :: 0x00AD`
- [OS == .WINDOWS] `WM_INPUT_DEVICE_CHANGE :: 0x00FE`
- [OS == .WINDOWS] `WM_INPUT :: 0x00FF`
- [OS == .WINDOWS] `WM_KEYFIRST :: 0x0100`
- [OS == .WINDOWS] `WM_KEYDOWN :: 0x0100`
- [OS == .WINDOWS] `WM_KEYUP :: 0x0101`
- [OS == .WINDOWS] `WM_CHAR :: 0x0102`
- [OS == .WINDOWS] `WM_DEADCHAR :: 0x0103`
- [OS == .WINDOWS] `WM_SYSKEYDOWN :: 0x0104`
- [OS == .WINDOWS] `WM_SYSKEYUP :: 0x0105`
- [OS == .WINDOWS] `WM_SYSCHAR :: 0x0106`
- [OS == .WINDOWS] `WM_SYSDEADCHAR :: 0x0107`
- [OS == .WINDOWS] `WM_UNICHAR :: 0x0109`
- [OS == .WINDOWS] `WM_KEYLAST :: 0x0109`
- [OS == .WINDOWS] `UNICODE_NOCHAR :: 0xFFFF`
- [OS == .WINDOWS] `WM_IME_STARTCOMPOSITION :: 0x010D`
- [OS == .WINDOWS] `WM_IME_ENDCOMPOSITION :: 0x010E`
- [OS == .WINDOWS] `WM_IME_COMPOSITION :: 0x010F`
- [OS == .WINDOWS] `WM_IME_KEYLAST :: 0x010F`
- [OS == .WINDOWS] `WM_INITDIALOG :: 0x0110`
- [OS == .WINDOWS] `WM_COMMAND :: 0x0111`
- [OS == .WINDOWS] `WM_SYSCOMMAND :: 0x0112`
- [OS == .WINDOWS] `WM_TIMER :: 0x0113`
- [OS == .WINDOWS] `WM_HSCROLL :: 0x0114`
- [OS == .WINDOWS] `WM_VSCROLL :: 0x0115`
- [OS == .WINDOWS] `WM_INITMENU :: 0x0116`
- [OS == .WINDOWS] `WM_INITMENUPOPUP :: 0x0117`
- [OS == .WINDOWS] `WM_GESTURE :: 0x0119`
- [OS == .WINDOWS] `WM_GESTURENOTIFY :: 0x011A`
- [OS == .WINDOWS] `WM_MENUSELECT :: 0x011F`
- [OS == .WINDOWS] `WM_MENUCHAR :: 0x0120`
- [OS == .WINDOWS] `WM_ENTERIDLE :: 0x0121`
- [OS == .WINDOWS] `WM_MENURBUTTONUP :: 0x0122`
- [OS == .WINDOWS] `WM_MENUDRAG :: 0x0123`
- [OS == .WINDOWS] `WM_MENUGETOBJECT :: 0x0124`
- [OS == .WINDOWS] `WM_UNINITMENUPOPUP :: 0x0125`
- [OS == .WINDOWS] `WM_MENUCOMMAND :: 0x0126`
- [OS == .WINDOWS] `WM_CHANGEUISTATE :: 0x0127`
- [OS == .WINDOWS] `WM_UPDATEUISTATE :: 0x0128`
- [OS == .WINDOWS] `WM_QUERYUISTATE :: 0x0129`
- [OS == .WINDOWS] `WM_CTLCOLORMSGBOX :: 0x0132`
- [OS == .WINDOWS] `WM_CTLCOLOREDIT :: 0x0133`
- [OS == .WINDOWS] `WM_CTLCOLORLISTBOX :: 0x0134`
- [OS == .WINDOWS] `WM_CTLCOLORBTN :: 0x0135`
- [OS == .WINDOWS] `WM_CTLCOLORDLG :: 0x0136`
- [OS == .WINDOWS] `WM_CTLCOLORSCROLLBAR :: 0x0137`
- [OS == .WINDOWS] `WM_CTLCOLORSTATIC :: 0x0138`
- [OS == .WINDOWS] `MN_GETHMENU :: 0x01E1`
- [OS == .WINDOWS] `WM_MOUSEFIRST :: 0x0200`
- [OS == .WINDOWS] `WM_MOUSEMOVE :: 0x0200`
- [OS == .WINDOWS] `WM_LBUTTONDOWN :: 0x0201`
- [OS == .WINDOWS] `WM_LBUTTONUP :: 0x0202`
- [OS == .WINDOWS] `WM_LBUTTONDBLCLK :: 0x0203`
- [OS == .WINDOWS] `WM_RBUTTONDOWN :: 0x0204`
- [OS == .WINDOWS] `WM_RBUTTONUP :: 0x0205`
- [OS == .WINDOWS] `WM_RBUTTONDBLCLK :: 0x0206`
- [OS == .WINDOWS] `WM_MBUTTONDOWN :: 0x0207`
- [OS == .WINDOWS] `WM_MBUTTONUP :: 0x0208`
- [OS == .WINDOWS] `WM_MBUTTONDBLCLK :: 0x0209`
- [OS == .WINDOWS] `WM_MOUSEWHEEL :: 0x020A`
- [OS == .WINDOWS] `WM_XBUTTONDOWN :: 0x020B`
- [OS == .WINDOWS] `WM_XBUTTONUP :: 0x020C`
- [OS == .WINDOWS] `WM_XBUTTONDBLCLK :: 0x020D`
- [OS == .WINDOWS] `WM_MOUSEHWHEEL :: 0x020E`
- [OS == .WINDOWS] `WM_MOUSELAST :: 0x020E`
- [OS == .WINDOWS] `WM_SIZING :: 0x0214`
- [OS == .WINDOWS] `WM_DEVICECHANGE :: 0x0219`
- [OS == .WINDOWS] `WM_ENTERSIZEMOVE :: 0x0231`
- [OS == .WINDOWS] `WM_EXITSIZEMOVE :: 0x0232`
- [OS == .WINDOWS] `WM_DROPFILES :: 0x0233`
- [OS == .WINDOWS] `WM_TOUCH :: 0x0240`
- [OS == .WINDOWS] `WM_DPICHANGED :: 0x02E0`
- [OS == .WINDOWS] `SC_KEYMENU :: 0xF100`
- [OS == .WINDOWS] `HWND_TOP : HWND : null`
- [OS == .WINDOWS] `HWND_BOTTOM :: cast(HWND) 1`
- [OS == .WINDOWS] `HWND_TOPMOST :: cast(HWND) -1`
- [OS == .WINDOWS] `HWND_NOTOPMOST :: cast(HWND) -2`
- [OS == .WINDOWS] `WHITE_BRUSH :: 0`
- [OS == .WINDOWS] `LTGRAY_BRUSH :: 1`
- [OS == .WINDOWS] `GRAY_BRUSH :: 2`
- [OS == .WINDOWS] `DKGRAY_BRUSH :: 3`
- [OS == .WINDOWS] `BLACK_BRUSH :: 4`
- [OS == .WINDOWS] `NULL_BRUSH :: 5`
- [OS == .WINDOWS] `HOLLOW_BRUSH :: NULL_BRUSH`
- [OS == .WINDOWS] `WHITE_PEN :: 6`
- [OS == .WINDOWS] `BLACK_PEN :: 7`
- [OS == .WINDOWS] `NULL_PEN :: 8`
- [OS == .WINDOWS] `OEM_FIXED_FONT :: 10`
- [OS == .WINDOWS] `ANSI_FIXED_FONT :: 11`
- [OS == .WINDOWS] `ANSI_VAR_FONT :: 12`
- [OS == .WINDOWS] `SYSTEM_FONT :: 13`
- [OS == .WINDOWS] `DEVICE_DEFAULT_FONT :: 14`
- [OS == .WINDOWS] `DEFAULT_PALETTE :: 15`
- [OS == .WINDOWS] `SYSTEM_FIXED_FONT :: 16`
- [OS == .WINDOWS] `VK_LBUTTON :: 0x01`
- [OS == .WINDOWS] `VK_RBUTTON :: 0x02`
- [OS == .WINDOWS] `VK_CANCEL :: 0x03`
- [OS == .WINDOWS] `VK_MBUTTON :: 0x04`
- [OS == .WINDOWS] `VK_XBUTTON1 :: 0x05`
- [OS == .WINDOWS] `VK_XBUTTON2 :: 0x06`
- [OS == .WINDOWS] `VK_BACK :: 0x08`
- [OS == .WINDOWS] `VK_TAB :: 0x09`
- [OS == .WINDOWS] `VK_CLEAR :: 0x0C`
- [OS == .WINDOWS] `VK_RETURN :: 0x0D`
- [OS == .WINDOWS] `VK_SHIFT :: 0x10`
- [OS == .WINDOWS] `VK_CONTROL :: 0x11`
- [OS == .WINDOWS] `VK_MENU :: 0x12`
- [OS == .WINDOWS] `VK_PAUSE :: 0x13`
- [OS == .WINDOWS] `VK_CAPITAL :: 0x14`
- [OS == .WINDOWS] `VK_KANA :: 0x15`
- [OS == .WINDOWS] `VK_HANGUL :: 0x15`
- [OS == .WINDOWS] `VK_JUNJA :: 0x17`
- [OS == .WINDOWS] `VK_FINAL :: 0x18`
- [OS == .WINDOWS] `VK_HANJA :: 0x19`
- [OS == .WINDOWS] `VK_KANJI :: 0x19`
- [OS == .WINDOWS] `VK_ESCAPE :: 0x1B`
- [OS == .WINDOWS] `VK_CONVERT :: 0x1C`
- [OS == .WINDOWS] `VK_NONCONVERT :: 0x1D`
- [OS == .WINDOWS] `VK_ACCEPT :: 0x1E`
- [OS == .WINDOWS] `VK_MODECHANGE :: 0x1F`
- [OS == .WINDOWS] `VK_SPACE :: 0x20`
- [OS == .WINDOWS] `VK_PRIOR :: 0x21`
- [OS == .WINDOWS] `VK_NEXT :: 0x22`
- [OS == .WINDOWS] `VK_END :: 0x23`
- [OS == .WINDOWS] `VK_HOME :: 0x24`
- [OS == .WINDOWS] `VK_LEFT :: 0x25`
- [OS == .WINDOWS] `VK_UP :: 0x26`
- [OS == .WINDOWS] `VK_RIGHT :: 0x27`
- [OS == .WINDOWS] `VK_DOWN :: 0x28`
- [OS == .WINDOWS] `VK_SELECT :: 0x29`
- [OS == .WINDOWS] `VK_PRINT :: 0x2A`
- [OS == .WINDOWS] `VK_EXECUTE :: 0x2B`
- [OS == .WINDOWS] `VK_SNAPSHOT :: 0x2C`
- [OS == .WINDOWS] `VK_INSERT :: 0x2D`
- [OS == .WINDOWS] `VK_DELETE :: 0x2E`
- [OS == .WINDOWS] `VK_HELP :: 0x2F`
- [OS == .WINDOWS] `VK_LWIN :: 0x5B`
- [OS == .WINDOWS] `VK_RWIN :: 0x5C`
- [OS == .WINDOWS] `VK_APPS :: 0x5D`
- [OS == .WINDOWS] `VK_SLEEP :: 0x5F`
- [OS == .WINDOWS] `VK_NUMPAD0 :: 0x60`
- [OS == .WINDOWS] `VK_NUMPAD1 :: 0x61`
- [OS == .WINDOWS] `VK_NUMPAD2 :: 0x62`
- [OS == .WINDOWS] `VK_NUMPAD3 :: 0x63`
- [OS == .WINDOWS] `VK_NUMPAD4 :: 0x64`
- [OS == .WINDOWS] `VK_NUMPAD5 :: 0x65`
- [OS == .WINDOWS] `VK_NUMPAD6 :: 0x66`
- [OS == .WINDOWS] `VK_NUMPAD7 :: 0x67`
- [OS == .WINDOWS] `VK_NUMPAD8 :: 0x68`
- [OS == .WINDOWS] `VK_NUMPAD9 :: 0x69`
- [OS == .WINDOWS] `VK_MULTIPLY :: 0x6A`
- [OS == .WINDOWS] `VK_ADD :: 0x6B`
- [OS == .WINDOWS] `VK_SEPARATOR :: 0x6C`
- [OS == .WINDOWS] `VK_SUBTRACT :: 0x6D`
- [OS == .WINDOWS] `VK_DECIMAL :: 0x6E`
- [OS == .WINDOWS] `VK_DIVIDE :: 0x6F`
- [OS == .WINDOWS] `VK_F1 :: 0x70`
- [OS == .WINDOWS] `VK_F2 :: 0x71`
- [OS == .WINDOWS] `VK_F3 :: 0x72`
- [OS == .WINDOWS] `VK_F4 :: 0x73`
- [OS == .WINDOWS] `VK_F5 :: 0x74`
- [OS == .WINDOWS] `VK_F6 :: 0x75`
- [OS == .WINDOWS] `VK_F7 :: 0x76`
- [OS == .WINDOWS] `VK_F8 :: 0x77`
- [OS == .WINDOWS] `VK_F9 :: 0x78`
- [OS == .WINDOWS] `VK_F10 :: 0x79`
- [OS == .WINDOWS] `VK_F11 :: 0x7A`
- [OS == .WINDOWS] `VK_F12 :: 0x7B`
- [OS == .WINDOWS] `VK_F13 :: 0x7C`
- [OS == .WINDOWS] `VK_F14 :: 0x7D`
- [OS == .WINDOWS] `VK_F15 :: 0x7E`
- [OS == .WINDOWS] `VK_F16 :: 0x7F`
- [OS == .WINDOWS] `VK_F17 :: 0x80`
- [OS == .WINDOWS] `VK_F18 :: 0x81`
- [OS == .WINDOWS] `VK_F19 :: 0x82`
- [OS == .WINDOWS] `VK_F20 :: 0x83`
- [OS == .WINDOWS] `VK_F21 :: 0x84`
- [OS == .WINDOWS] `VK_F22 :: 0x85`
- [OS == .WINDOWS] `VK_F23 :: 0x86`
- [OS == .WINDOWS] `VK_F24 :: 0x87`
- [OS == .WINDOWS] `VK_NUMLOCK :: 0x90`
- [OS == .WINDOWS] `VK_SCROLL :: 0x91`
- [OS == .WINDOWS] `VK_OEM_NEC_EQUAL :: 0x92`
- [OS == .WINDOWS] `VK_OEM_FJ_JISHO :: 0x92`
- [OS == .WINDOWS] `VK_OEM_FJ_MASSHOU :: 0x93`
- [OS == .WINDOWS] `VK_OEM_FJ_TOUROKU :: 0x94`
- [OS == .WINDOWS] `VK_OEM_FJ_LOYA :: 0x95`
- [OS == .WINDOWS] `VK_OEM_FJ_ROYA :: 0x96`
- [OS == .WINDOWS] `VK_LSHIFT :: 0xA0`
- [OS == .WINDOWS] `VK_RSHIFT :: 0xA1`
- [OS == .WINDOWS] `VK_LCONTROL :: 0xA2`
- [OS == .WINDOWS] `VK_RCONTROL :: 0xA3`
- [OS == .WINDOWS] `VK_LMENU :: 0xA4`
- [OS == .WINDOWS] `VK_RMENU :: 0xA5`
- [OS == .WINDOWS] `VK_BROWSER_BACK :: 0xA6`
- [OS == .WINDOWS] `VK_BROWSER_FORWARD :: 0xA7`
- [OS == .WINDOWS] `VK_BROWSER_REFRESH :: 0xA8`
- [OS == .WINDOWS] `VK_BROWSER_STOP :: 0xA9`
- [OS == .WINDOWS] `VK_BROWSER_SEARCH :: 0xAA`
- [OS == .WINDOWS] `VK_BROWSER_FAVORITES :: 0xAB`
- [OS == .WINDOWS] `VK_BROWSER_HOME :: 0xAC`
- [OS == .WINDOWS] `VK_VOLUME_MUTE :: 0xAD`
- [OS == .WINDOWS] `VK_VOLUME_DOWN :: 0xAE`
- [OS == .WINDOWS] `VK_VOLUME_UP :: 0xAF`
- [OS == .WINDOWS] `VK_MEDIA_NEXT_TRACK :: 0xB0`
- [OS == .WINDOWS] `VK_MEDIA_PREV_TRACK :: 0xB1`
- [OS == .WINDOWS] `VK_MEDIA_STOP :: 0xB2`
- [OS == .WINDOWS] `VK_MEDIA_PLAY_PAUSE :: 0xB3`
- [OS == .WINDOWS] `VK_LAUNCH_MAIL :: 0xB4`
- [OS == .WINDOWS] `VK_LAUNCH_MEDIA_SELECT :: 0xB5`
- [OS == .WINDOWS] `VK_LAUNCH_APP1 :: 0xB6`
- [OS == .WINDOWS] `VK_LAUNCH_APP2 :: 0xB7`
- [OS == .WINDOWS] `VK_OEM_1 :: 0xBA`
- [OS == .WINDOWS] `VK_OEM_PLUS :: 0xBB`
- [OS == .WINDOWS] `VK_OEM_COMMA :: 0xBC`
- [OS == .WINDOWS] `VK_OEM_MINUS :: 0xBD`
- [OS == .WINDOWS] `VK_OEM_PERIOD :: 0xBE`
- [OS == .WINDOWS] `VK_OEM_2 :: 0xBF`
- [OS == .WINDOWS] `VK_OEM_3 :: 0xC0`
- [OS == .WINDOWS] `VK_OEM_4 :: 0xDB`
- [OS == .WINDOWS] `VK_OEM_5 :: 0xDC`
- [OS == .WINDOWS] `VK_OEM_6 :: 0xDD`
- [OS == .WINDOWS] `VK_OEM_7 :: 0xDE`
- [OS == .WINDOWS] `VK_OEM_8 :: 0xDF`
- [OS == .WINDOWS] `VK_OEM_AX :: 0xE1`
- [OS == .WINDOWS] `VK_OEM_102 :: 0xE2`
- [OS == .WINDOWS] `VK_ICO_HELP :: 0xE3`
- [OS == .WINDOWS] `VK_ICO_00 :: 0xE4`
- [OS == .WINDOWS] `VK_PROCESSKEY :: 0xE5`
- [OS == .WINDOWS] `VK_ICO_CLEAR :: 0xE6`
- [OS == .WINDOWS] `VK_PACKET :: 0xE7`
- [OS == .WINDOWS] `SIZE_RESTORED :: 0`
- [OS == .WINDOWS] `SIZE_MINIMIZED :: 1`
- [OS == .WINDOWS] `SIZE_MAXIMIZED :: 2`
- [OS == .WINDOWS] `SIZE_MAXSHOW :: 3`
- [OS == .WINDOWS] `SIZE_MAXHIDE :: 4`
- [OS == .WINDOWS] `SWP_NOSIZE :: 0x0001`
- [OS == .WINDOWS] `SWP_NOMOVE :: 0x0002`
- [OS == .WINDOWS] `SWP_NOZORDER :: 0x0004`
- [OS == .WINDOWS] `SWP_NOREDRAW :: 0x0008`
- [OS == .WINDOWS] `SWP_NOACTIVATE :: 0x0010`
- [OS == .WINDOWS] `SWP_FRAMECHANGED :: 0x0020`
- [OS == .WINDOWS] `SWP_SHOWWINDOW :: 0x0040`
- [OS == .WINDOWS] `SWP_HIDEWINDOW :: 0x0080`
- [OS == .WINDOWS] `SWP_NOCOPYBITS :: 0x0100`
- [OS == .WINDOWS] `SWP_NOOWNERZORDER :: 0x0200`
- [OS == .WINDOWS] `SWP_NOSENDCHANGING :: 0x0400`
- [OS == .WINDOWS] `SWP_DRAWFRAME :: SWP_FRAMECHANGED`
- [OS == .WINDOWS] `SWP_NOREPOSITION :: SWP_NOOWNERZORDER`
- [OS == .WINDOWS] `SWP_DEFERERASE :: 0x2000`
- [OS == .WINDOWS] `SWP_ASYNCWINDOWPOS :: 0x4000`
- [OS == .WINDOWS] `SW_HIDE :: 0`
- [OS == .WINDOWS] `SW_SHOWNORMAL :: 1`
- [OS == .WINDOWS] `SW_NORMAL :: 1`
- [OS == .WINDOWS] `SW_SHOWMINIMIZED :: 2`
- [OS == .WINDOWS] `SW_SHOWMAXIMIZED :: 3`
- [OS == .WINDOWS] `SW_MAXIMIZE :: 3`
- [OS == .WINDOWS] `SW_SHOWNOACTIVATE :: 4`
- [OS == .WINDOWS] `SW_SHOW :: 5`
- [OS == .WINDOWS] `SW_MINIMIZE :: 6`
- [OS == .WINDOWS] `SW_SHOWMINNOACTIVE :: 7`
- [OS == .WINDOWS] `SW_SHOWNA :: 8`
- [OS == .WINDOWS] `SW_RESTORE :: 9`
- [OS == .WINDOWS] `SW_SHOWDEFAULT :: 10`
- [OS == .WINDOWS] `SW_FORCEMINIMIZE :: 11`
- [OS == .WINDOWS] `SW_MAX :: 11`
- [OS == .WINDOWS] `CW_USEDEFAULT :: 0x80000000`
- [OS == .WINDOWS] `AdjustWindowRect :: (lpRect: *RECT, dwStyle: u32, bMenu: BOOL) -> BOOL #foreign user32`
- [OS == .WINDOWS] `AdjustWindowRectEx :: (lpRect: *RECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL #foreign user32`
- [OS == .WINDOWS] `CreateWindowExA :: (dwExStyle: u32, lpClassName: *u8, lpWindowName: *u8, dwStyle: u32, x: s32, y: s32, nWidth: s32, nHeight: s32, hwndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: *void) -> HWND #foreign user32`
- [OS == .WINDOWS] `CreateWindowExW :: (dwExStyle: u32, lpClassName: *u16, lpWindowName: *u16, dwStyle: u32, x: s32, y: s32, nWidth: s32, nHeight: s32, hwndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: *void) -> HWND #foreign user32`
- [OS == .WINDOWS] `CreateSolidBrush :: (color: u32) -> HBRUSH #foreign gdi`
- [OS == .WINDOWS] `DestroyWindow :: (hWnd: HWND) -> BOOL #foreign user32`
- [OS == .WINDOWS] `GetClientRect :: (hwnd: HWND, rect: *RECT) -> s32 #foreign user32`
- [OS == .WINDOWS] `GetDesktopWindow :: () -> HWND #foreign user32`
- [OS == .WINDOWS] `GetForegroundWindow :: () -> HWND #foreign user32`
- [OS == .WINDOWS] `GetWindowRect :: (hwnd: HWND, rect: *RECT) -> s32 #foreign user32`
- [OS == .WINDOWS] `SetProcessDPIAware :: () -> s32 #foreign user32`
- [OS == .WINDOWS] `SetWindowPos :: (hwnd: HWND, hWndInsertAfter: HWND, X: s32, Y: s32, cx: s32, cy: s32, uFlags: u32) -> BOOL #foreign user32`
- [OS == .WINDOWS] `SetWindowTextA :: (hWnd: HWND, lpString :*u8) -> BOOL #foreign user32`
- [OS == .WINDOWS] `SetWindowTextW :: (hWnd: HWND, lpString :*u16) -> BOOL #foreign user32`
- [OS == .WINDOWS] `ShowWindow :: (hwnd: HWND, nCmdShow: s32) -> s32 #foreign user32`
- [OS == .WINDOWS] `WindowFromPoint :: (Point: POINT) -> HWND #foreign user32`
- [OS == .WINDOWS] `WNDCLASSEXA :: struct`
- [OS == .WINDOWS] `WNDCLASSEXW :: struct`
- [OS == .WINDOWS] `CS_VREDRAW :: 0x0001`
- [OS == .WINDOWS] `CS_HREDRAW :: 0x0002`
- [OS == .WINDOWS] `CS_OWNDC :: 0x0020`
- [OS == .WINDOWS] `WS_OVERLAPPED :: 0x00000000`
- [OS == .WINDOWS] `WS_POPUP :: 0x80000000`
- [OS == .WINDOWS] `WS_CHILD :: 0x40000000`
- [OS == .WINDOWS] `WS_MINIMIZE :: 0x20000000`
- [OS == .WINDOWS] `WS_VISIBLE :: 0x10000000`
- [OS == .WINDOWS] `WS_DISABLED :: 0x08000000`
- [OS == .WINDOWS] `WS_CLIPSIBLINGS :: 0x04000000`
- [OS == .WINDOWS] `WS_CLIPCHILDREN :: 0x02000000`
- [OS == .WINDOWS] `WS_MAXIMIZE :: 0x01000000`
- [OS == .WINDOWS] `WS_CAPTION :: 0x00C00000`
- [OS == .WINDOWS] `WS_BORDER :: 0x00800000`
- [OS == .WINDOWS] `WS_DLGFRAME :: 0x00400000`
- [OS == .WINDOWS] `WS_VSCROLL :: 0x00200000`
- [OS == .WINDOWS] `WS_HSCROLL :: 0x00100000`
- [OS == .WINDOWS] `WS_SYSMENU :: 0x00080000`
- [OS == .WINDOWS] `WS_THICKFRAME :: 0x00040000`
- [OS == .WINDOWS] `WS_GROUP :: 0x00020000`
- [OS == .WINDOWS] `WS_TABSTOP :: 0x00010000`
- [OS == .WINDOWS] `WS_MINIMIZEBOX :: 0x00020000`
- [OS == .WINDOWS] `WS_MAXIMIZEBOX :: 0x00010000`
- [OS == .WINDOWS] `WS_SIZEBOX :: WS_THICKFRAME`
- [OS == .WINDOWS] `WS_OVERLAPPEDWINDOW :: WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX`
- [OS == .WINDOWS] `WS_EX_DLGMODALFRAME :: 0x00000001`
- [OS == .WINDOWS] `WS_EX_NOPARENTNOTIFY :: 0x00000004`
- [OS == .WINDOWS] `WS_EX_TOPMOST :: 0x00000008`
- [OS == .WINDOWS] `WS_EX_ACCEPTFILES :: 0x00000010`
- [OS == .WINDOWS] `WS_EX_TRANSPARENT :: 0x00000020`
- [OS == .WINDOWS] `WS_EX_MDICHILD :: 0x00000040`
- [OS == .WINDOWS] `WS_EX_TOOLWINDOW :: 0x00000080`
- [OS == .WINDOWS] `WS_EX_WINDOWEDGE :: 0x00000100`
- [OS == .WINDOWS] `WS_EX_CLIENTEDGE :: 0x00000200`
- [OS == .WINDOWS] `WS_EX_CONTEXTHELP :: 0x00000400`
- [OS == .WINDOWS] `WS_EX_RIGHT :: 0x00001000`
- [OS == .WINDOWS] `WS_EX_LEFT :: 0x00000000`
- [OS == .WINDOWS] `WS_EX_RTLREADING :: 0x00002000`
- [OS == .WINDOWS] `WS_EX_LTRREADING :: 0x00000000`
- [OS == .WINDOWS] `WS_EX_LEFTSCROLLBAR :: 0x00004000`
- [OS == .WINDOWS] `WS_EX_RIGHTSCROLLBAR :: 0x00000000`
- [OS == .WINDOWS] `WS_EX_CONTROLPARENT :: 0x00010000`
- [OS == .WINDOWS] `WS_EX_STATICEDGE :: 0x00020000`
- [OS == .WINDOWS] `WS_EX_APPWINDOW :: 0x00040000`
- [OS == .WINDOWS] `WS_EX_OVERLAPPEDWINDOW :: (WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE)`
- [OS == .WINDOWS] `WS_EX_PALETTEWINDOW :: (WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST)`
- [OS == .WINDOWS] `WS_EX_LAYERED :: 0x00080000`
- [OS == .WINDOWS] `WS_EX_NOINHERITLAYOUT :: 0x00100000`
- [OS == .WINDOWS] `WS_EX_NOREDIRECTIONBITMAP :: 0x00200000`
- [OS == .WINDOWS] `WS_EX_LAYOUTRTL :: 0x00400000`
- [OS == .WINDOWS] `WS_EX_COMPOSITED :: 0x02000000`
- [OS == .WINDOWS] `WS_EX_NOACTIVATE :: 0x08000000`
- [OS == .WINDOWS] `GWL_WNDPROC :: -4`
- [OS == .WINDOWS] `GWL_HINSTANCE :: -6`
- [OS == .WINDOWS] `GWL_HWNDPARENT :: -8`
- [OS == .WINDOWS] `GWL_STYLE :: -16`
- [OS == .WINDOWS] `GWL_EXSTYLE :: -20`
- [OS == .WINDOWS] `GWL_USERDATA :: -21`
- [OS == .WINDOWS] `GWL_ID :: -12`
- [OS == .WINDOWS] `GetWindowLongA :: (hwnd: HWND, nIndex: s32) -> s32 #foreign user32`
- [OS == .WINDOWS] `GetWindowLongW :: (hwnd: HWND, nIndex: s32) -> s32 #foreign user32`
- [OS == .WINDOWS] `RegisterClassExA :: (lpwcx: *WNDCLASSEXA) -> s16 #foreign user32`
- [OS == .WINDOWS] `RegisterClassExW :: (lpwcx: *WNDCLASSEXW) -> s16 #foreign user32`
- [OS == .WINDOWS] `SetWindowLongA :: (hwnd: HWND, nIndex: s32, dwNewLong: s32) -> s32 #foreign user32`
- [OS == .WINDOWS] `SetWindowLongW :: (hwnd: HWND, nIndex: s32, dwNewLong: s32) -> s32 #foreign user32`
- [OS == .WINDOWS] `SetWindowLongPtrW :: (hwnd: HWND, nIndex: s32, dwNewLong: s64) -> s64 #foreign user32`
- [OS == .WINDOWS] `UnregisterClassA :: (lpClassName: LPCSTR, hInstance: HINSTANCE) -> BOOL #foreign user32`
- [OS == .WINDOWS] `UnregisterClassW :: (lpClassName: LPCWSTR, hInstance: HINSTANCE) -> BOOL #foreign user32`
- [OS == .WINDOWS] `ClientToScreen :: (hwnd: HWND, lpPoint: *POINT) -> s32 #foreign user32`
- [OS == .WINDOWS] `ScreenToClient :: (hwnd: HWND, p: *POINT) -> s32 #foreign user32`
- [OS == .WINDOWS] `GetWindowDC :: (hwnd: HWND) -> HDC #foreign user32`
- [OS == .WINDOWS] `UpdateWindow :: (hwnd: HWND) -> s32 #foreign user32`
- [OS == .WINDOWS] `ValidateRect :: (hwnd: HWND, lpRect: *RECT) -> BOOL #foreign user32`
- [OS == .WINDOWS] `CreateCompatibleDC :: (hdc: HDC) -> HDC #foreign gdi`
- [OS == .WINDOWS] `DeleteDC :: (hdc: HDC) -> BOOL #foreign gdi`
- [OS == .WINDOWS] `DeleteObject :: (ho: HGDIOBJ) -> BOOL #foreign gdi`
- [OS == .WINDOWS] `POINTL :: struct`
- [OS == .WINDOWS] `DM_ORIENTATION :: 0x00000001`
- [OS == .WINDOWS] `DM_PAPERSIZE :: 0x00000002`
- [OS == .WINDOWS] `DM_PAPERLENGTH :: 0x00000004`
- [OS == .WINDOWS] `DM_PAPERWIDTH :: 0x00000008`
- [OS == .WINDOWS] `DM_SCALE :: 0x00000010`
- [OS == .WINDOWS] `DM_POSITION :: 0x00000020`
- [OS == .WINDOWS] `DM_NUP :: 0x00000040`
- [OS == .WINDOWS] `DM_DISPLAYORIENTATION :: 0x00000080`
- [OS == .WINDOWS] `DM_COPIES :: 0x00000100`
- [OS == .WINDOWS] `DM_DEFAULTSOURCE :: 0x00000200`
- [OS == .WINDOWS] `DM_PRINTQUALITY :: 0x00000400`
- [OS == .WINDOWS] `DM_COLOR :: 0x00000800`
- [OS == .WINDOWS] `DM_DUPLEX :: 0x00001000`
- [OS == .WINDOWS] `DM_YRESOLUTION :: 0x00002000`
- [OS == .WINDOWS] `DM_TTOPTION :: 0x00004000`
- [OS == .WINDOWS] `DM_COLLATE :: 0x00008000`
- [OS == .WINDOWS] `DM_FORMNAME :: 0x00010000`
- [OS == .WINDOWS] `DM_LOGPIXELS :: 0x00020000`
- [OS == .WINDOWS] `DM_BITSPERPEL :: 0x00040000`
- [OS == .WINDOWS] `DM_PELSWIDTH :: 0x00080000`
- [OS == .WINDOWS] `DM_PELSHEIGHT :: 0x00100000`
- [OS == .WINDOWS] `DM_DISPLAYFLAGS :: 0x00200000`
- [OS == .WINDOWS] `DM_DISPLAYFREQUENCY :: 0x00400000`
- [OS == .WINDOWS] `DM_ICMMETHOD :: 0x00800000`
- [OS == .WINDOWS] `DM_ICMINTENT :: 0x01000000`
- [OS == .WINDOWS] `DM_MEDIATYPE :: 0x02000000`
- [OS == .WINDOWS] `DM_DITHERTYPE :: 0x04000000`
- [OS == .WINDOWS] `DM_PANNINGWIDTH :: 0x08000000`
- [OS == .WINDOWS] `DM_PANNINGHEIGHT :: 0x10000000`
- [OS == .WINDOWS] `DM_DISPLAYFIXEDOUTPUT :: 0x20000000`
- [OS == .WINDOWS] `DEVMODEA :: struct`
- [OS == .WINDOWS] `ENUM_CURRENT_SETTINGS :: cast,no_check(u32, -1)`
- [OS == .WINDOWS] `ENUM_REGISTRY_SETTINGS :: cast,no_check(u32, -2)`
- [OS == .WINDOWS] `EnumDisplaySettingsA :: (lpszDeviceName: *u8, iModeNum: u32, lpDevMode: *DEVMODEA) -> BOOL #foreign user32`
- [OS == .WINDOWS] `GetDC :: (hwnd: HWND) -> HDC #foreign user32`
- [OS == .WINDOWS] `PW_CLIENTONLY :: 0x1`
- [OS == .WINDOWS] `PW_RENDERFULLCONTENT :: 0x2`
- [OS == .WINDOWS] `PrintWindow :: (hwnd: HWND, hdcBlt: HDC, nFlags: u32) -> BOOL #foreign user32`
- [OS == .WINDOWS] `DRIVERVERSION :: 0`
- [OS == .WINDOWS] `TECHNOLOGY :: 2`
- [OS == .WINDOWS] `HORZSIZE :: 4`
- [OS == .WINDOWS] `VERTSIZE :: 6`
- [OS == .WINDOWS] `HORZRES :: 8`
- [OS == .WINDOWS] `VERTRES :: 10`
- [OS == .WINDOWS] `BITSPIXEL :: 12`
- [OS == .WINDOWS] `PLANES :: 14`
- [OS == .WINDOWS] `NUMBRUSHES :: 16`
- [OS == .WINDOWS] `NUMPENS :: 18`
- [OS == .WINDOWS] `NUMMARKERS :: 20`
- [OS == .WINDOWS] `NUMFONTS :: 22`
- [OS == .WINDOWS] `NUMCOLORS :: 24`
- [OS == .WINDOWS] `PDEVICESIZE :: 26`
- [OS == .WINDOWS] `CURVECAPS :: 28`
- [OS == .WINDOWS] `LINECAPS :: 30`
- [OS == .WINDOWS] `POLYGONALCAPS :: 32`
- [OS == .WINDOWS] `TEXTCAPS :: 34`
- [OS == .WINDOWS] `CLIPCAPS :: 36`
- [OS == .WINDOWS] `RASTERCAPS :: 38`
- [OS == .WINDOWS] `ASPECTX :: 40`
- [OS == .WINDOWS] `ASPECTY :: 42`
- [OS == .WINDOWS] `ASPECTXY :: 44`
- [OS == .WINDOWS] `LOGPIXELSX :: 88`
- [OS == .WINDOWS] `LOGPIXELSY :: 90`
- [OS == .WINDOWS] `SIZEPALETTE :: 104`
- [OS == .WINDOWS] `NUMRESERVED :: 106`
- [OS == .WINDOWS] `COLORRES :: 108`
- [OS == .WINDOWS] `PHYSICALWIDTH :: 110`
- [OS == .WINDOWS] `PHYSICALHEIGHT :: 111`
- [OS == .WINDOWS] `PHYSICALOFFSETX :: 112`
- [OS == .WINDOWS] `PHYSICALOFFSETY :: 113`
- [OS == .WINDOWS] `SCALINGFACTORX :: 114`
- [OS == .WINDOWS] `SCALINGFACTORY :: 115`
- [OS == .WINDOWS] `VREFRESH :: 116`
- [OS == .WINDOWS] `DESKTOPVERTRES :: 117`
- [OS == .WINDOWS] `DESKTOPHORZRES :: 118`
- [OS == .WINDOWS] `BLTALIGNMENT :: 119`
- [OS == .WINDOWS] `SHADEBLENDCAPS :: 120`
- [OS == .WINDOWS] `COLORMGMTCAPS :: 121`
- [OS == .WINDOWS] `GetDeviceCaps :: (hdc: HDC, index: s32) -> s32 #foreign gdi`
- [OS == .WINDOWS] `GetObjectA :: (h: HANDLE, c: s32, pv: *void) -> s32 #foreign gdi`
- [OS == .WINDOWS] `GetObjectW :: (h: HANDLE, c: s32, pv: *void) -> s32 #foreign gdi`
- [OS == .WINDOWS] `GetStockObject :: (fnObject: s32) -> HANDLE #foreign gdi`
- [OS == .WINDOWS] `ReleaseDC :: (hwnd: HWND, hdc: HDC) -> s32 #foreign user32`
- [OS == .WINDOWS] `SelectObject :: (hdc: HDC, h: HGDIOBJ) -> HGDIOBJ #foreign gdi`
- [OS == .WINDOWS] `WindowFromDC :: (dc: HDC) -> HWND #foreign user32`
- [OS == .WINDOWS] `ICONINFO :: struct`
- [OS == .WINDOWS] `ICONINFOEXW :: struct`
- [OS == .WINDOWS] `BITMAP :: struct`
- [OS == .WINDOWS] `RGBQUAD :: struct`
- [OS == .WINDOWS] `BITMAPINFO :: struct`
- [OS == .WINDOWS] `BITMAPFILEHEADER :: struct`
- [OS == .WINDOWS] `ExtractIconA :: (hInst: HINSTANCE, lpszExeFileName: *u8, nIconIndex: u32) -> HICON #foreign shell32`
- [OS == .WINDOWS] `ExtractIconW :: (hInst: HINSTANCE, lpszExeFileName: *u16, nIconIndex: u32) -> HICON #foreign shell32`
- [OS == .WINDOWS] `GetIconInfo :: (hIcon: HICON, piconinfo: *ICONINFO) -> BOOL #foreign user32`
- [OS == .WINDOWS] `GetIconInfoExA :: (hIcon: HICON, piconinfo: *void) -> BOOL #foreign user32`
- [OS == .WINDOWS] `GetIconInfoExW :: (hIcon: HICON, piconinfo: *ICONINFOEXW) -> BOOL #foreign user32`
- [OS == .WINDOWS] `LoadIconA :: (hInstance: HINSTANCE, lpIconName: *u8) -> HICON #foreign user32`
- [OS == .WINDOWS] `LoadIconW :: (hInstance: HINSTANCE, lpIconName: *u16) -> HICON #foreign user32`
- [OS == .WINDOWS] `IDC_APPSTARTING :: cast(*void) 32650`
- [OS == .WINDOWS] `IDC_ARROW :: cast(*void) 32512`
- [OS == .WINDOWS] `IDC_CROSS :: cast(*void) 32515`
- [OS == .WINDOWS] `IDC_HAND :: cast(*void) 32649`
- [OS == .WINDOWS] `IDC_HELP :: cast(*void) 32651`
- [OS == .WINDOWS] `IDC_IBEAM :: cast(*void) 32513`
- [OS == .WINDOWS] `IDC_ICON :: cast(*void) 32641`
- [OS == .WINDOWS] `IDC_NO :: cast(*void) 32648`
- [OS == .WINDOWS] `IDC_SIZE :: cast(*void) 32640`
- [OS == .WINDOWS] `IDC_SIZEALL :: cast(*void) 32646`
- [OS == .WINDOWS] `IDC_SIZENESW :: cast(*void) 32643`
- [OS == .WINDOWS] `IDC_SIZENS :: cast(*void) 32645`
- [OS == .WINDOWS] `IDC_SIZENWSE :: cast(*void) 32642`
- [OS == .WINDOWS] `IDC_SIZEWE :: cast(*void) 32644`
- [OS == .WINDOWS] `IDC_UPARROW :: cast(*void) 32516`
- [OS == .WINDOWS] `IDC_WAIT :: cast(*void) 32514`
- [OS == .WINDOWS] `ClipCursor :: (lpRect: *RECT) -> s32 #foreign user32`
- [OS == .WINDOWS] `GetCursorPos :: (p: *POINT) -> s32 #foreign user32`
- [OS == .WINDOWS] `LoadCursorA :: (hInstance: HINSTANCE, lpCursorName: *u8) -> HCURSOR #foreign user32`
- [OS == .WINDOWS] `LoadCursorW :: (hInstance: HINSTANCE, lpCursorName: *s16) -> HCURSOR #foreign user32`
- [OS == .WINDOWS] `SetCursor :: (cursor: HCURSOR) #foreign user32`
- [OS == .WINDOWS] `ShowCursor :: (bShow: s32) -> s32 #foreign user32`
- [OS == .WINDOWS] `MSG :: struct`
- [OS == .WINDOWS] `PM_NOREMOVE :: 0x0000`
- [OS == .WINDOWS] `PM_REMOVE :: 0x0001`
- [OS == .WINDOWS] `PM_NOYIELD :: 0x0002`
- [OS == .WINDOWS] `DispatchMessageA :: (msg: *MSG) -> s32 #foreign user32`
- [OS == .WINDOWS] `DispatchMessageW :: (msg: *MSG) -> s32 #foreign user32`
- [OS == .WINDOWS] `GetMessageExtraInfo :: () -> LPARAM #foreign user32`
- [OS == .WINDOWS] `PeekMessageA :: (msg: *MSG, hwnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32) -> s32 #foreign user32`
- [OS == .WINDOWS] `PeekMessageW :: (msg: *MSG, hwnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32) -> s32 #foreign user32`
- [OS == .WINDOWS] `PostQuitMessage :: (index: s32) -> void #foreign user32`
- [OS == .WINDOWS] `TranslateMessage :: (msg: *MSG) -> s32 #foreign user32`
- [OS == .WINDOWS] `DefWindowProcA :: (hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> s64 #foreign user32`
- [OS == .WINDOWS] `DefWindowProcW :: (hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> s64 #foreign user32`
- [OS == .WINDOWS] `MONITOR_DEFAULTTONULL :: 0x00000000`
- [OS == .WINDOWS] `MONITOR_DEFAULTTOPRIMARY :: 0x00000001`
- [OS == .WINDOWS] `MONITOR_DEFAULTTONEAREST :: 0x00000002`
- [OS == .WINDOWS] `MONITORINFO :: struct`
- [OS == .WINDOWS] `CCHDEVICENAME :: 32`
- [OS == .WINDOWS] `CCHFORMNAME :: 32`
- [OS == .WINDOWS] `MONITORINFOEXA :: struct`
- [OS == .WINDOWS] `EnumDisplayMonitors :: (hdc: HDC, lprcClip: *RECT, lpfnEnum:MONITORENUMPROC, dwData:LPARAM) -> BOOL #foreign user32`
- [OS == .WINDOWS] `GetMonitorInfoA :: (hMonitor: HMONITOR, lpmi: *MONITORINFO) -> BOOL #foreign user32`
- [OS == .WINDOWS] `GetMonitorInfoW :: (hMonitor: HMONITOR, lpmi: *MONITORINFO) -> BOOL #foreign user32`
- [OS == .WINDOWS] `MONITORENUMPROC :: #type (hMonitor: HMONITOR, hdcMonitor: HDC, lprcMonitor: *RECT, dwData: LPARAM) -> BOOL #c_call`
- [OS == .WINDOWS] `MonitorFromPoint :: (pt: POINT, dwFlags: u32) -> HMONITOR #foreign user32`
- [OS == .WINDOWS] `MonitorFromRect :: (lprc: *RECT, dwFlags: u32) -> HMONITOR #foreign user32`
- [OS == .WINDOWS] `MonitorFromWindow :: (hwnd: HWND, dwFlags: u32) -> HMONITOR #foreign user32`
- [OS == .WINDOWS] `GetActiveWindow :: () -> HWND #foreign user32`
- [OS == .WINDOWS] `GetAsyncKeyState :: (nVirtKey: s32) -> s16 #foreign user32`
- [OS == .WINDOWS] `GetFocus :: () -> HWND #foreign user32`
- [OS == .WINDOWS] `GetKeyState :: (nVirtKey: s32) -> s16 #foreign user32`
- [OS == .WINDOWS] `SetActiveWindow :: (hwnd: HWND) -> HWND #foreign user32`
- [OS == .WINDOWS] `ReleaseCapture :: () -> BOOL #foreign user32`
- [OS == .WINDOWS] `SetCapture :: (hwnd: HWND) -> HWND #foreign user32`
- [OS == .WINDOWS] `RAWINPUTHEADER :: struct`
- [OS == .WINDOWS] `RAWINPUT :: struct`
- [OS == .WINDOWS] `RAWMOUSE :: struct`
- [OS == .WINDOWS] `RAWHID :: struct`
- [OS == .WINDOWS] `RAWKEYBOARD :: struct`
- [OS == .WINDOWS] `RAWINPUTDEVICE :: struct`
- [OS == .WINDOWS] `RID_INPUT :: 0x10000003`
- [OS == .WINDOWS] `RID_HEADER :: 0x10000005`
- [OS == .WINDOWS] `RIM_INPUT :: 0x00000000`
- [OS == .WINDOWS] `RIM_INPUTSINK :: 0x00000001`
- [OS == .WINDOWS] `RIM_TYPEMOUSE :: 0x00000000`
- [OS == .WINDOWS] `RIM_TYPEKEYBOARD :: 0x00000001`
- [OS == .WINDOWS] `RIM_TYPEHID :: 0x00000002`
- [OS == .WINDOWS] `MOUSE_MOVE_RELATIVE :: 0x00`
- [OS == .WINDOWS] `MOUSE_MOVE_ABSOLUTE :: 0x01`
- [OS == .WINDOWS] `MOUSE_VIRTUAL_DESKTOP :: 0x02`
- [OS == .WINDOWS] `MOUSE_ATTRIBUTES_CHANGED :: 0x04`
- [OS == .WINDOWS] `MOUSE_MOVE_NOCOALESCE :: 0x08`
- [OS == .WINDOWS] `RI_MOUSE_LEFT_BUTTON_DOWN :: 0x0001`
- [OS == .WINDOWS] `RI_MOUSE_LEFT_BUTTON_UP :: 0x0002`
- [OS == .WINDOWS] `RI_MOUSE_RIGHT_BUTTON_DOWN :: 0x0004`
- [OS == .WINDOWS] `RI_MOUSE_RIGHT_BUTTON_UP :: 0x0008`
- [OS == .WINDOWS] `RI_MOUSE_MIDDLE_BUTTON_DOWN :: 0x0010`
- [OS == .WINDOWS] `RI_MOUSE_MIDDLE_BUTTON_UP :: 0x0020`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_1_DOWN :: RI_MOUSE_LEFT_BUTTON_DOWN`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_1_UP :: RI_MOUSE_LEFT_BUTTON_UP`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_2_DOWN :: RI_MOUSE_RIGHT_BUTTON_DOWN`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_2_UP :: RI_MOUSE_RIGHT_BUTTON_UP`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_3_DOWN :: RI_MOUSE_MIDDLE_BUTTON_DOWN`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_3_UP :: RI_MOUSE_MIDDLE_BUTTON_UP`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_4_DOWN :: 0x0040`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_4_UP :: 0x0080`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_5_DOWN :: 0x0100`
- [OS == .WINDOWS] `RI_MOUSE_BUTTON_5_UP :: 0x0200`
- [OS == .WINDOWS] `RI_MOUSE_WHEEL :: 0x0400`
- [OS == .WINDOWS] `RI_KEY_MAKE :: 0`
- [OS == .WINDOWS] `RI_KEY_BREAK :: 1`
- [OS == .WINDOWS] `RI_KEY_E0 :: 2`
- [OS == .WINDOWS] `RI_KEY_E1 :: 4`
- [OS == .WINDOWS] `GetRawInputData :: (hRawInput: HRAWINPUT, uiCommand: u32, pData: *void, pcbSize: *u32, cbSizeHeader: u32) -> u32 #foreign user32`
- [OS == .WINDOWS] `GetRawInputDeviceInfoA :: (hDevice: HANDLE, uiCommand: u32, pData: *void, pcbSize: *u32) -> u32 #foreign user32`
- [OS == .WINDOWS] `RegisterRawInputDevices :: (pRawInputDevices: *void, uiNumDevices: u32, cbSize: u32) -> s32 #foreign user32`
- [OS == .WINDOWS] `DeviceIoControl :: (hDevice: HANDLE, dwIoControlCode: u32, lpInBuffer: *void, nInBufferSize: u32, lpOutBuffer: *void, nOutBufferSize: u32, lpBytesReturned: *u32, lpOverlapped: *OVERLAPPED) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `DEV_BROADCAST_HDR :: struct`
- [OS == .WINDOWS] `DBT_DEVTYP_DEVICEINTERFACE :: 0x00000005`
- [OS == .WINDOWS] `DEV_BROADCAST_DEVICEINTERFACE_W :: struct`
- [OS == .WINDOWS] `DEVICE_NOTIFY_WINDOW_HANDLE : u32 : 0`
- [OS == .WINDOWS] `XBOX_GUID :: GUID.{ 0xec87f1e3, 0xc13b, 0x4100, .[ 0xb5, 0xf7, 0x8b, 0x84, 0xd5, 0x42, 0x60, 0xcb ] }`
- [OS == .WINDOWS] `GUID_DEVINTERFACE_HID :: GUID.{ 0x4d1e55b2, 0xf16f, 0x11cf, .[ 0x88, 0xcb, 0x00, 0x11, 0x11, 0x00, 0x00, 0x30 ] }`
- [OS == .WINDOWS] `HDEVNOTIFY :: *void`
- [OS == .WINDOWS] `RegisterDeviceNotificationW :: (hRecipient: HANDLE, NotificationFilter: *void, Flags: u32) -> HDEVNOTIFY #foreign user32`
- [OS == .WINDOWS] `DBT_DEVICEARRIVAL :: 0x8000`
- [OS == .WINDOWS] `DBT_DEVICEREMOVECOMPLETE :: 0x8004`
- [OS == .WINDOWS] `HDEVINFO :: *void`
- [OS == .WINDOWS] `DIGCF_DEFAULT :: 0x00000001`
- [OS == .WINDOWS] `DIGCF_PRESENT :: 0x00000002`
- [OS == .WINDOWS] `DIGCF_ALLCLASSES :: 0x00000004`
- [OS == .WINDOWS] `DIGCF_PROFILE :: 0x00000008`
- [OS == .WINDOWS] `DIGCF_DEVICEINTERFACE :: 0x00000010`
- [OS == .WINDOWS] `SetupDiGetClassDevsW :: (ClassGuid: *GUID, Enumerator: *u16, hwndParent: HWND, Flags: u32) -> HDEVINFO #foreign setupapi`
- [OS == .WINDOWS] `SP_DEVICE_INTERFACE_DATA :: struct`
- [OS == .WINDOWS] `SP_DEVINFO_DATA :: struct`
- [OS == .WINDOWS] `SetupDiEnumDeviceInterfaces :: (DeviceInfoSet: HDEVINFO, DeviceInfoData: *SP_DEVINFO_DATA, InterfaceClassGuid: *GUID, MemberIndex: u32, DeviceInterfaceData: *SP_DEVICE_INTERFACE_DATA) -> BOOL #foreign setupapi`
- [OS == .WINDOWS] `SP_DEVICE_INTERFACE_DETAIL_DATA_W :: struct`
- [OS == .WINDOWS] `SetupDiGetDeviceInterfaceDetailW :: (DeviceInfoSet: HDEVINFO, DeviceInterfaceData: *SP_DEVICE_INTERFACE_DATA, DeviceInterfaceDetailData: *SP_DEVICE_INTERFACE_DETAIL_DATA_W, DeviceInterfaceDetailDataSize: u32, RequiredSize: *u32, DeviceInfoData: *SP_DEVINFO_DATA) -> BOOL #foreign setupapi`
- [OS == .WINDOWS] `SetupDiDestroyDeviceInfoList :: (DeviceInfoSet: HDEVINFO) -> BOOL #foreign setupapi`
- [OS == .WINDOWS] `HTOUCHINPUT :: HANDLE`
- [OS == .WINDOWS] `TOUCHINPUT :: struct`
- [OS == .WINDOWS] `TOUCH_COORD_TO_PIXEL :: (c: s32) -> float #expand`
- [OS == .WINDOWS] `TOUCHEVENTF_MOVE :: 0x0001`
- [OS == .WINDOWS] `TOUCHEVENTF_DOWN :: 0x0002`
- [OS == .WINDOWS] `TOUCHEVENTF_UP :: 0x0004`
- [OS == .WINDOWS] `TOUCHEVENTF_INRANGE :: 0x0008`
- [OS == .WINDOWS] `TOUCHEVENTF_PRIMARY :: 0x0010`
- [OS == .WINDOWS] `TOUCHEVENTF_NOCOALESCE :: 0x0020`
- [OS == .WINDOWS] `TOUCHEVENTF_PEN :: 0x0040`
- [OS == .WINDOWS] `TOUCHEVENTF_PALM :: 0x0080`
- [OS == .WINDOWS] `TOUCHINPUTMASKF_TIMEFROMSYSTEM :: 0x0001`
- [OS == .WINDOWS] `TOUCHINPUTMASKF_EXTRAINFO :: 0x0002`
- [OS == .WINDOWS] `TOUCHINPUTMASKF_CONTACTAREA :: 0x0004`
- [OS == .WINDOWS] `GetTouchInputInfo :: (hTouchInput: HTOUCHINPUT, cInputs: u32, pInputs: *TOUCHINPUT, cbSize: s32) -> BOOL #foreign user32`
- [OS == .WINDOWS] `CloseTouchInputHandle :: (hTouchInput: HTOUCHINPUT) -> BOOL #foreign user32`
- [OS == .WINDOWS] `TWF_FINETOUCH :: 0x00000001`
- [OS == .WINDOWS] `TWF_WANTPALM :: 0x00000002`
- [OS == .WINDOWS] `RegisterTouchWindow :: (hwnd: HWND, ulFlags: u32) -> BOOL #foreign user32`
- [OS == .WINDOWS] `UnregisterTouchWindow :: (hwnd: HWND) -> BOOL #foreign user32`
- [OS == .WINDOWS] `IsTouchWindow :: (hwnd: HWND, pulFlags: *u32) -> BOOL #foreign user32`
- [OS == .WINDOWS] `PIXELFORMATDESCRIPTOR :: struct`
- [OS == .WINDOWS] `PFD_DOUBLEBUFFER :: 0x00000001`
- [OS == .WINDOWS] `PFD_STEREO :: 0x00000002`
- [OS == .WINDOWS] `PFD_DRAW_TO_WINDOW :: 0x00000004`
- [OS == .WINDOWS] `PFD_DRAW_TO_BITMAP :: 0x00000008`
- [OS == .WINDOWS] `PFD_SUPPORT_GDI :: 0x00000010`
- [OS == .WINDOWS] `PFD_SUPPORT_OPENGL :: 0x00000020`
- [OS == .WINDOWS] `PFD_GENERIC_FORMAT :: 0x00000040`
- [OS == .WINDOWS] `PFD_NEED_PALETTE :: 0x00000080`
- [OS == .WINDOWS] `PFD_NEED_SYSTEM_PALETTE :: 0x00000100`
- [OS == .WINDOWS] `PFD_SWAP_EXCHANGE :: 0x00000200`
- [OS == .WINDOWS] `PFD_SWAP_COPY :: 0x00000400`
- [OS == .WINDOWS] `PFD_SWAP_LAYER_BUFFERS :: 0x00000800`
- [OS == .WINDOWS] `PFD_GENERIC_ACCELERATED :: 0x00001000`
- [OS == .WINDOWS] `PFD_SUPPORT_DIRECTDRAW :: 0x00002000`
- [OS == .WINDOWS] `PFD_DIRECT3D_ACCELERATED :: 0x00004000`
- [OS == .WINDOWS] `PFD_SUPPORT_COMPOSITION :: 0x00008000`
- [OS == .WINDOWS] `PFD_TYPE_RGBA :: 0`
- [OS == .WINDOWS] `PFD_TYPE_COLORINDEX :: 1`
- [OS == .WINDOWS] `PFD_MAIN_PLANE :: 0`
- [OS == .WINDOWS] `PFD_OVERLAY_PLANE :: 1`
- [OS == .WINDOWS] `PFD_UNDERLAY_PLANE :: -1`
- [OS == .WINDOWS] `WGL_FONT_LINES :: 0`
- [OS == .WINDOWS] `WGL_FONT_POLYGONS :: 1`
- [OS == .WINDOWS] `WGL_SWAP_MAIN_PLANE :: 0x00000001`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY1 :: 0x00000002`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY2 :: 0x00000004`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY3 :: 0x00000008`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY4 :: 0x00000010`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY5 :: 0x00000020`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY6 :: 0x00000040`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY7 :: 0x00000080`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY8 :: 0x00000100`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY9 :: 0x00000200`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY10 :: 0x00000400`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY11 :: 0x00000800`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY12 :: 0x00001000`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY13 :: 0x00002000`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY14 :: 0x00004000`
- [OS == .WINDOWS] `WGL_SWAP_OVERLAY15 :: 0x00008000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY1 :: 0x00010000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY2 :: 0x00020000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY3 :: 0x00040000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY4 :: 0x00080000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY5 :: 0x00100000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY6 :: 0x00200000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY7 :: 0x00400000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY8 :: 0x00800000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY9 :: 0x01000000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY10 :: 0x02000000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY11 :: 0x04000000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY12 :: 0x08000000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY13 :: 0x10000000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY14 :: 0x20000000`
- [OS == .WINDOWS] `WGL_SWAP_UNDERLAY15 :: 0x40000000`
- [OS == .WINDOWS] `HGLRC :: HANDLE`
- [OS == .WINDOWS] `ChoosePixelFormat :: (hdc: HDC, ppfd: *PIXELFORMATDESCRIPTOR) -> s32 #foreign gdi`
- [OS == .WINDOWS] `DescribePixelFormat :: (hdc: HDC, iPixelFormat: s32, nBytes: u32, ppfd: *PIXELFORMATDESCRIPTOR) -> s32 #foreign gdi`
- [OS == .WINDOWS] `GetPixelFormat :: (hdc: HDC) -> s32 #foreign gdi`
- [OS == .WINDOWS] `SetPixelFormat :: (hdc: HDC, iPixelFormat: s32, ppfd: *PIXELFORMATDESCRIPTOR) -> BOOL #foreign gdi`
- [OS == .WINDOWS] `SwapBuffers :: (hdc: HDC) -> BOOL #foreign gdi`
- [OS == .WINDOWS] `wglCopyContext :: (hglrcSrc: HGLRC, hglrcDst: HGLRC, mask: u32) -> BOOL #foreign gl`
- [OS == .WINDOWS] `wglCreateContext :: (hdc: HDC) -> HGLRC #foreign gl`
- [OS == .WINDOWS] `wglCreateLayerContext :: (hdc: HDC, level: s32) -> HGLRC #foreign gl`
- [OS == .WINDOWS] `wglDeleteContext :: (oldContext: HGLRC) -> BOOL #foreign gl`
- [OS == .WINDOWS] `wglGetCurrentContext :: () -> HGLRC #foreign gl`
- [OS == .WINDOWS] `wglGetCurrentDC :: () -> HDC #foreign gl`
- [OS == .WINDOWS] `wglGetProcAddress :: (lpszProc: *u8) -> *void #foreign gl`
- [OS == .WINDOWS] `wglMakeCurrent :: (hdc: HDC, newContext: HGLRC) -> BOOL #foreign gl`
- [OS == .WINDOWS] `wglRealizeLayerPalette :: (hdc: HDC, iLayerPlane: s32, bRealize: BOOL) -> BOOL #foreign gl`
- [OS == .WINDOWS] `wglShareLists :: (hrcSrvShare: HGLRC, hrcSrvSource: HGLRC) -> BOOL #foreign gl`
- [OS == .WINDOWS] `wglSwapLayerBuffers :: (hdc: HDC, fuFlags: u32) -> BOOL #foreign gl`
- [OS == .WINDOWS] `wglUseFontBitmapsA :: (hDC: HDC, first: u32, count: u32, listBase: u32) -> BOOL #foreign gl`
- [OS == .WINDOWS] `wglUseFontBitmapsW :: (hDC: HDC, first: u32, count: u32, listBase: u32) -> BOOL #foreign gl`
- [OS == .WINDOWS] `using PROCESS_DPI_AWARENESS :: enum s32`
- [OS == .WINDOWS] `using MONITOR_DPI_TYPE :: enum s32`
- [OS == .WINDOWS] `using DPI_AWARENESS :: enum s32`
- [OS == .WINDOWS] `DPI_AWARENESS_CONTEXT :: #type *struct`
- [OS == .WINDOWS] `DPI_AWARENESS_CONTEXT_UNAWARE :: cast(DPI_AWARENESS_CONTEXT)-1`
- [OS == .WINDOWS] `DPI_AWARENESS_CONTEXT_SYSTEM_AWARE :: cast(DPI_AWARENESS_CONTEXT)-2`
- [OS == .WINDOWS] `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE :: cast(DPI_AWARENESS_CONTEXT)-3`
- [OS == .WINDOWS] `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 :: cast(DPI_AWARENESS_CONTEXT)-4`
- [OS == .WINDOWS] `DPI_AWARENESS_CONTEXT_UNAWARE_GDISCALED :: cast(DPI_AWARENESS_CONTEXT)-5`
- [OS == .WINDOWS] `GetLogicalProcessorInformationEx :: ( RelationshipType: LOGICAL_PROCESSOR_RELATIONSHIP, Buffer: *SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX, ReturnedLength: *u32 ) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `OSVERSIONINFOA :: struct`
- [OS == .WINDOWS] `OSVERSIONINFOW :: struct`
- [OS == .WINDOWS] `OSVERSIONINFOEXA :: struct`
- [OS == .WINDOWS] `OSVERSIONINFOEXW :: struct`
- [OS == .WINDOWS] `SYSTEM_INFO :: struct`
- [OS == .WINDOWS] `MAX_COMPUTERNAME_LENGTH :: 31`
- [OS == .WINDOWS] `using LOGICAL_PROCESSOR_RELATIONSHIP :: enum s32`
- [OS == .WINDOWS] `SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX :: struct`
- [OS == .WINDOWS] `PROCESSOR_RELATIONSHIP :: struct`
- [OS == .WINDOWS] `NUMA_NODE_RELATIONSHIP :: struct`
- [OS == .WINDOWS] `CACHE_RELATIONSHIP :: struct`
- [OS == .WINDOWS] `using PROCESSOR_CACHE_TYPE :: enum`
- [OS == .WINDOWS] `GROUP_RELATIONSHIP :: struct`
- [OS == .WINDOWS] `PROCESSOR_GROUP_INFO :: struct`
- [OS == .WINDOWS] `GROUP_AFFINITY :: struct`
- [OS == .WINDOWS] `SYSTEM_CPU_SET_INFORMATION :: struct`
- [OS == .WINDOWS] `GetComputerNameA :: (lpBuffer: *u8, nSize: *u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetComputerNameW :: (lpBuffer: *u16, nSize: *u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetSystemDirectoryA :: (lpBuffer: *u8, uSize: u32) -> u32 #foreign kernel32`
- [OS == .WINDOWS] `GetSystemDirectoryW :: (lpBuffer: *u16, uSize: u32) -> u32 #foreign kernel32`
- [OS == .WINDOWS] `GetSystemInfo :: (lpSystemInfo : *SYSTEM_INFO) -> void #foreign kernel32`
- [OS == .WINDOWS] `GetNativeSystemInfo :: (lpSystemInfo : *SYSTEM_INFO) -> void #foreign kernel32`
- [OS == .WINDOWS] `GetUserNameA :: (lpBuffer: *u8, lpnSize: *u32) -> BOOL #foreign Advapi32`
- [OS == .WINDOWS] `GetUserNameW :: (lpBuffer: *u16, lpnSize: *u32) -> BOOL #foreign Advapi32`
- [OS == .WINDOWS] `GetVersion :: () -> u32 #foreign kernel32`
- [OS == .WINDOWS] `GetVersionExA :: (lpVersionInfo: *OSVERSIONINFOA) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetVersionExW :: (lpVersionInfo: *OSVERSIONINFOW) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `QueryPerformanceFrequency :: (result: *LARGE_INTEGER) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `QueryPerformanceCounter :: (result: *LARGE_INTEGER) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `VER_NT_WORKSTATION :: 0x0000001`
- [OS == .WINDOWS] `VER_NT_DOMAIN_CONTROLLER :: 0x0000002`
- [OS == .WINDOWS] `VER_NT_SERVER :: 0x0000003`
- [OS == .WINDOWS] `RtlGetVersion :: (lpVersionInfo : *OSVERSIONINFOW) -> void #foreign ntdll`
- [OS == .WINDOWS] `SECURITY_ATTRIBUTES :: struct`
- [OS == .WINDOWS] `LIST_ENTRY :: struct`
- [OS == .WINDOWS] `RTL_CRITICAL_SECTION_DEBUG :: struct`
- [OS == .WINDOWS] `RTL_CRITICAL_SECTION :: struct`
- [OS == .WINDOWS] `CRITICAL_SECTION :: RTL_CRITICAL_SECTION`
- [OS == .WINDOWS] `OVERLAPPED :: struct`
- [OS == .WINDOWS] `STATUS_PENDING : u32 : 0x00000103`
- [OS == .WINDOWS] `HasOverlappedIoCompleted :: inline (overlapped: OVERLAPPED) -> bool`
- [OS == .WINDOWS] `PIPE_WAIT : u32 : 0x00000000`
- [OS == .WINDOWS] `PIPE_NOWAIT : u32 : 0x00000001`
- [OS == .WINDOWS] `PIPE_ACCESS_INBOUND : u32 : 0x00000001`
- [OS == .WINDOWS] `PIPE_ACCESS_OUTBOUND : u32 : 0x00000002`
- [OS == .WINDOWS] `PIPE_ACCESS_DUPLEX : u32 : 0x00000003`
- [OS == .WINDOWS] `PIPE_TYPE_BYTE : u32 : 0x00000000`
- [OS == .WINDOWS] `PIPE_READMODE_BYTE : u32 : 0x00000000`
- [OS == .WINDOWS] `PIPE_ACCEPT_REMOTE_CLIENTS : u32 : 0x00000000`
- [OS == .WINDOWS] `PIPE_REJECT_REMOTE_CLIENTS : u32 : 0x00000008`
- [OS == .WINDOWS] `FILE_FLAG_FIRST_PIPE_INSTANCE : u32 : 0x00080000`
- [OS == .WINDOWS] `GENERIC_ALL : u32 : 0x10000000`
- [OS == .WINDOWS] `GENERIC_EXECUTE : u32 : 0x20000000`
- [OS == .WINDOWS] `GENERIC_WRITE : u32 : 0x40000000`
- [OS == .WINDOWS] `GENERIC_READ : u32 : 0x80000000`
- [OS == .WINDOWS] `PIPE_UNLIMITED_INSTANCES : u32 : 255`
- [OS == .WINDOWS] `CreatePipe :: (hReadPipe: *HANDLE, hWritePipe: *HANDLE, lpPipeAttributes: *SECURITY_ATTRIBUTES, nSize: DWORD) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetNamedPipeHandleState :: (hNamedPipe: HANDLE, lpMode: *DWORD, lpMaxConnectionCount: *DWORD, lpCollectDataTimeout: *DWORD) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `PeekNamedPipe :: (hNamedPipe: HANDLE, lpBuffer: *void, nBufferSize: DWORD, lpByesRead: *DWORD, lpTotalBytesAvail: *DWORD, lpBytesLeftThisMessage: *u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateNamedPipeW :: (lpName: *u16, dwOpenMode: u32, dwPipeMode: u32, nMaxInstances: u32, nOutBufferSize: u32, nInBufferSize: u32, nDefaultTimeOut: u32, lpSecurityAttributes: *SECURITY_ATTRIBUTES) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `ConnectNamedPipe :: (hNamedPipe: HANDLE, lpOverlapped: *OVERLAPPED) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateEventA :: (lpEventAttributes: *SECURITY_ATTRIBUTES, bManualReset: s32, bInitialState: s32, lpName: *u8) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateEventW :: (lpEventAttributes: *SECURITY_ATTRIBUTES, bManualReset: s32, bInitialState: s32, lpName: *u16) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateSemaphoreA :: (lpSemaphoreAttributes: *SECURITY_ATTRIBUTES, lInitialCount: s32, lMaximumCount: s32, lpName: *u8) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateSemaphoreW :: (lpSemaphoreAttributes: *SECURITY_ATTRIBUTES, lInitialCount: s32, lMaximumCount: s32, lpName: *u16) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateWaitableTimerW :: (lpTimerAttributes: *SECURITY_ATTRIBUTES, bManualReset: BOOL, lpTimerName: *u16) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `DeleteCriticalSection :: (lpCriticalSection: *CRITICAL_SECTION) #foreign kernel32`
- [OS == .WINDOWS] `EnterCriticalSection :: (lpCriticalSection: *CRITICAL_SECTION) #foreign kernel32`
- [OS == .WINDOWS] `GetOverlappedResult :: (hFile: HANDLE, lpOverlapped: *OVERLAPPED, lpNumberOfBytesTransferred: *s32, bWait: s32) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `InitializeCriticalSection :: (lpCriticalSection: *CRITICAL_SECTION) #foreign kernel32`
- [OS == .WINDOWS] `LeaveCriticalSection :: (lpCriticalSection: *CRITICAL_SECTION) #foreign kernel32`
- [OS == .WINDOWS] `PulseEvent :: (handle: HANDLE) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `ReleaseSemaphore :: (hSemaphore: HANDLE, lReleaseCount: s32, lpPreviousCount: *s32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetWaitableTimer :: (hTimer: HANDLE, lpDueTime: *LARGE_INTEGER, lPeriod: s32, pfnCompletionRoutine: *void , lpArgToCompeltionRoutine: *void, fResume: BOOL) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `WaitForMultipleObjects :: (nCount: DWORD, lpHandles: *HANDLE, bWaitAll: BOOL, dwMilliseconds: DWORD) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `WaitForSingleObject :: (handle: HANDLE, milliseconds: DWORD) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `CONDITION_VARIABLE :: struct`
- [OS == .WINDOWS] `InitializeConditionVariable :: (ConditionVariable: *CONDITION_VARIABLE) #foreign kernel32`
- [OS == .WINDOWS] `WakeConditionVariable :: (ConditionVariable: *CONDITION_VARIABLE) #foreign kernel32`
- [OS == .WINDOWS] `WakeAllConditionVariable :: (ConditionVariable: *CONDITION_VARIABLE) #foreign kernel32`
- [OS == .WINDOWS] `SleepConditionVariableCS :: (ConditionVariable: *CONDITION_VARIABLE, CriticalSection: *CRITICAL_SECTION, dwMilliseconds: DWORD) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SleepConditionVariableSRW :: (ConditionVariable: *CONDITION_VARIABLE, SRWLock: *void, dwMilliseconds: DWORD, Flags: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `HGLOBAL :: HANDLE`
- [OS == .WINDOWS] `HDROP :: HANDLE`
- [OS == .WINDOWS] `GMEM_FIXED :: 0x0000`
- [OS == .WINDOWS] `GMEM_MOVEABLE :: 0x0002`
- [OS == .WINDOWS] `GMEM_NOCOMPACT :: 0x0010`
- [OS == .WINDOWS] `GMEM_NODISCARD :: 0x0020`
- [OS == .WINDOWS] `GMEM_ZEROINIT :: 0x0040`
- [OS == .WINDOWS] `GMEM_MODIFY :: 0x0080`
- [OS == .WINDOWS] `GMEM_DISCARDABLE :: 0x0100`
- [OS == .WINDOWS] `GMEM_NOT_BANKED :: 0x1000`
- [OS == .WINDOWS] `GMEM_SHARE :: 0x2000`
- [OS == .WINDOWS] `GMEM_DDESHARE :: 0x2000`
- [OS == .WINDOWS] `GMEM_NOTIFY :: 0x4000`
- [OS == .WINDOWS] `GMEM_LOWER :: GMEM_NOT_BANKED`
- [OS == .WINDOWS] `GMEM_VALID_FLAGS :: 0x7F72`
- [OS == .WINDOWS] `GMEM_INVALID_HANDLE :: 0x8000`
- [OS == .WINDOWS] `GHND :: (GMEM_MOVEABLE | GMEM_ZEROINIT)`
- [OS == .WINDOWS] `GPTR :: (GMEM_FIXED | GMEM_ZEROINIT)`
- [OS == .WINDOWS] `MEMORYSTATUSEX :: struct`
- [OS == .WINDOWS] `PAGE_NOACCESS :: 0x01`
- [OS == .WINDOWS] `PAGE_READONLY :: 0x02`
- [OS == .WINDOWS] `PAGE_READWRITE :: 0x04`
- [OS == .WINDOWS] `PAGE_WRITECOPY :: 0x08`
- [OS == .WINDOWS] `PAGE_EXECUTE :: 0x10`
- [OS == .WINDOWS] `PAGE_EXECUTE_READ :: 0x20`
- [OS == .WINDOWS] `PAGE_EXECUTE_READWRITE :: 0x40`
- [OS == .WINDOWS] `PAGE_EXECUTE_WRITECOPY :: 0x80`
- [OS == .WINDOWS] `PAGE_GUARD :: 0x100`
- [OS == .WINDOWS] `PAGE_NOCACHE :: 0x200`
- [OS == .WINDOWS] `PAGE_WRITECOMBINE :: 0x400`
- [OS == .WINDOWS] `PAGE_REVERT_TO_FILE_MAP :: 0x80000000`
- [OS == .WINDOWS] `PAGE_ENCLAVE_THREAD_CONTROL :: 0x80000000`
- [OS == .WINDOWS] `PAGE_TARGETS_NO_UPDATE :: 0x40000000`
- [OS == .WINDOWS] `PAGE_TARGETS_INVALID :: 0x40000000`
- [OS == .WINDOWS] `PAGE_ENCLAVE_UNVALIDATED :: 0x20000000`
- [OS == .WINDOWS] `MEM_COMMIT :: 0x00001000`
- [OS == .WINDOWS] `MEM_RESERVE :: 0x00002000`
- [OS == .WINDOWS] `MEM_DECOMMIT :: 0x00004000`
- [OS == .WINDOWS] `MEM_RELEASE :: 0x00008000`
- [OS == .WINDOWS] `MEM_FREE :: 0x00010000`
- [OS == .WINDOWS] `MEM_PRIVATE :: 0x00020000`
- [OS == .WINDOWS] `MEM_MAPPED :: 0x00040000`
- [OS == .WINDOWS] `MEM_RESET :: 0x00080000`
- [OS == .WINDOWS] `MEM_TOP_DOWN :: 0x00100000`
- [OS == .WINDOWS] `MEM_WRITE_WATCH :: 0x00200000`
- [OS == .WINDOWS] `MEM_PHYSICAL :: 0x00400000`
- [OS == .WINDOWS] `MEM_ROTATE :: 0x00800000`
- [OS == .WINDOWS] `MEM_DIFFERENT_IMAGE_BASE_OK :: 0x00800000`
- [OS == .WINDOWS] `MEM_RESET_UNDO :: 0x01000000`
- [OS == .WINDOWS] `MEM_LARGE_PAGES :: 0x20000000`
- [OS == .WINDOWS] `MEM_4MB_PAGES :: 0x80000000`
- [OS == .WINDOWS] `MEM_64K_PAGES :: MEM_LARGE_PAGES | MEM_PHYSICAL`
- [OS == .WINDOWS] `PSECURE_MEMORY_CACHE_CALLBACK :: #type (Addr: *void, Range: SIZE_T) -> BOOLEAN #c_call`
- [OS == .WINDOWS] `AddSecureMemoryCacheCallback :: (pfnCallBack: PSECURE_MEMORY_CACHE_CALLBACK) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateFileMappingA :: (hFile: HANDLE, lpFileMappingAttributes: *SECURITY_ATTRIBUTES, flProtect: DWORD, dwMaximumSizeHigh: DWORD, dwMaximumSizeLow: DWORD, lpName: LPCSTR) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateFileMappingW :: (hFile: HANDLE, lpFileMappingAttributes: *SECURITY_ATTRIBUTES, flProtect: DWORD, dwMaximumSizeHigh: DWORD, dwMaximumSizeLow: DWORD, lpName: *u16) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `GlobalAlloc :: (uFlags: u32, dwBytes: SIZE_T) -> HGLOBAL #foreign kernel32`
- [OS == .WINDOWS] `GlobalLock :: (hMem: HGLOBAL) -> *void #foreign kernel32`
- [OS == .WINDOWS] `GlobalMemoryStatusEx :: (lpBuffer: *MEMORYSTATUSEX) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GlobalUnlock :: (hMem: HGLOBAL) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `LocalFree :: (hMem: HLOCAL) -> HLOCAL #foreign kernel32`
- [OS == .WINDOWS] `MapViewOfFile :: (hFileMappingObject: HANDLE, dwDesiredAccess: DWORD, dwFileOffsetHigh: DWORD, dwFileOffsetLow: DWORD, dwNumberOfBytesToMap: SIZE_T) -> *void #foreign kernel32`
- [OS == .WINDOWS] `OpenFileMappingA :: (dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: LPCSTR) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `OpenFileMappingW :: (dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: *u16) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `UnmapViewOfFile :: (address: *void) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `VirtualAlloc :: (lpAddress: *void, dwSize: u64, flAllocationType: u32, flProtect: u32) -> *void #foreign kernel32`
- [OS == .WINDOWS] `VirtualFree :: (lpAddress: *void, dwSize: u64, dwFreeType: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `VirtualLock :: (lpAddress: *void, dwSize: u64) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `VirtualProtect :: (lpAddress: *void, dwSize: u64, flNewProtect: u32, lpflOldProtect: *u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `VirtualQuery :: (lpAddress: *void, lpBuffer: *MEMORY_BASIC_INFORMATION, dwLength: SIZE_T) -> SIZE_T #foreign kernel32`
- [OS == .WINDOWS] `CloseHandle :: (handle: HANDLE) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `SetHandleInformation :: (hObject: HANDLE, dwMask: DWORD, dwFlags: DWORD) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FreeLibrary :: (module: HMODULE) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetModuleFileNameA :: (module: HANDLE, result: *u8, size: s32) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `GetModuleFileNameW :: (module: HANDLE, result: *u16, size: s32) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `GetModuleHandleA :: (module_name: *u8) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `GetModuleHandleW :: (module_name: *u16) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `GetProcAddress :: (module: HMODULE, proc_name: *u8) -> *void #foreign kernel32`
- [OS == .WINDOWS] `LoadLibraryA :: (filename: *u8) -> HMODULE #foreign kernel32`
- [OS == .WINDOWS] `LoadLibraryW :: (filename: *u16) -> HMODULE #foreign kernel32`
- [OS == .WINDOWS] `LoadLibraryExA :: (filename: *u8, file: HANDLE, flags: DWORD) -> HMODULE #foreign kernel32`
- [OS == .WINDOWS] `LoadLibraryExW :: (filename: *u16, file: HANDLE, flags: DWORD) -> HMODULE #foreign kernel32`
- [OS == .WINDOWS] `DONT_RESOLVE_DLL_REFERENCES :: 0x00000001`
- [OS == .WINDOWS] `LOAD_IGNORE_CODE_AUTHZ_LEVEL :: 0x00000010`
- [OS == .WINDOWS] `LOAD_LIBRARY_AS_DATAFILE :: 0x00000002`
- [OS == .WINDOWS] `LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE :: 0x00000040`
- [OS == .WINDOWS] `LOAD_LIBRARY_AS_IMAGE_RESOURCE :: 0x00000020`
- [OS == .WINDOWS] `LOAD_LIBRARY_SEARCH_APPLICATION_DIR :: 0x00000200`
- [OS == .WINDOWS] `LOAD_LIBRARY_SEARCH_DEFAULT_DIRS :: 0x00001000`
- [OS == .WINDOWS] `LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR :: 0x00000100`
- [OS == .WINDOWS] `LOAD_LIBRARY_SEARCH_SYSTEM32 :: 0x00000800`
- [OS == .WINDOWS] `LOAD_LIBRARY_SEARCH_USER_DIRS :: 0x00000400`
- [OS == .WINDOWS] `LOAD_WITH_ALTERED_SEARCH_PATH :: 0x00000008`
- [OS == .WINDOWS] `DLL_PROCESS_ATTACH :: 1`
- [OS == .WINDOWS] `DLL_THREAD_ATTACH :: 2`
- [OS == .WINDOWS] `DLL_THREAD_DETACH :: 3`
- [OS == .WINDOWS] `DLL_PROCESS_DETACH :: 0`
- [OS == .WINDOWS] `SetDllDirectoryA :: (lpPathName: *u8) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetDllDirectoryW :: (lpPathName: *u16) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FILE_NOTIFY_INFORMATION :: struct`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_FILE_NAME :: 0x00000001`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_DIR_NAME :: 0x00000002`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_ATTRIBUTES :: 0x00000004`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_SIZE :: 0x00000008`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_LAST_WRITE :: 0x00000010`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_LAST_ACCESS :: 0x00000020`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_CREATION :: 0x00000040`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_EA :: 0x00000080`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_SECURITY :: 0x00000100`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_STREAM_NAME :: 0x00000200`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_STREAM_SIZE :: 0x00000400`
- [OS == .WINDOWS] `FILE_NOTIFY_CHANGE_STREAM_WRITE :: 0x00000800`
- [OS == .WINDOWS] `FILE_ACTION_ADDED :: 1`
- [OS == .WINDOWS] `FILE_ACTION_REMOVED :: 2`
- [OS == .WINDOWS] `FILE_ACTION_MODIFIED :: 3`
- [OS == .WINDOWS] `FILE_ACTION_RENAMED_OLD_NAME :: 4`
- [OS == .WINDOWS] `FILE_ACTION_RENAMED_NEW_NAME :: 5`
- [OS == .WINDOWS] `FILE_ACTION_ADDED_STREAM :: 6`
- [OS == .WINDOWS] `FILE_ACTION_REMOVED_STREAM :: 7`
- [OS == .WINDOWS] `FILE_ACTION_MODIFIED_STREAM :: 8`
- [OS == .WINDOWS] `CreateDirectoryA :: (lpPathName: *u8, lpSecurityAttributes: *SECURITY_ATTRIBUTES) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `CreateDirectoryW :: (lpPathName: *u16, lpSecurityAttributes: *SECURITY_ATTRIBUTES) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `GetCurrentDirectoryA :: (nBufferLength: DWORD, lpBuffer: *u8) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `GetCurrentDirectoryW :: (nBufferLength: DWORD, lpBuffer: *u16) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `ReadDirectoryChangesW :: (hDirectory: HANDLE, lpBuffer: *void, nBufferLength: s32, bWatchSubtree: s32, dwNotifyFilter: s32, lpBytesReturned: *s32, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: *void) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `RemoveDirectoryA :: (lpPathName: *u8) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `RemoveDirectoryW :: (lpPathName: *u16) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetCurrentDirectoryW :: (name: *u16) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `ATTACH_PARENT_PROCESS :: cast,no_check(u32, -1)`
- [OS == .WINDOWS] `AttachConsole :: (dwProcessId: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `AllocConsole :: () -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FreeConsole :: () -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CONSOLE_SCREEN_BUFFER_INFO :: struct`
- [OS == .WINDOWS] `GetStdHandle :: (handle: s32 ) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `GetConsoleScreenBufferInfo :: (hConsoleOutput: HANDLE, lpConsoleScreenBufferInfo : *CONSOLE_SCREEN_BUFFER_INFO) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetConsoleTextAttribute :: (hConsoleOutput: HANDLE, wAttributes : u16) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetConsoleMode :: (hConsoleHandle: HANDLE, lpMode: *DWORD) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetConsoleMode :: (hConsoleHandle: HANDLE, lpMode: DWORD) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetConsoleCP :: (wCodePageID: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetConsoleOutputCP :: (wCodePageID: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `ENABLE_VIRTUAL_TERMINAL_PROCESSING :: 0x4`
- [OS == .WINDOWS] `LCID :: s32`
- [OS == .WINDOWS] `GetUserDefaultLCID :: () -> LCID #foreign kernel32`
- [OS == .WINDOWS] `CF_TEXT :: 1`
- [OS == .WINDOWS] `CF_BITMAP :: 2`
- [OS == .WINDOWS] `CF_METAFILEPICT :: 3`
- [OS == .WINDOWS] `CF_SYLK :: 4`
- [OS == .WINDOWS] `CF_DIF :: 5`
- [OS == .WINDOWS] `CF_TIFF :: 6`
- [OS == .WINDOWS] `CF_OEMTEXT :: 7`
- [OS == .WINDOWS] `CF_DIB :: 8`
- [OS == .WINDOWS] `CF_PALETTE :: 9`
- [OS == .WINDOWS] `CF_PENDATA :: 10`
- [OS == .WINDOWS] `CF_RIFF :: 11`
- [OS == .WINDOWS] `CF_WAVE :: 12`
- [OS == .WINDOWS] `CF_UNICODETEXT :: 13`
- [OS == .WINDOWS] `CF_ENHMETAFILE :: 14`
- [OS == .WINDOWS] `CF_HDROP :: 15`
- [OS == .WINDOWS] `CF_LOCALE :: 16`
- [OS == .WINDOWS] `CF_DIBV5 :: 17`
- [OS == .WINDOWS] `CF_MAX :: 18`
- [OS == .WINDOWS] `CF_OWNERDISPLAY :: 0x0080`
- [OS == .WINDOWS] `CF_DSPTEXT :: 0x0081`
- [OS == .WINDOWS] `CF_DSPBITMAP :: 0x0082`
- [OS == .WINDOWS] `CF_DSPMETAFILEPICT :: 0x0083`
- [OS == .WINDOWS] `CF_DSPENHMETAFILE :: 0x008E`
- [OS == .WINDOWS] `CloseClipboard :: () -> BOOL #foreign user32`
- [OS == .WINDOWS] `EmptyClipboard :: () -> BOOL #foreign user32`
- [OS == .WINDOWS] `GetClipboardData :: (uFormat: u32) -> HANDLE #foreign user32`
- [OS == .WINDOWS] `OpenClipboard :: (hwnd: HWND) -> BOOL #foreign user32`
- [OS == .WINDOWS] `SetClipboardData :: (uFormat: u32, hMem: HANDLE) -> HANDLE #foreign user32`
- [OS == .WINDOWS] `STARTUPINFOA :: struct`
- [OS == .WINDOWS] `STARTUPINFOW :: struct`
- [OS == .WINDOWS] `STARTF_USESTDHANDLES :: 0x00000100`
- [OS == .WINDOWS] `PROCESS_INFORMATION :: struct`
- [OS == .WINDOWS] `using Creation_Flags :: enum_flags u32`
- [OS == .WINDOWS] `WAIT_FAILED :: 0xFFFFFFFF`
- [OS == .WINDOWS] `WIN_TIMEOUT_INFINITE :: 0xFFFFFFFF`
- [OS == .WINDOWS] `THREAD_PRIORITY_NORMAL :: 0`
- [OS == .WINDOWS] `THREAD_PRIORITY_ABOVE_NORMAL :: 1`
- [OS == .WINDOWS] `THREAD_PRIORITY_HIGHEST :: 2`
- [OS == .WINDOWS] `JOBOBJECTINFOCLASS :: enum`
- [OS == .WINDOWS] `using Job_Object_Limit :: enum u32`
- [OS == .WINDOWS] `JOBOBJECT_BASIC_LIMIT_INFORMATION :: struct`
- [OS == .WINDOWS] `IO_COUNTERS :: struct`
- [OS == .WINDOWS] `JOBOBJECT_EXTENDED_LIMIT_INFORMATION :: struct`
- [OS == .WINDOWS] `AssignProcessToJobObject :: (hJob: HANDLE, hProcess: HANDLE) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateJobObjectA :: (lpSecurityAttributes: *SECURITY_ATTRIBUTES, name: *u8) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateProcessA :: (lpApplicationName: *u8, lpCommandLine: *u8, lpSecurityAttributes: *SECURITY_ATTRIBUTES, lpThreadAttributes: *SECURITY_ATTRIBUTES, bInheritHandles: BOOL, dwCreationFlags: Creation_Flags, lpEnvironment: *void, lpCurrentDirectory: *u8, lpStartupInfo: *STARTUPINFOA, lpProcessInformation: *PROCESS_INFORMATION) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateProcessW :: (lpApplicationName: *u16, lpCommandLine: *u16, lpSecurityAttributes: *SECURITY_ATTRIBUTES, lpThreadAttributes: *SECURITY_ATTRIBUTES, bInheritHandles: BOOL, dwCreationFlags: Creation_Flags, lpEnvironment: *void, lpCurrentDirectory: *u16, lpStartupInfo: *STARTUPINFOW, lpProcessInformation: *PROCESS_INFORMATION) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetExitCodeProcess :: (hProcess: HANDLE, lpExitCode: *u32) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `CreateThread :: (threadAttributes: *void, stackSize: s64, startAddress: *void, parameter: *void, creation_flags: Creation_Flags, threadIdReturn: *s32) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `ExitProcess :: (exit_code: u32) -> void #foreign kernel32`
- [OS == .WINDOWS] `GetCurrentProcess :: () -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `GetCurrentProcessId :: () -> u32 #foreign kernel32`
- [OS == .WINDOWS] `GetCurrentThread :: () -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `GetCurrentThreadId :: () -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `GetProcessId :: (Process: HANDLE) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `GetProcessWorkingSetSize :: (hProcess: HANDLE, lpMinimumWorkingSetSize: *SIZE_T, lpMaximumWorkingSetSize: *SIZE_T) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetThreadId :: (Thread: HANDLE) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `OpenThread :: (dwDesiredAccess: u32, bInheritHandle: BOOL, dwThreadId: u32) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `ResumeThread :: (thread: HANDLE) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `SetEnvironmentVariableA :: (name: *u8, value: *u8) #foreign kernel32`
- [OS == .WINDOWS] `SetInformationJobObject :: (hJob: HANDLE, JobOjectInformationClass: JOBOBJECTINFOCLASS, lpJobObjectInformation: *void, cbJobObjectInformationLength: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetThreadDescription :: (hThread: HANDLE, lpThreadDescription: *u16) -> HRESULT #foreign kernel32`
- [OS == .WINDOWS] `SetThreadPriority :: (thread: HANDLE, priority: s32) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `Sleep :: (milliseconds: s32) -> void #foreign kernel32`
- [OS == .WINDOWS] `SuspendThread :: (hThread: HANDLE) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `TerminateProcess :: (hProcess: HANDLE, uExitCode: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `DebugActiveProcess :: (dwProcessId: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `DebugBreak :: () #foreign kernel32`
- [OS == .WINDOWS] `GetThreadContext :: (hThread: HANDLE, lpContext: *CONTEXT) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `IsDebuggerPresent :: () -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `OutputDebugStringA :: (s : *u8) -> void #foreign kernel32`
- [OS == .WINDOWS] `OutputDebugStringW :: (s : *u16) ->void #foreign kernel32`
- [OS == .WINDOWS] `MMSYSERR_BASE :: 0`
- [OS == .WINDOWS] `WAVERR_BASE :: 32`
- [OS == .WINDOWS] `MIDIERR_BASE :: 64`
- [OS == .WINDOWS] `TIMERR_BASE :: 96`
- [OS == .WINDOWS] `JOYERR_BASE :: 160`
- [OS == .WINDOWS] `MCIERR_BASE :: 256`
- [OS == .WINDOWS] `MIXERR_BASE :: 1024`
- [OS == .WINDOWS] `TIMERR_NOERROR :: 0`
- [OS == .WINDOWS] `TIMERR_NOCANDO :: TIMERR_BASE + 1`
- [OS == .WINDOWS] `TIMERR_STRUCT :: TIMERR_BASE + 33`
- [OS == .WINDOWS] `MMSYSERR_NOERROR :: 0`
- [OS == .WINDOWS] `MMSYSERR_ERROR :: MMSYSERR_BASE + 1`
- [OS == .WINDOWS] `TIMECAPS :: struct`
- [OS == .WINDOWS] `MMTIME :: struct`
- [OS == .WINDOWS] `timeGetDevCaps :: (ptc: *TIMECAPS, cbtc: u32) -> u32 #foreign winmm`
- [OS == .WINDOWS] `timeBeginPeriod :: (period: u32) -> u32 #foreign winmm`
- [OS == .WINDOWS] `timeEndPeriod :: (period: u32) -> u32 #foreign winmm`
- [OS == .WINDOWS] `timeGetTime :: () -> u32 #foreign winmm`
- [OS == .WINDOWS] `timeGetSystemTime :: (pmmt: *MMTIME, cbmmt: u32) -> u32 #foreign winmm`
- [OS == .WINDOWS] `MB_OK :: 0x00000000`
- [OS == .WINDOWS] `MB_OKCANCEL :: 0x00000001`
- [OS == .WINDOWS] `MB_ABORTRETRYIGNORE :: 0x00000002`
- [OS == .WINDOWS] `MB_YESNOCANCEL :: 0x00000003`
- [OS == .WINDOWS] `MB_YESNO :: 0x00000004`
- [OS == .WINDOWS] `MB_RETRYCANCEL :: 0x00000005`
- [OS == .WINDOWS] `MB_CANCELTRYCONTINUE :: 0x00000006`
- [OS == .WINDOWS] `MB_ICONHAND :: 0x00000010`
- [OS == .WINDOWS] `MB_ICONQUESTION :: 0x00000020`
- [OS == .WINDOWS] `MB_ICONEXCLAMATION :: 0x00000030`
- [OS == .WINDOWS] `MB_ICONASTERISK :: 0x00000040`
- [OS == .WINDOWS] `MB_USERICON :: 0x00000080`
- [OS == .WINDOWS] `MB_ICONWARNING :: MB_ICONEXCLAMATION`
- [OS == .WINDOWS] `MB_ICONERROR :: MB_ICONHAND`
- [OS == .WINDOWS] `MB_ICONINFORMATION :: MB_ICONASTERISK`
- [OS == .WINDOWS] `MB_ICONSTOP :: MB_ICONHAND`
- [OS == .WINDOWS] `MB_DEFBUTTON1 :: 0x00000000`
- [OS == .WINDOWS] `MB_DEFBUTTON2 :: 0x00000100`
- [OS == .WINDOWS] `MB_DEFBUTTON3 :: 0x00000200`
- [OS == .WINDOWS] `MB_DEFBUTTON4 :: 0x00000300`
- [OS == .WINDOWS] `MB_APPLMODAL :: 0x00000000`
- [OS == .WINDOWS] `MB_SYSTEMMODAL :: 0x00001000`
- [OS == .WINDOWS] `MB_TASKMODAL :: 0x00002000`
- [OS == .WINDOWS] `MB_HELP :: 0x00004000`
- [OS == .WINDOWS] `MB_NOFOCUS :: 0x00008000`
- [OS == .WINDOWS] `MB_SETFOREGROUND :: 0x00010000`
- [OS == .WINDOWS] `MB_DEFAULT_DESKTOP_ONLY :: 0x00020000`
- [OS == .WINDOWS] `MB_TOPMOST :: 0x00040000`
- [OS == .WINDOWS] `MB_RIGHT :: 0x00080000`
- [OS == .WINDOWS] `MB_RTLREADING :: 0x00100000`
- [OS == .WINDOWS] `MB_SERVICE_NOTIFICATION :: 0x00200000`
- [OS == .WINDOWS] `MB_TYPEMASK :: 0x0000000F`
- [OS == .WINDOWS] `MB_ICONMASK :: 0x000000F0`
- [OS == .WINDOWS] `MB_DEFMASK :: 0x00000F00`
- [OS == .WINDOWS] `MB_MODEMASK :: 0x00003000`
- [OS == .WINDOWS] `MB_MISCMASK :: 0x0000C000`
- [OS == .WINDOWS] `IDOK :: 1`
- [OS == .WINDOWS] `IDCANCEL :: 2`
- [OS == .WINDOWS] `IDABORT :: 3`
- [OS == .WINDOWS] `IDRETRY :: 4`
- [OS == .WINDOWS] `IDIGNORE :: 5`
- [OS == .WINDOWS] `IDYES :: 6`
- [OS == .WINDOWS] `IDNO :: 7`
- [OS == .WINDOWS] `IDCLOSE :: 8`
- [OS == .WINDOWS] `IDHELP :: 9`
- [OS == .WINDOWS] `IDTRYAGAIN :: 10`
- [OS == .WINDOWS] `IDCONTINUE :: 11`
- [OS == .WINDOWS] `MessageBoxA :: (hWnd: HWND, lpText: *u8, lpCaption: *u8, uType:u32) -> s32 #foreign user32`
- [OS == .WINDOWS] `MessageBoxW :: (hWnd: HWND, lpText: *u16, lpCaption: *u16, uType:u32) -> s32 #foreign user32`
- [OS == .WINDOWS] `INVALID_FILE_ATTRIBUTES : u32 : 0xffff_ffff`
- [OS == .WINDOWS] `WIN32_FIND_DATAW :: struct`
- [OS == .WINDOWS] `FILE_GENERIC_ALL :: 0x10000000`
- [OS == .WINDOWS] `FILE_GENERIC_READ :: 0x80000000`
- [OS == .WINDOWS] `FILE_GENERIC_WRITE :: 0x40000000`
- [OS == .WINDOWS] `FILE_GENERIC_EXECUTE :: 0x20000000`
- [OS == .WINDOWS] `FILE_LIST_DIRECTORY :: 0x1`
- [OS == .WINDOWS] `FILE_EXECUTE :: 0x20`
- [OS == .WINDOWS] `FILE_READ_ATTRIBUTES :: 0x80`
- [OS == .WINDOWS] `FILE_READ_DATA :: 0x1`
- [OS == .WINDOWS] `FILE_TRAVERSE :: 0x20`
- [OS == .WINDOWS] `using SetFilePointer_MoveMethod :: enum u32`
- [OS == .WINDOWS] `using FINDEX_INFO_LEVELS :: enum`
- [OS == .WINDOWS] `using FINDEX_SEARCH_OPS :: enum`
- [OS == .WINDOWS] `FIND_FIRST_EX_CASE_SENSITIVE :: 0x1`
- [OS == .WINDOWS] `FIND_FIRST_EX_LARGE_FETCH :: 0x2`
- [OS == .WINDOWS] `FIND_FIRST_EX_ON_DISK_ENTRIES_ONLY :: 0x4`
- [OS == .WINDOWS] `CancelIo :: (hFile: HANDLE) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CopyFileW :: (lpFileName: *u16, lpNewFileName: *u16, bFailIfExists: BOOL) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CopyFileA :: (lpFileName: *u8, lpNewFileName: *u8, bFailIfExists: BOOL) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateFileA :: (filename: *u8, desired_access: u32, share_mode: u32 = FILE_SHARE_READ, security: *void = null, creation: u32 = OPEN_EXISTING, flags_and_attributes: u32 = 0, hTemplateFile: HANDLE = null) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateFileW :: (filename: *u16, desired_access: u32, share_mode: u32 = FILE_SHARE_READ, security: *void = null, creation: u32 = OPEN_EXISTING, flags_and_attributes: u32 = 0, hTemplateFile: HANDLE = null) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `CreateHardLinkA :: (lpFileName: *u8, lpExistingFileName: *u8, lpSecurityAttributes: *SECURITY_ATTRIBUTES) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateHardLinkW :: (lpFileName: *u16, lpExistingFileName: *u16, lpSecurityAttributes: *SECURITY_ATTRIBUTES) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `CreateIoCompletionPort :: (FileHandle: HANDLE, ExistingCompletionPort: HANDLE, CompletionKey: u64, NumberOfConcurrentThreads: u32) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `DeleteFileA :: (filename: *u8) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `DeleteFileW :: (filename: *u16) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FindClose :: (file: HANDLE) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FindFirstFileA :: (lpFileName: *u8, lpFindFileData: *void) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `FindFirstFileW :: (lpFileName: *u16, lpFindFileData: *WIN32_FIND_DATAW) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `FindFirstFileExA :: (lpFileName: *u8, fInfoLevelId: FINDEX_INFO_LEVELS, lpFindFileData: *void, fSearchOp: FINDEX_SEARCH_OPS, lpSearchFilter: *void, dwAdditionalFlags: DWORD) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `FindFirstFileExW :: (lpFileName: *u16, fInfoLevelId: FINDEX_INFO_LEVELS, lpFindFileData: *void, fSearchOp: FINDEX_SEARCH_OPS, lpSearchFilter: *void, dwAdditionalFlags: DWORD) -> HANDLE #foreign kernel32`
- [OS == .WINDOWS] `FindNextFileA :: (hFile: HANDLE, lpFindFileData: *void) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FindNextFileW :: (hFile: HANDLE, lpFindFileData: *WIN32_FIND_DATAW) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FlushFileBuffers :: (hFile: HANDLE) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetFileAttributesA :: (lpFileName: *u8) -> u32 #foreign kernel32`
- [OS == .WINDOWS] `GetFileAttributesW :: (lpFileName: *u16) -> u32 #foreign kernel32`
- [OS == .WINDOWS] `GetFileInformationByHandleEx :: (hFile: HANDLE, FileInformationClass: FILE_INFO_BY_HANDLE_CLASS, lpFileInformation: *void, dwBufferSize: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetFileSize :: (file: HANDLE, FileSizeHigh: *u32) -> u32 #foreign kernel32`
- [OS == .WINDOWS] `GetFileSizeEx :: (file: HANDLE, size: *LARGE_INTEGER) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `GetFinalPathNameByHandleW :: (hFile: HANDLE, lpszFilePath: *u16, cchFilePath: DWORD, dwFlags: DWORD) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `GetFullPathNameA :: (filename: *u8, buffer_length: DWORD, buffer: *u8, file_part: **u8) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `GetFullPathNameW :: (filename: *u16, buffer_length: DWORD, buffer: *u16, file_part: **u16) -> DWORD #foreign kernel32`
- [OS == .WINDOWS] `GetQueuedCompletionStatus :: (CompletionPort: HANDLE, NumberOfBytesTransferred: *u32, CompletionKey: *u64, Overlapped: **OVERLAPPED, Milliseconds: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `LockFile :: (file: HANDLE, dwFileOffsetLow: DWORD, dwFileOffsetHigh: DWORD, nNumberOfBytesToLockLow: DWORD, nNumberOfBytesToLockHigh: DWORD) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `LockFileEx :: (file: HANDLE, dwFlags: DWORD, dwReserved: DWORD, nNumberOfBytesToLockLow: DWORD, nNumberOfBytesToLockHigh: DWORD, lpOverlapped: *OVERLAPPED) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `MoveFileA :: (filename: *u8, new_filename: *u8) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `MoveFileW :: (filename: *u16, new_filename: *u16) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `ReadFile :: (file: HANDLE, buffer: *void, to_read: DWORD, did_read: *DWORD, overlapped: *void = null) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `ReplaceFileW :: (lpReplacedFileName: LPCWSTR, lpReplacementFileName: LPCWSTR, lpBackupFileName: LPCWSTR, dwReplaceFlags: DWORD, lpExclude: *void, lpReserved: *void) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `using FILE_INFO_BY_HANDLE_CLASS :: enum`
- [OS == .WINDOWS] `FILE_STANDARD_INFO :: struct`
- [OS == .WINDOWS] `FILE_END_OF_FILE_INFO :: struct`
- [OS == .WINDOWS] `FILE_FULL_DIR_INFO :: struct`
- [OS == .WINDOWS] `SetFileInformationByHandle :: (File: HANDLE, FileInformationClass: FILE_INFO_BY_HANDLE_CLASS, FileInformation: *void, BufferSize: u32) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SetFilePointer :: (hFile: HANDLE, liDistanceToMove: s32, lpNewFilePointer: *s32, dwMoveMethod: SetFilePointer_MoveMethod) -> u32 #foreign kernel32`
- [OS == .WINDOWS] `SetFilePointerEx :: (hFile: HANDLE, liDistanceToMove: LARGE_INTEGER, lpNewFilePointer: *LARGE_INTEGER, dwMoveMethod: SetFilePointer_MoveMethod) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `WriteFile :: (handle: HANDLE, buffer: *void, buffer_length: DWORD, written_result: *DWORD, overlapped: *void) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `DwmIsCompositionEnabled :: (pfEnabled: *BOOL) -> HRESULT #foreign Dwmapi`
- [OS == .WINDOWS] `IMAGE_FILE_MACHINE_I386 :: 0x014c`
- [OS == .WINDOWS] `IMAGE_FILE_MACHINE_IA64 :: 0x0200`
- [OS == .WINDOWS] `IMAGE_FILE_MACHINE_AMD64 :: 0x8664`
- [OS == .WINDOWS] `ADDRESS_MODE :: enum s32`
- [OS == .WINDOWS] `ADDRESS64 :: struct`
- [OS == .WINDOWS] `KDHELP64 :: struct`
- [OS == .WINDOWS] `STACKFRAME64 :: struct`
- [OS == .WINDOWS] `M128A :: struct #align 16`
- [OS == .WINDOWS] `XSAVE_FORMAT :: struct`
- [OS == .WINDOWS] `CONTEXT_i386 :: 0x00010000`
- [OS == .WINDOWS] `CONTEXT_AMD64 :: 0x00100000`
- [OS == .WINDOWS] `CONTEXT_ARM64 :: 0x00400000`
- [OS == .WINDOWS] `CONTEXT_CONTROL :: #ifx CPU == .ARM64 then (CONTEXT_ARM64 | 0x00000001) else (CONTEXT_AMD64 | 0x00000001)`
- [OS == .WINDOWS] `CONTEXT_INTEGER :: #ifx CPU == .ARM64 then (CONTEXT_ARM64 | 0x00000002) else (CONTEXT_AMD64 | 0x00000002)`
- [OS == .WINDOWS] `CONTEXT_SEGMENTS :: (CONTEXT_AMD64 | 0x00000004)`
- [OS == .WINDOWS] `CONTEXT_FLOATING_POINT :: #ifx CPU == .ARM64 then (CONTEXT_ARM64 | 0x00000004) else (CONTEXT_AMD64 | 0x00000008)`
- [OS == .WINDOWS] `CONTEXT_DEBUG_REGISTERS :: #ifx CPU == .ARM64 then (CONTEXT_ARM64 | 0x00000008) else (CONTEXT_AMD64 | 0x00000010)`
- [OS == .WINDOWS] `CONTEXT_FULL :: (CONTEXT_CONTROL | CONTEXT_INTEGER | CONTEXT_FLOATING_POINT)`
- [OS == .WINDOWS] `CONTEXT :: struct #type_info_none #align 16`
- [OS == .WINDOWS] `IMAGEHLP_SYMBOL64 :: struct`
- [OS == .WINDOWS] `IMAGEHLP_LINE64 :: struct`
- [OS == .WINDOWS] `IMAGEHLP_LINEW64 :: struct`
- [OS == .WINDOWS] `SYMBOL_INFO :: struct`
- [OS == .WINDOWS] `SYMOPT :: enum_flags u32`
- [OS == .WINDOWS] `STATUS_ACCESS_VIOLATION :: 0x80000005`
- [OS == .WINDOWS] `EXCEPTION_WRITE_FAULT :: 1`
- [OS == .WINDOWS] `MINIDUMP_TYPE :: enum_flags u32`
- [OS == .WINDOWS] `MINIDUMP_CALLBACK_TYPE :: enum u32`
- [OS == .WINDOWS] `MINIDUMP_EXCEPTION_INFORMATION :: struct`
- [OS == .WINDOWS] `MINIDUMP_EXCEPTION_INFORMATION64 :: struct`
- [OS == .WINDOWS] `MINIDUMP_USER_STREAM :: struct`
- [OS == .WINDOWS] `MINIDUMP_USER_STREAM_INFORMATION :: struct`
- [OS == .WINDOWS] `MINIDUMP_CALLBACK_INPUT :: struct`
- [OS == .WINDOWS] `MINIDUMP_MEMORY_INFO :: struct`
- [OS == .WINDOWS] `MINIDUMP_CALLBACK_OUTPUT :: struct`
- [OS == .WINDOWS] `MINIDUMP_CALLBACK_ROUTINE :: #type (CallbackParam: *void, CallbackInput: *MINIDUMP_CALLBACK_INPUT, CallbackOutput: *MINIDUMP_CALLBACK_OUTPUT) -> BOOL #c_call`
- [OS == .WINDOWS] `MINIDUMP_CALLBACK_INFORMATION :: struct`
- [OS == .WINDOWS] `ReadProcessMemoryProc64 :: #type (hProcess: HANDLE, lpBaseAddress: u64, lpBuffer: *void, nSize: u32, lpNumberOfBytesRead: *u32) -> BOOL #c_call`
- [OS == .WINDOWS] `FunctionTableAccessProc64 :: #type (hProcess: HANDLE, AddrBase:u64) -> *void #c_call`
- [OS == .WINDOWS] `GetModuleBaseProc64 :: #type (hProcess: HANDLE, dwAddr:u64) -> u64 #c_call`
- [OS == .WINDOWS] `TranslateAddressProc64 :: #type (hProcess: HANDLE, hThread: HANDLE, lpaddr: *ADDRESS64) -> u64 #c_call`
- [OS == .WINDOWS] `MiniDumpWriteDump :: (hProcess: HANDLE, ProcessId: DWORD, hFile: HANDLE, DumpType: MINIDUMP_TYPE, ExceptionParam: *MINIDUMP_EXCEPTION_INFORMATION, UserStreamParam: *MINIDUMP_USER_STREAM_INFORMATION, CallbackParam: *MINIDUMP_CALLBACK_INFORMATION) -> BOOL #foreign DbgHelp`
- [OS == .WINDOWS] `StackWalk64 :: (MachineType: u32, hProcess: HANDLE, hThread: HANDLE, StackFrame:*STACKFRAME64, ContextRecord: *void, ReadMemoryRoutine: ReadProcessMemoryProc64, FunctionTableAccessRoutine: FunctionTableAccessProc64, GetModuleBaseRoutine: GetModuleBaseProc64, TranslateAddres: TranslateAddressProc64) -> BOOL #foreign DbgHelp`
- [OS == .WINDOWS] `SymCleanup :: (hProcess: HANDLE) -> BOOL #foreign DbgHelp`
- [OS == .WINDOWS] `SymFromAddr :: (hProcess: HANDLE, Address: u64, Displacement: *u64, Symbol: *SYMBOL_INFO) -> BOOL #foreign DbgHelp`
- [OS == .WINDOWS] `SymFunctionTableAccess64 :: (hProcess: HANDLE, AddrBase:u64) -> *void #foreign DbgHelp`
- [OS == .WINDOWS] `SymGetLineFromAddr64 :: (hProcess: HANDLE, dwAddr: u64, pdwDisplacement: *u32, Line: *IMAGEHLP_LINE64) -> BOOL #foreign DbgHelp`
- [OS == .WINDOWS] `SymGetModuleBase64 :: (hProcess: HANDLE, dwAddr:u64) -> u64 #foreign DbgHelp`
- [OS == .WINDOWS] `SymGetSymFromAddr64 :: (hProcess: HANDLE, Address: u64, Displacement: *u64, Symbol: *IMAGEHLP_SYMBOL64) -> BOOL #foreign DbgHelp`
- [OS == .WINDOWS] `SymInitialize :: (hProcess: HANDLE, UserSearchPath: *u8, fInvadeProcess: BOOL) -> BOOL #foreign DbgHelp`
- [OS == .WINDOWS] `SymSetOptions :: (SymOptions: SYMOPT) -> SYMOPT #foreign DbgHelp`
- [OS == .WINDOWS] `EXCEPTION_MAXIMUM_PARAMETERS :: 15`
- [OS == .WINDOWS] `EXCEPTION_RECORD :: struct`
- [OS == .WINDOWS] `EXCEPTION_POINTERS :: struct`
- [OS == .WINDOWS] `LPTOP_LEVEL_EXCEPTION_FILTER :: #type (ExceptionInfo: *EXCEPTION_POINTERS) -> s32 #c_call`
- [OS == .WINDOWS] `EXCEPTION_EXECUTE_HANDLER :: 1`
- [OS == .WINDOWS] `EXCEPTION_CONTINUE_SEARCH :: 0`
- [OS == .WINDOWS] `EXCEPTION_CONTINUE_EXECUTION :: -1`
- [OS == .WINDOWS] `RtlCaptureContext :: (ContextRecord: *CONTEXT) #foreign kernel32`
- [OS == .WINDOWS] `SetUnhandledExceptionFilter :: (lpTopLevelExceptionFilter: LPTOP_LEVEL_EXCEPTION_FILTER) -> LPTOP_LEVEL_EXCEPTION_FILTER #foreign kernel32`
- [OS == .WINDOWS] `using SEM :: enum_flags u32`
- [OS == .WINDOWS] `FORMAT_MESSAGE_ALLOCATE_BUFFER :: 0x00000100`
- [OS == .WINDOWS] `FORMAT_MESSAGE_IGNORE_INSERTS :: 0x00000200`
- [OS == .WINDOWS] `FORMAT_MESSAGE_FROM_SYSTEM :: 0x00001000`
- [OS == .WINDOWS] `using FLASHW :: enum_flags u32`
- [OS == .WINDOWS] `FLASHWINFO :: struct`
- [OS == .WINDOWS] `OS_Error_Code :: #type,isa u32`
- [OS == .WINDOWS] `FlashWindow :: (hWnd: HWND, bInvert: BOOL) -> BOOL #foreign user32`
- [OS == .WINDOWS] `FlashWindowEx :: (pfwi: *FLASHWINFO) -> BOOL #foreign user32`
- [OS == .WINDOWS] `FormatMessageA :: (dwFlags: u32, lpSource: *void, dwMessageId: u32, dwLanguageId: u32, lpBuffer: **u8, nSize: u32, Arguments: *void) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `FormatMessageW :: (dwFlags: u32, lpSource: *void, dwMessageId: u32, dwLanguageId: u32, lpBuffer: **u16, nSize: u32, Arguments: *void) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `GetErrorMode :: () -> SEM #foreign kernel32`
- [OS == .WINDOWS] `GetLastError :: () -> OS_Error_Code #foreign kernel32`
- [OS == .WINDOWS] `NTSTATUS :: DWORD`
- [OS == .WINDOWS] `RtlGetLastNtStatus :: () -> NTSTATUS #foreign ntdll`
- [OS == .WINDOWS] `SetErrorMode :: (uMode : SEM) -> SEM #foreign kernel32`
- [OS == .WINDOWS] `SYSTEMTIME :: struct`
- [OS == .WINDOWS] `FILETIME :: struct`
- [OS == .WINDOWS] `TIME_ZONE_INFORMATION :: struct`
- [OS == .WINDOWS] `FileTimeToLocalFileTime :: (lpFileTime: *FILETIME, lpLocalFileTime: *FILETIME) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `LocalFileTimeToFileTime :: (lpLocalFileTime: *FILETIME, lpFileTime: *FILETIME) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `FileTimeToSystemTime :: (lpFileTime: *FILETIME, lpSystemTime: *SYSTEMTIME) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `GetFileTime :: (file: HANDLE, lpCreationTime: *FILETIME, lpLastAccessTime: *FILETIME, lpLastWriteTime: *FILETIME) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `GetLocalTime :: (time: *SYSTEMTIME) -> void #foreign kernel32`
- [OS == .WINDOWS] `GetSystemTime :: (time : *SYSTEMTIME) -> void #foreign kernel32`
- [OS == .WINDOWS] `GetSystemTimeAsFileTime :: (lpSystemTimeAsFileTime: *FILETIME) #foreign kernel32`
- [OS == .WINDOWS] `GetSystemTimePreciseAsFileTime :: (lpSystemTimeAsFileTime: *FILETIME) #foreign kernel32`
- [OS == .WINDOWS] `SystemTimeToFileTime :: (lpSystemTime : *SYSTEMTIME, lpFileTime : *FILETIME) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `SystemTimeToTzSpecificLocalTime :: (lpTimeZone:*TIME_ZONE_INFORMATION, lpUniversalTime:*SYSTEMTIME, lpLocalTime:*SYSTEMTIME) -> BOOL #foreign kernel32`
- [OS == .WINDOWS] `PathFileExistsW :: (pszPath: *u16) -> s32 #foreign Shlwapi`
- [OS == .WINDOWS] `STICKYKEYS :: struct`
- [OS == .WINDOWS] `TOGGLEKEYS :: struct`
- [OS == .WINDOWS] `FILTERKEYS :: struct`
- [OS == .WINDOWS] `SM_CXSCREEN :: 0`
- [OS == .WINDOWS] `SM_CYSCREEN :: 1`
- [OS == .WINDOWS] `SM_CXVSCROLL :: 2`
- [OS == .WINDOWS] `SM_CYHSCROLL :: 3`
- [OS == .WINDOWS] `SM_CYCAPTION :: 4`
- [OS == .WINDOWS] `SM_CXBORDER :: 5`
- [OS == .WINDOWS] `SM_CYBORDER :: 6`
- [OS == .WINDOWS] `SM_CXDLGFRAME :: 7`
- [OS == .WINDOWS] `SM_CYDLGFRAME :: 8`
- [OS == .WINDOWS] `SM_CYVTHUMB :: 9`
- [OS == .WINDOWS] `SM_CXHTHUMB :: 10`
- [OS == .WINDOWS] `SM_CXICON :: 11`
- [OS == .WINDOWS] `SM_CYICON :: 12`
- [OS == .WINDOWS] `SM_CXCURSOR :: 13`
- [OS == .WINDOWS] `SM_CYCURSOR :: 14`
- [OS == .WINDOWS] `SM_CYMENU :: 15`
- [OS == .WINDOWS] `SM_CXFULLSCREEN :: 16`
- [OS == .WINDOWS] `SM_CYFULLSCREEN :: 17`
- [OS == .WINDOWS] `SM_CYKANJIWINDOW :: 18`
- [OS == .WINDOWS] `SM_MOUSEPRESENT :: 19`
- [OS == .WINDOWS] `SM_CYVSCROLL :: 20`
- [OS == .WINDOWS] `SM_CXHSCROLL :: 21`
- [OS == .WINDOWS] `SM_DEBUG :: 22`
- [OS == .WINDOWS] `SM_SWAPBUTTON :: 23`
- [OS == .WINDOWS] `SM_RESERVED1 :: 24`
- [OS == .WINDOWS] `SM_RESERVED2 :: 25`
- [OS == .WINDOWS] `SM_RESERVED3 :: 26`
- [OS == .WINDOWS] `SM_RESERVED4 :: 27`
- [OS == .WINDOWS] `SM_CXMIN :: 28`
- [OS == .WINDOWS] `SM_CYMIN :: 29`
- [OS == .WINDOWS] `SM_CXSIZE :: 30`
- [OS == .WINDOWS] `SM_CYSIZE :: 31`
- [OS == .WINDOWS] `SM_CXFRAME :: 32`
- [OS == .WINDOWS] `SM_CYFRAME :: 33`
- [OS == .WINDOWS] `SM_CXMINTRACK :: 34`
- [OS == .WINDOWS] `SM_CYMINTRACK :: 35`
- [OS == .WINDOWS] `SM_CXDOUBLECLK :: 36`
- [OS == .WINDOWS] `SM_CYDOUBLECLK :: 37`
- [OS == .WINDOWS] `SM_CXICONSPACING :: 38`
- [OS == .WINDOWS] `SM_CYICONSPACING :: 39`
- [OS == .WINDOWS] `SM_MENUDROPALIGNMENT :: 40`
- [OS == .WINDOWS] `SM_PENWINDOWS :: 41`
- [OS == .WINDOWS] `SM_DBCSENABLED :: 42`
- [OS == .WINDOWS] `SM_CMOUSEBUTTONS :: 43`
- [OS == .WINDOWS] `SM_CXFIXEDFRAME :: SM_CXDLGFRAME`
- [OS == .WINDOWS] `SM_CYFIXEDFRAME :: SM_CYDLGFRAME`
- [OS == .WINDOWS] `SM_CXSIZEFRAME :: SM_CXFRAME`
- [OS == .WINDOWS] `SM_CYSIZEFRAME :: SM_CYFRAME`
- [OS == .WINDOWS] `SM_SECURE :: 44`
- [OS == .WINDOWS] `SM_CXEDGE :: 45`
- [OS == .WINDOWS] `SM_CYEDGE :: 46`
- [OS == .WINDOWS] `SM_CXMINSPACING :: 47`
- [OS == .WINDOWS] `SM_CYMINSPACING :: 48`
- [OS == .WINDOWS] `SM_CXSMICON :: 49`
- [OS == .WINDOWS] `SM_CYSMICON :: 50`
- [OS == .WINDOWS] `SM_CYSMCAPTION :: 51`
- [OS == .WINDOWS] `SM_CXSMSIZE :: 52`
- [OS == .WINDOWS] `SM_CYSMSIZE :: 53`
- [OS == .WINDOWS] `SM_CXMENUSIZE :: 54`
- [OS == .WINDOWS] `SM_CYMENUSIZE :: 55`
- [OS == .WINDOWS] `SM_ARRANGE :: 56`
- [OS == .WINDOWS] `SM_CXMINIMIZED :: 57`
- [OS == .WINDOWS] `SM_CYMINIMIZED :: 58`
- [OS == .WINDOWS] `SM_CXMAXTRACK :: 59`
- [OS == .WINDOWS] `SM_CYMAXTRACK :: 60`
- [OS == .WINDOWS] `SM_CXMAXIMIZED :: 61`
- [OS == .WINDOWS] `SM_CYMAXIMIZED :: 62`
- [OS == .WINDOWS] `SM_NETWORK :: 63`
- [OS == .WINDOWS] `SM_CLEANBOOT :: 67`
- [OS == .WINDOWS] `SM_CXDRAG :: 68`
- [OS == .WINDOWS] `SM_CYDRAG :: 69`
- [OS == .WINDOWS] `SM_SHOWSOUNDS :: 70`
- [OS == .WINDOWS] `SM_CXMENUCHECK :: 71`
- [OS == .WINDOWS] `SM_CYMENUCHECK :: 72`
- [OS == .WINDOWS] `SM_SLOWMACHINE :: 73`
- [OS == .WINDOWS] `SM_MIDEASTENABLED :: 74`
- [OS == .WINDOWS] `SM_MOUSEWHEELPRESENT :: 75`
- [OS == .WINDOWS] `SM_XVIRTUALSCREEN :: 76`
- [OS == .WINDOWS] `SM_YVIRTUALSCREEN :: 77`
- [OS == .WINDOWS] `SM_CXVIRTUALSCREEN :: 78`
- [OS == .WINDOWS] `SM_CYVIRTUALSCREEN :: 79`
- [OS == .WINDOWS] `SM_CMONITORS :: 80`
- [OS == .WINDOWS] `SM_SAMEDISPLAYFORMAT :: 81`
- [OS == .WINDOWS] `SM_IMMENABLED :: 82`
- [OS == .WINDOWS] `SM_CXFOCUSBORDER :: 83`
- [OS == .WINDOWS] `SM_CYFOCUSBORDER :: 84`
- [OS == .WINDOWS] `SM_TABLETPC :: 86`
- [OS == .WINDOWS] `SM_MEDIACENTER :: 87`
- [OS == .WINDOWS] `SM_STARTER :: 88`
- [OS == .WINDOWS] `SM_SERVERR2 :: 89`
- [OS == .WINDOWS] `SM_MOUSEHORIZONTALWHEELPRESENT :: 91`
- [OS == .WINDOWS] `SM_CXPADDEDBORDER :: 92`
- [OS == .WINDOWS] `SM_DIGITIZER :: 94`
- [OS == .WINDOWS] `SM_MAXIMUMTOUCHES :: 95`
- [OS == .WINDOWS] `SM_CMETRICS :: 97`
- [OS == .WINDOWS] `SM_REMOTESESSION :: 0x1000`
- [OS == .WINDOWS] `SM_SHUTTINGDOWN :: 0x2000`
- [OS == .WINDOWS] `SM_REMOTECONTROL :: 0x2001`
- [OS == .WINDOWS] `SM_CARETBLINKINGENABLED :: 0x2002`
- [OS == .WINDOWS] `SM_CONVERTIBLESLATEMODE :: 0x2003`
- [OS == .WINDOWS] `SM_SYSTEMDOCKED :: 0x2004`
- [OS == .WINDOWS] `SPI_GETWORKAREA :: 0x0030`
- [OS == .WINDOWS] `SPI_GETFILTERKEYS :: 0x0032`
- [OS == .WINDOWS] `SPI_SETFILTERKEYS :: 0x0033`
- [OS == .WINDOWS] `SPI_GETTOGGLEKEYS :: 0x0034`
- [OS == .WINDOWS] `SPI_SETTOGGLEKEYS :: 0x0035`
- [OS == .WINDOWS] `SPI_GETSTICKYKEYS :: 0x003A`
- [OS == .WINDOWS] `SPI_SETSTICKYKEYS :: 0x003B`
- [OS == .WINDOWS] `using STICKYKEY_FLAGS :: enum_flags u32`
- [OS == .WINDOWS] `using TOGGLEKEY_FLAGS :: enum_flags u32`
- [OS == .WINDOWS] `using FILTERKEY_FLAGS :: enum_flags u32`
- [OS == .WINDOWS] `GetSystemMetrics :: (nIndex: s32) -> s32 #foreign user32`
- [OS == .WINDOWS] `SystemParametersInfoA :: (uiAction: u32, uiParam: u32, pvParam: *void, fWinIni: u32) -> BOOL #foreign user32`
- [OS == .WINDOWS] `SystemParametersInfoW :: (uiAction: u32, uiParam: u32, pvParam: *void, fWinIni: u32) -> BOOL #foreign user32`
- [OS == .WINDOWS] `BITMAPINFOHEADER :: struct`
- [OS == .WINDOWS] `BI_RGB :: 0`
- [OS == .WINDOWS] `BI_RLE8 :: 1`
- [OS == .WINDOWS] `BI_RLE4 :: 2`
- [OS == .WINDOWS] `BI_BITFIELDS :: 3`
- [OS == .WINDOWS] `BI_JPEG :: 4`
- [OS == .WINDOWS] `BI_PNG :: 5`
- [OS == .WINDOWS] `ROP :: enum u32`
- [OS == .WINDOWS] `BitBlt :: (hdc: HDC, x: s32, y: s32, cx: s32, cy: s32, hdcSrc: HDC, x1: s32, y1: s32, rop: ROP) -> BOOL #foreign gdi`
- [OS == .WINDOWS] `CreateCompatibleBitmap :: (hdc: HDC, cx: s32, cy: s32) -> HBITMAP #foreign gdi`
- [OS == .WINDOWS] `DIB_RGB_COLORS :: 0`
- [OS == .WINDOWS] `GetDIBits :: (hdc: HDC, hbm: HBITMAP, start: u32, cLines: u32, lpvBits: *void, lpbmi: *BITMAPINFO, usage: u32) -> s32 #foreign gdi`
- [OS == .WINDOWS] `BLTMODE :: enum u32`
- [OS == .WINDOWS] `SetStretchBltMode :: (hdc: HDC, mode: BLTMODE ) -> s32 #foreign gdi`
- [OS == .WINDOWS] `StretchBlt :: (hdcDest: HDC, xDest: s32, yDest: s32, wDest: s32, hDest: s32, hdcSrc: HDC, xSrc: s32, ySrc: s32, wSrc: s32, hSrc: s32, rop: ROP) -> BOOL #foreign gdi`
- [OS == .WINDOWS] `OFN_NOCHANGEDIR :: 0x00000008`
- [OS == .WINDOWS] `OPENFILENAME :: struct (T: Type)`
- [OS == .WINDOWS] `OPENFILENAMEA :: OPENFILENAME(T=u8)`
- [OS == .WINDOWS] `OPENFILENAMEW :: OPENFILENAME(T=u16)`
- [OS == .WINDOWS] `CommDlgExtendedError :: () -> u32 #foreign comdlg32`
- [OS == .WINDOWS] `GetOpenFileNameA :: (arg: *OPENFILENAMEA) -> BOOL #foreign comdlg32`
- [OS == .WINDOWS] `GetOpenFileNameW :: (arg: *OPENFILENAMEW) -> BOOL #foreign comdlg32`
- [OS == .WINDOWS] `GetSaveFileNameA :: (arg: *OPENFILENAMEA) -> BOOL #foreign comdlg32`
- [OS == .WINDOWS] `GetSaveFileNameW :: (arg: *OPENFILENAMEW) -> BOOL #foreign comdlg32`
- [OS == .WINDOWS] `using FILEOP_FLAGS :: enum_flags u16`
- [OS == .WINDOWS] `FO_MOVE :: 1`
- [OS == .WINDOWS] `FO_COPY :: 2`
- [OS == .WINDOWS] `FO_DELETE :: 3`
- [OS == .WINDOWS] `FO_RENAME :: 4`
- [OS == .WINDOWS] `SHFILEOPSTRUCTA :: struct`
- [OS == .WINDOWS] `SHFILEOPSTRUCTW :: struct`
- [OS == .WINDOWS] `DragAcceptFiles :: (hWnd: HWND, fAccept: BOOL) #foreign shell32`
- [OS == .WINDOWS] `DragFinish :: (hDrop: HDROP) #foreign shell32`
- [OS == .WINDOWS] `DragQueryFileW :: (hDrop: HDROP, iFile: u32, lpszFile: *u16, cch: u32) -> u32 #foreign shell32`
- [OS == .WINDOWS] `SHFileOperationA :: (FileOp: *SHFILEOPSTRUCTA) -> s32 #foreign shell32`
- [OS == .WINDOWS] `SHFileOperationW :: (FileOp: *SHFILEOPSTRUCTW) -> s32 #foreign shell32`
- [OS == .WINDOWS] `_CRT_WARN :: 0`
- [OS == .WINDOWS] `_CRT_ERROR :: 1`
- [OS == .WINDOWS] `_CRT_ASSERT :: 2`
- [OS == .WINDOWS] `_CRT_ERRCNT :: 3`
- [OS == .WINDOWS] `_CRTDBG_MODE_FILE :: 0x1`
- [OS == .WINDOWS] `_CRTDBG_MODE_DEBUG :: 0x2`
- [OS == .WINDOWS] `_CRTDBG_MODE_WNDW :: 0x4`
- [OS == .WINDOWS] `_CRTDBG_REPORT_MODE :: -1`
- [OS == .WINDOWS] `_beginthreadex :: (_Security: *void, _StackSize: u32, _StartAddress: *void, _ArgList: *void, _InitFlag: Creation_Flags, _ThrdAddr: *s32) -> *u32 #foreign Crt`
- [OS == .WINDOWS] `_endthreadex :: (return_code: u32) #foreign Crt`
- [OS == .WINDOWS] `getenv :: (var: *u8) -> *u8 #foreign Crt`
- [OS == .WINDOWS] `_wgetenv :: (var: *s16) -> *s16 #foreign Crt`
- [OS == .WINDOWS] `STD_INPUT_HANDLE :: -10`
- [OS == .WINDOWS] `STD_OUTPUT_HANDLE :: -11`
- [OS == .WINDOWS] `STD_ERROR_HANDLE :: -12`
- [OS == .WINDOWS] `FILE_SHARE_READ :: 0x1`
- [OS == .WINDOWS] `FILE_SHARE_WRITE :: 0x2`
- [OS == .WINDOWS] `FILE_SHARE_DELETE :: 0x4`
- [OS == .WINDOWS] `CREATE_NEW :: 1`
- [OS == .WINDOWS] `CREATE_ALWAYS :: 2`
- [OS == .WINDOWS] `OPEN_EXISTING :: 3`
- [OS == .WINDOWS] `OPEN_ALWAYS :: 4`
- [OS == .WINDOWS] `TRUNCATE_EXISTING :: 5`
- [OS == .WINDOWS] `FILE_ATTRIBUTE_READONLY :: 0x01`
- [OS == .WINDOWS] `FILE_ATTRIBUTE_HIDDEN :: 0x02`
- [OS == .WINDOWS] `FILE_ATTRIBUTE_SYSTEM :: 0x04`
- [OS == .WINDOWS] `FILE_ATTRIBUTE_DIRECTORY :: 0x10`
- [OS == .WINDOWS] `FILE_ATTRIBUTE_NORMAL :: 0x80`
- [OS == .WINDOWS] `FILE_ATTRIBUTE_REPARSE_POINT :: 0x400`
- [OS == .WINDOWS] `IO_REPARSE_TAG_SYMLINK :: 0xA000000C`
- [OS == .WINDOWS] `FILE_FLAG_BACKUP_SEMANTICS :: 0x02000000`
- [OS == .WINDOWS] `FILE_FLAG_OVERLAPPED :: 0x40000000`
- [OS == .WINDOWS] `INVALID_HANDLE_VALUE :: cast(HANDLE) -1`
- [OS == .WINDOWS] `HANDLE_FLAG_INHERIT :: 1`
- [OS == .WINDOWS] `HANDLE_FLAG_PROTECT_FROM_CLOSE :: 2`
- [OS == .WINDOWS] `MAKELANGID :: inline (usPrimaryLanguage: u16, usSubLanguage: u16) -> u16`
- [OS == .WINDOWS] `LANG_NEUTRAL :: 0`
- [OS == .WINDOWS] `LANG_ENGLISH :: 0x09`
- [OS == .WINDOWS] `SUBLANG_DEFAULT :: 0x1`
- [OS == .WINDOWS] `WideCharToMultiByte :: (CodePage: u32, dwFlags: s32, lpWideCharStr: *u16, cchWideChar: s32, lpMultiByteStr: *u8, cbMultiByte: s32, lpDefaultChar: *u8, lpUsedDefaultChar: *s32) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `MultiByteToWideChar :: (CodePage: u32, dwFlags: s32, lpMultiByteStr: *u8, cbMultiByte: s32, lpWideCharStr: *u16, cchWideChar: s32) -> s32 #foreign kernel32`
- [OS == .WINDOWS] `CP_ACP :: 0`
- [OS == .WINDOWS] `CP_UTF8 :: 65001`
- [OS == .WINDOWS] `WC_NO_BEST_FIT_CHARS :: 0x0000_0400`
- [OS == .WINDOWS] `WC_COMPOSITECHECK :: 0x0000_0200`
- [OS == .WINDOWS] `IUnknown_VTable :: struct`
- [OS == .WINDOWS] `IUnknown_QueryInterface :: inline (this: *IUnknown, riid: *IID, ppvObject: **void) -> HRESULT`
- [OS == .WINDOWS] `IUnknown_AddRef :: inline (this: *IUnknown) -> u32`
- [OS == .WINDOWS] `IUnknown_Release :: inline (this: *IUnknown) -> u32`
- [OS == .WINDOWS] `vtable :: (obj: *IUnknown) -> *IUnknown_VTable`
- [OS == .WINDOWS] `IUnknown_UUID :: "00000000-0000-0000-C000-000000000046"`
- [OS == .WINDOWS] `IUnknown :: struct`
- [OS == .WINDOWS] `safe_release :: (obj: *IUnknown) @Releases`
- [OS == .WINDOWS] `safe_release_and_reset :: (b: **$T/interface IUnknown)`
- [OS == .WINDOWS] `safe_release_and_reset :: (b: **IUnknown)`
- [OS == .WINDOWS] `string_to_guid :: uid`
- [OS == .WINDOWS] `uid :: (str: string) -> GUID`
- [OS == .WINDOWS] `FILE_MAP_ALL_ACCESS :: 0xf001f`
- [OS == .WINDOWS] `FILE_MAP_READ :: 4`
- [OS == .WINDOWS] `FILE_MAP_WRITE :: 2`
- [OS == .WINDOWS] `FILE_MAP_COPY :: 1`
- [OS == .WINDOWS] `ERROR_FILE_NOT_FOUND :: 0x002`
- [OS == .WINDOWS] `ERROR_ACCESS_DENIED :: 0x005`
- [OS == .WINDOWS] `ERROR_INVALID_HANDLE :: 0x006`
- [OS == .WINDOWS] `ERROR_BROKEN_PIPE :: 0x06D`
- [OS == .WINDOWS] `ERROR_ALREADY_EXISTS :: 0x0B7`
- [OS == .WINDOWS] `ERROR_PIPE_BUSY :: 0x0E7`
- [OS == .WINDOWS] `ERROR_NO_DATA :: 0x0E8`
- [OS == .WINDOWS] `ERROR_MORE_DATA :: 0x0EA`
- [OS == .WINDOWS] `WAIT_OBJECT_0 :: 0x000`
- [OS == .WINDOWS] `WAIT_TIMEOUT :: 0x102`
- [OS == .WINDOWS] `ERROR_PIPE_CONNECTED :: 0x217`
- [OS == .WINDOWS] `ERROR_IO_PENDING :: 0x3E5`
- [OS == .WINDOWS] `REG_NONE :: 0`
- [OS == .WINDOWS] `REG_SZ :: 1`
- [OS == .WINDOWS] `REG_DWORD :: 4`
- [OS == .WINDOWS] `MAX_PATH :: 260`
- [OS == .WINDOWS] `Msvc_Destructor_Flags :: enum_flags u32`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `ADDRESS64`

```jai
ADDRESS64 :: struct {
    Offset: u64;
    Segment: u16;
    Mode: ADDRESS_MODE;
}
```
### `ADDRESS_MODE`

```jai
ADDRESS_MODE :: enum s32 {
    AddrMode1616 :: 0;
    AddrMode1632 :: 1;
    AddrModeReal :: 2;
    AddrModeFlat :: 3;
}
```
### `BITMAP`

```jai
BITMAP :: struct {
  bmType: u32;
  bmWidth: u32;
  bmHeight: u32;
  bmWidthBytes: u32;
  bmPlanes: u16;
  bmBitsPixel: u16;
  bmBits: *void;
}
```
### `BITMAPFILEHEADER`

```jai
BITMAPFILEHEADER :: struct {
    bfType: u16;
    bfSize: u32;
    bfReserved1: u16;
    bfReserved2: u16;
    bfOffBits: u32;
}
```
### `BITMAPINFO`

```jai
BITMAPINFO :: struct {
    bmiHeader: BITMAPINFOHEADER;
    bmiColors: [1] RGBQUAD;
}
```
### `BITMAPINFOHEADER`

```jai
BITMAPINFOHEADER :: struct {
    biSize:          u32;
    biWidth:         s32;
    biHeight:        s32;
    biPlanes:        u16;
    biBitCount:      u16;
    biCompression:   u32;
    biSizeImage:     u32;
    biXPelsPerMeter: s32;
    biYPelsPerMeter: s32;
    biClrUsed:       u32;
    biClrImportant:  u32;
}
```
### `BLTMODE`

```jai
BLTMODE :: enum u32 {
    BLACKONWHITE :: 0x0001;
    WHITEONBLACK :: 0x0002;
    COLORONCOLOR :: 0x0003;
    HALFTONE :: 0x0004;
}
```
### `BOOL`

```jai
BOOL    :: enum s32 { FALSE :: 0; TRUE :: 1; }
```
### `BOOLEAN`

```jai
BOOLEAN :: enum u8 { FALSE :: 0; TRUE :: 1; }
```
### `CACHE_RELATIONSHIP`

```jai
CACHE_RELATIONSHIP :: struct {
    Level:         u8;
    Associativity: u8;
    LineSize:      u16;
    CacheSize:     u32;
    Type: PROCESSOR_CACHE_TYPE;
    Reserved: [18] u8;
    GroupCount:    u16;
    GroupMask: GROUP_AFFINITY;
}
```
### `CONDITION_VARIABLE`

```jai
CONDITION_VARIABLE :: struct {
    Ptr: *void;
}
```
### `CONSOLE_SCREEN_BUFFER_INFO`

```jai
CONSOLE_SCREEN_BUFFER_INFO :: struct {
    dwSize, dwCursorPosition: COORD;
    wAttributes: u16;
    srWindow: SMALL_RECT;
    dwMaximumWindowSize: COORD;
}
```
### `CONTEXT` [CPU == .ARM64]

The arm64 thread context (`ARM64_NT_CONTEXT`), 912 bytes.

```jai
CONTEXT :: struct #type_info_none #align 16 {
    ContextFlags: u32;
    Cpsr: u32;

    union {
        X: [31] u64;
        struct {
            X0: u64;
            X1: u64;
            X2: u64;
            X3: u64;
            X4: u64;
            X5: u64;
            X6: u64;
            X7: u64;
            X8: u64;
            X9: u64;
            X10: u64;
            X11: u64;
            X12: u64;
            X13: u64;
            X14: u64;
            X15: u64;
            X16: u64;
            X17: u64;
            X18: u64;
            X19: u64;
            X20: u64;
            X21: u64;
            X22: u64;
            X23: u64;
            X24: u64;
            X25: u64;
            X26: u64;
            X27: u64;
            X28: u64;
            Fp: u64;
            Lr: u64;
        }
    }

    Sp: u64;
    Pc: u64;

    V: [32] M128A;
    Fpcr: u32;
    Fpsr: u32;

    Bcr: [8] u32;
    Bvr: [8] u64;
    Wcr: [2] u32;
    Wvr: [2] u64;
}
```

### `CONTEXT` [CPU == .X64]

```jai
CONTEXT :: struct #type_info_none #align 16 {
    P1Home: u64;
    P2Home: u64;
    P3Home: u64;
    P4Home: u64;
    P5Home: u64;
    P6Home: u64;

    ContextFlags: u32;
    MxCsr: u32;

    SegCs: u16;
    SegDs: u16;
    SegEs: u16;
    SegFs: u16;
    SegGs: u16;
    SegSs: u16;
    EFlags: u32;

    Dr0: u64;
    Dr1: u64;
    Dr2: u64;
    Dr3: u64;
    Dr6: u64;
    Dr7: u64;

    Rax: u64;
    Rcx: u64;
    Rdx: u64;
    Rbx: u64;
    Rsp: u64;
    Rbp: u64;
    Rsi: u64;
    Rdi: u64;
    R8: u64;
    R9: u64;
    R10: u64;
    R11: u64;
    R12: u64;
    R13: u64;
    R14: u64;
    R15: u64;

    Rip: u64;

    union {
        FltSave: XSAVE_FORMAT;
        struct {
            Header: [2]M128A;
            Legacy: [8]M128A;
            Xmm0: M128A;
            Xmm1: M128A;
            Xmm2: M128A;
            Xmm3: M128A;
            Xmm4: M128A;
            Xmm5: M128A;
            Xmm6: M128A;
            Xmm7: M128A;
            Xmm8: M128A;
            Xmm9: M128A;
            Xmm10: M128A;
            Xmm11: M128A;
            Xmm12: M128A;
            Xmm13: M128A;
            Xmm14: M128A;
            Xmm15: M128A;
        }
    }

    VectorRegister: [26]M128A;
    VectorControl: u64;

    DebugControl: u64;
    LastBranchToRip: u64;
    LastBranchFromRip: u64;
    LastExceptionToRip: u64;
    LastExceptionFromRip: u64;
}
```
### `COORD`

```jai
COORD :: struct {
    x, y: s16;
}
```
### `Creation_Flags`

```jai
using Creation_Flags :: enum_flags u32 {
    DEBUG_PROCESS                    :: 0x00000001;
    DEBUG_ONLY_THIS_PROCESS          :: 0x00000002;
    CREATE_SUSPENDED                 :: 0x00000004;
    DETACHED_PROCESS                 :: 0x00000008;

    CREATE_NEW_CONSOLE               :: 0x00000010;
    NORMAL_PRIORITY_CLASS            :: 0x00000020;
    IDLE_PRIORITY_CLASS              :: 0x00000040;
    HIGH_PRIORITY_CLASS              :: 0x00000080;

    REALTIME_PRIORITY_CLASS          :: 0x00000100;
    CREATE_NEW_PROCESS_GROUP         :: 0x00000200;
    CREATE_UNICODE_ENVIRONMENT       :: 0x00000400;
    CREATE_SEPARATE_WOW_VDM          :: 0x00000800;

    CREATE_SHARED_WOW_VDM            :: 0x00001000;
    CREATE_FORCEDOS                  :: 0x00002000;
    BELOW_NORMAL_PRIORITY_CLASS      :: 0x00004000;
    ABOVE_NORMAL_PRIORITY_CLASS      :: 0x00008000;

    INHERIT_PARENT_AFFINITY          :: 0x00010000;
    INHERIT_CALLER_PRIORITY          :: 0x00020000;
    CREATE_PROTECTED_PROCESS         :: 0x00040000;
    EXTENDED_STARTUPINFO_PRESENT     :: 0x00080000;

    PROCESS_MODE_BACKGROUND_BEGIN    :: 0x00100000;
    PROCESS_MODE_BACKGROUND_END      :: 0x00200000;
    CREATE_SECURE_PROCESS            :: 0x00400000;

    CREATE_BREAKAWAY_FROM_JOB        :: 0x01000000;
    CREATE_PRESERVE_CODE_AUTHZ_LEVEL :: 0x02000000;
    CREATE_DEFAULT_ERROR_MODE        :: 0x04000000;
    CREATE_NO_WINDOW                 :: 0x08000000;

    PROFILE_USER                     :: 0x10000000;
    PROFILE_KERNEL                   :: 0x20000000;
    PROFILE_SERVER                   :: 0x40000000;
    CREATE_IGNORE_SYSTEM_DEFAULT     :: 0x80000000;
}
```
### `DEVMODEA`

```jai
DEVMODEA :: struct {
    dmDeviceName: [CCHDEVICENAME] u8;
    dmSpecVersion: u16;
    dmDriverVersion: u16;
    dmSize: u16;
    dmDriverExtra: u16;
    dmFields: u32;
    DUMMYUNIONNAME: union {
        DUMMYSTRUCTNAME: struct {
            dmOrientation: s16;
            dmPaperSize: s16;
            dmPaperLength: s16;
            dmPaperWidth: s16;
            dmScale: s16;
            dmCopies: s16;
            dmDefaultSource: s16;
            dmPrintQuality: s16;
        };
        dmPosition: POINTL;
        DUMMYSTRUCTNAME2: struct {
            dmPosition: POINTL;
            dmDisplayOrientation: u32;
            dmDisplayFixedOutput: u32;
        };
    };
    dmColor:  s16;
    dmDuplex: s16;
    dmYResolution: s16;
    dmTTOption: s16;
    dmCollate:  s16;
    dmFormName: [CCHFORMNAME] u8;
    dmLogPixels:  u16;
    dmBitsPerPel: u32;
    dmPelsWidth:  u32;
    dmPelsHeight: u32;
    DUMMYUNIONNAME2: union {
        dmDisplayFlags: u32;
        dmNup: u32;
    };
    dmDisplayFrequency: u32;
    dmICMMethod: u32;
    dmICMIntent: u32;
    dmMediaType: u32;
    dmDitherType: u32;
    dmReserved1: u32;
    dmReserved2: u32;
    dmPanningWidth: u32;
    dmPanningHeight: u32;
}
```
### `DEV_BROADCAST_DEVICEINTERFACE_W`

```jai
DEV_BROADCAST_DEVICEINTERFACE_W :: struct {
    dbcc_size: DWORD;
    dbcc_devicetype: DWORD;
    dbcc_reserved: DWORD;
    dbcc_classguid: GUID;
    dbcc_name: [1] u16;
}
```
### `DEV_BROADCAST_HDR`

```jai
DEV_BROADCAST_HDR :: struct {
    dbch_size: u32;
    dbch_devicetype: u32;
    dbch_reserved: u32;
}
```
### `DPI_AWARENESS`

```jai
using DPI_AWARENESS :: enum s32 {
    DPI_AWARENESS_INVALID :: -1;
    DPI_AWARENESS_UNAWARE :: 0;
    DPI_AWARENESS_SYSTEM_AWARE :: 1;
    DPI_AWARENESS_PER_MONITOR_AWARE :: 2;
}
```
### `EXCEPTION_POINTERS`

```jai
EXCEPTION_POINTERS :: struct {
    ExceptionRecord : *EXCEPTION_RECORD;
    ContextRecord : *CONTEXT;
}
```
### `EXCEPTION_RECORD`

```jai
EXCEPTION_RECORD :: struct {
    ExceptionCode: u32;
    ExceptionFlags: u32;
    ExceptionRecord: *EXCEPTION_RECORD;
    ExceptionAddress: *void;
    NumberParameters: u32;
    ExceptionInformation: [EXCEPTION_MAXIMUM_PARAMETERS]u64;
}
```
### `FILEOP_FLAGS`

```jai
using FILEOP_FLAGS :: enum_flags u16 {
    FOF_MULTIDESTFILES        :: 0x0001;
    FOF_CONFIRMMOUSE          :: 0x0002;
    FOF_SILENT                :: 0x0004;
    FOF_RENAMEONCOLLISION     :: 0x0008;
    FOF_NOCONFIRMATION        :: 0x0010;
    FOF_WANTMAPPINGHANDLE     :: 0x0020;
    FOF_ALLOWUNDO             :: 0x0040;
    FOF_FILESONLY             :: 0x0080;
    FOF_SIMPLEPROGRESS        :: 0x0100;
    FOF_NOCONFIRMMKDIR        :: 0x0200;
    FOF_NOERRORUI             :: 0x0400;
    FOF_NOCOPYSECURITYATTRIBS :: 0x0800;
    FOF_NORECURSION           :: 0x1000;
    FOF_NO_CONNECTED_ELEMENTS :: 0x2000;
    FOF_WANTNUKEWARNING       :: 0x4000;
    FOF_NORECURSEREPARSE      :: 0x8000;
    FOF_NO_UI :: FOF_SILENT | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_NOCONFIRMMKDIR;
}
```
### `FILETIME`

```jai
FILETIME :: struct {
    union {
        QuadPart: u64 #align 4;

        struct {
            dwLowDateTime  : u32;
            dwHighDateTime : u32;
        }
    };
}
```
### `FILE_END_OF_FILE_INFO`

```jai
FILE_END_OF_FILE_INFO :: struct {
    EndOfFile: LARGE_INTEGER;
}
```
### `FILE_FULL_DIR_INFO`

```jai
FILE_FULL_DIR_INFO :: struct {
    NextEntryOffset: u32;
    FileIndex:       u32;
    CreationTime:    LARGE_INTEGER;
    LastAccessTime:  LARGE_INTEGER;
    LastWriteTime:   LARGE_INTEGER;
    ChangeTime:      LARGE_INTEGER;
    EndOfFile:       LARGE_INTEGER;
    AllocationSize:  LARGE_INTEGER;
    FileAttributes:  u32;
    FileNameLength:  u32;
    EaSize:          u32;
    FileName:     [1]u16;
}
```
### `FILE_INFO_BY_HANDLE_CLASS`

```jai
using FILE_INFO_BY_HANDLE_CLASS :: enum {
    FileBasicInfo;
    FileStandardInfo;
    FileNameInfo;
    FileRenameInfo;
    FileDispositionInfo;
    FileAllocationInfo;
    FileEndOfFileInfo;
    FileStreamInfo;
    FileCompressionInfo;
    FileAttributeTagInfo;
    FileIdBothDirectoryInfo;
    FileIdBothDirectoryRestartInfo;
    FileIoPriorityHintInfo;
    FileRemoteProtocolInfo;
    FileFullDirectoryInfo;
    FileFullDirectoryRestartInfo;
    FileStorageInfo;
    FileAlignmentInfo;
    FileIdInfo;
    FileIdExtdDirectoryInfo;
    FileIdExtdDirectoryRestartInfo;
    FileDispositionInfoEx;
    FileRenameInfoEx;
    FileCaseSensitiveInfo;
    FileNormalizedNameInfo;
    MaximumFileInfoByHandleClass;
}
```
### `FILE_NOTIFY_INFORMATION`

```jai
FILE_NOTIFY_INFORMATION :: struct {
    NextEntryOffset: s32;
    Action:          s32;
    FileNameLength:  s32;
    FileName: [1]    s16;
}
```
### `FILE_STANDARD_INFO`

```jai
FILE_STANDARD_INFO :: struct {
    AllocationSize: LARGE_INTEGER;
    EndOfFile:      LARGE_INTEGER;
    NumberOfLinks:  u32;
    DeletePending:  BOOLEAN;
    Directory:      BOOLEAN;
}
```
### `FILTERKEYS`

```jai
FILTERKEYS :: struct {
    cbSize:      u32 = 24;
    dwFlags:     FILTERKEY_FLAGS;
    iWaitMSec:   u32;
    iDelayMSec:  u32;
    iRepeatMSec: u32;
    iBounceMSec: u32;
}
```
### `FILTERKEY_FLAGS`

```jai
using FILTERKEY_FLAGS :: enum_flags u32 {
    FKF_FILTERKEYSON  :: 0x00000001;
    FKF_AVAILABLE     :: 0x00000002;
    FKF_HOTKEYACTIVE  :: 0x00000004;
    FKF_CONFIRMHOTKEY :: 0x00000008;
    FKF_HOTKEYSOUND   :: 0x00000010;
    FKF_INDICATOR     :: 0x00000020;
    FKF_CLICKON       :: 0x00000040;
}
```
### `FINDEX_INFO_LEVELS`

```jai
using FINDEX_INFO_LEVELS :: enum {
  FindExInfoStandard;
  FindExInfoBasic;
  FindExInfoMaxInfoLevel;
}
```
### `FINDEX_SEARCH_OPS`

```jai
using FINDEX_SEARCH_OPS :: enum {
  FindExSearchNameMatch;
  FindExSearchLimitToDirectories;
  FindExSearchLimitToDevices;
  FindExSearchMaxSearchOp;
}
```
### `FLASHW`

```jai
using FLASHW :: enum_flags u32 {
    FLASHW_STOP :: 0;
    FLASHW_CAPTION :: 0x00000001;
    FLASHW_TRAY :: 0x00000002;
    FLASHW_ALL :: 0x00000003;
    FLASHW_TIMER :: 0x00000004;
    FLASHW_TIMERNOFG :: 0x0000000C;
}
```
### `FLASHWINFO`

```jai
FLASHWINFO :: struct {
    cbSize : u32;
    hwnd : HWND;
    dwFlags : FLASHW;
    uCount : u32;
    dwTimeout : u32;
}
```
### `GROUP_AFFINITY`

```jai
GROUP_AFFINITY :: struct {
    Mask:     u64;
    Group:    u16;
    Reserved: [3] u16;
}
```
### `GROUP_RELATIONSHIP`

```jai
GROUP_RELATIONSHIP :: struct {
    MaximumGroupCount: u16;
    ActiveGroupCount:  u16;
    Reserved:     [20] u8;
    GroupInfo: PROCESSOR_GROUP_INFO;
}
```
### `GUID`

```jai
GUID :: struct {
    Data1: u32;
    Data2: u16;
    Data3: u16;
    Data4: [8] u8;
}
```
### `ICONINFO`

```jai
ICONINFO :: struct {
    fIcon: BOOL;
    xHotspot: u32;
    yHotspot: u32;
    hbmMask: HBITMAP;
    hbmColor: HBITMAP;
}
```
### `ICONINFOEXW`

```jai
ICONINFOEXW :: struct {
    cbSize: u32;
    fIcon: BOOL;
    xHotspot: u32;
    yHotspot: u32;
    hbmMask: HBITMAP;
    hbmColor: HBITMAP;
    wResID: u16;
    szModName: [MAX_PATH]u16;
    szResName: [MAX_PATH]u16;
}
```
### `IMAGEHLP_LINE64`

```jai
IMAGEHLP_LINE64 :: struct {
    SizeOfStruct : u32;
    Key : *void;
    LineNumber : u32;
    FileName : *u8;
    Address: u64;
}
```
### `IMAGEHLP_LINEW64`

```jai
IMAGEHLP_LINEW64 :: struct {
    SizeOfStruct : u32;
    Key : *void;
    LineNumber : u32;
    FileName : *u16;
    Address : u64;
}
```
### `IMAGEHLP_SYMBOL64`

```jai
IMAGEHLP_SYMBOL64 :: struct {
    SizeOfStruct : u32;
    Address: u64;
    Size : u32;
    Flags : u32;

    MaxNameLength : u32;
    Name: [1]u8;
}
```
### `IO_COUNTERS`

```jai
IO_COUNTERS :: struct {
    ReadOperationCount:  u64;
    WriteOperationCount: u64;
    OtherOperationCount: u64;
    ReadTransferCount:   u64;
    WriteTransferCount:  u64;
    OtherTransferCount:  u64;
}
```
### `IUnknown`

```jai
IUnknown :: struct {
    vtable: *IUnknown_VTable;
}
```
### `IUnknown_VTable`

```jai
IUnknown_VTable :: struct {
    QueryInterface: (this: *IUnknown, riid: *IID, ppvObject: **void) -> HRESULT #cpp_method;
    AddRef: (this: *IUnknown) -> u32 #cpp_method;
    Release: (this: *IUnknown) -> u32 #cpp_method;
}
```
### `JOBOBJECTINFOCLASS`

```jai
JOBOBJECTINFOCLASS :: enum {
    BasicAccountingInformation :: 1;
    BasicLimitInformation;
    BasicProcessIdList;
    BasicUIRestrictions;
    SecurityLimitInformation;
    EndOfJobTimeInformation;
    AssociateCompletionPortInformation;
    BasicAndIoAccountingInformation;
    ExtendedLimitInformation;
    JobSetInformation;
    GroupInformation;
    NotificationLimitInformation;
    LimitViolationInformation;
    GroupInformationEx;
    CpuRateControlInformation;
    CompletionFilter;
    CompletionCounter;
    Reserved1Information :: 18;
    Reserved2Information;
    Reserved3Information;
    Reserved4Information;
    Reserved5Information;
    Reserved6Information;
    Reserved7Information;
    Reserved8Information;
    Reserved9Information;
    Reserved10Information;
    Reserved11Information;
    Reserved12Information;
    Reserved13Information;
    Reserved14Information :: 31;
    NetRateControlInformation;
    NotificationLimitInformation2;
    LimitViolationInformation2;
    CreateSilo;
    SiloBasicInformation;
    Reserved15Information :: 37;
    Reserved16Information :: 38;
    Reserved17Information :: 39;
    Reserved18Information :: 40;
    Reserved19Information :: 41;
    Reserved20Information :: 42;
    Reserved21Information :: 43;
    Reserved22Information :: 44;
    Reserved23Information :: 45;
    Reserved24Information :: 46;
    Reserved25Information :: 47;
    MaxJobObjectInfoClass;
}
```
### `JOBOBJECT_BASIC_LIMIT_INFORMATION`

```jai
JOBOBJECT_BASIC_LIMIT_INFORMATION :: struct {
    PerProcessUserTimeLimit: u64;
    PerJobUserTimeLimit:     u64;
    LimitFlags:              Job_Object_Limit;
    MinimumWorkingSetSize:   SIZE_T;
    MaximumWorkingSetSize:   SIZE_T;
    ActiveProcessLimit:      u32;
    Affinity:                *u32;
    PriorityClass:           u32;
    SchedulingClass:         u32;
}
```
### `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`

```jai
JOBOBJECT_EXTENDED_LIMIT_INFORMATION :: struct {
  BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION;
  IoInfo:                  IO_COUNTERS;
  ProcessMemoryLimit: SIZE_T;
  JobMemoryLimit: SIZE_T;
  PeakProcessMemoryUsed: SIZE_T;
  PeakJobMemoryUsed: SIZE_T;
}
```
### `Job_Object_Limit`

```jai
using Job_Object_Limit :: enum u32 {

    JOB_OBJECT_LIMIT_WORKINGSET                 :: 0x00000001;
    JOB_OBJECT_LIMIT_PROCESS_TIME               :: 0x00000002;
    JOB_OBJECT_LIMIT_JOB_TIME                   :: 0x00000004;
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS             :: 0x00000008;
    JOB_OBJECT_LIMIT_AFFINITY                   :: 0x00000010;
    JOB_OBJECT_LIMIT_PRIORITY_CLASS             :: 0x00000020;
    JOB_OBJECT_LIMIT_PRESERVE_JOB_TIME          :: 0x00000040;
    JOB_OBJECT_LIMIT_SCHEDULING_CLASS           :: 0x00000080;

    JOB_OBJECT_LIMIT_PROCESS_MEMORY             :: 0x00000100;
    JOB_OBJECT_LIMIT_JOB_MEMORY                 :: 0x00000200;
    JOB_OBJECT_LIMIT_JOB_MEMORY_HIGH            :: JOB_OBJECT_LIMIT_JOB_MEMORY;
    JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION :: 0x00000400;
    JOB_OBJECT_LIMIT_BREAKAWAY_OK               :: 0x00000800;
    JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK        :: 0x00001000;
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE          :: 0x00002000;
    JOB_OBJECT_LIMIT_SUBSET_AFFINITY            :: 0x00004000;
    JOB_OBJECT_LIMIT_JOB_MEMORY_LOW             :: 0x00008000;

    JOB_OBJECT_LIMIT_JOB_READ_BYTES             :: 0x00010000;
    JOB_OBJECT_LIMIT_JOB_WRITE_BYTES            :: 0x00020000;
    JOB_OBJECT_LIMIT_RATE_CONTROL               :: 0x00040000;
    JOB_OBJECT_LIMIT_CPU_RATE_CONTROL           :: JOB_OBJECT_LIMIT_RATE_CONTROL;
    JOB_OBJECT_LIMIT_IO_RATE_CONTROL            :: 0x00080000;
    JOB_OBJECT_LIMIT_NET_RATE_CONTROL           :: 0x00100000;
}
```
### `KDHELP64`

```jai
KDHELP64 :: struct {
    Thread: u64;
    ThCallbackStack: u32;
    ThCallbackBStore: u32;
    NextCallback: u32;
    FramePointer: u32;
    KiCallUserMode: u64;
    KeUserCallbackDispatcher: u64;
    SystemRangeStart: u64;
    KiUserExceptionDispatcher: u64;
    StackBase: u64;
    StackLimit: u64;
    Reserved: [5]u64;
}
```
### `LARGE_INTEGER`

```jai
LARGE_INTEGER :: union {
    struct {
        LowPart : u32;
        HighPart : s32;
    }
    QuadPart : s64;
}
```
### `LIST_ENTRY`

```jai
LIST_ENTRY :: struct {
   Flink: *LIST_ENTRY;
   Blink: *LIST_ENTRY;
}
```
### `LOGICAL_PROCESSOR_RELATIONSHIP`

```jai
using LOGICAL_PROCESSOR_RELATIONSHIP :: enum s32 {
    RelationProcessorCore;
    RelationNumaNode;
    RelationCache;
    RelationProcessorPackage;
    RelationGroup;
    RelationProcessorDie;
    RelationNumaNodeEx;
    RelationProcessorModule;
    RelationAll :: 0xffff;
}
```
### `LUID`

```jai
LUID :: struct {
    LowPart:  u32;
    HighPart: s32;
}
```
### `M128A`

```jai
M128A :: struct #align 16 {
    Low: u64;
    High: s64;
}
```
### `MEMORYSTATUSEX`

```jai
MEMORYSTATUSEX :: struct {
    dwLength               : u32;
    dwMemoryLoad           : u32;
    ullTotalPhys           : u64;
    ullAvailPhys           : u64;
    ullTotalPageFile       : u64;
    ullAvailPageFile       : u64;
    ullTotalVirtual        : u64;
    ullAvailVirtual        : u64;
    ullAvailExtendedVirtual: u64;
}
```
### `MEMORY_BASIC_INFORMATION`

```jai
MEMORY_BASIC_INFORMATION :: struct {
    BaseAddress:       *void;
    AllocationBase:    *void;
    AllocationProtect: DWORD;
    PartitionId:       WORD;
    RegionSize:        SIZE_T;
    State:             DWORD;
    Protect:           DWORD;
    Type:              DWORD;
}
```
### `MINIDUMP_CALLBACK_INFORMATION`

```jai
MINIDUMP_CALLBACK_INFORMATION :: struct {
    CallbackRoutine: MINIDUMP_CALLBACK_ROUTINE #align 4;
    CallbackParam: *void #align 4;
}
```
### `MINIDUMP_CALLBACK_INPUT`

```jai
MINIDUMP_CALLBACK_INPUT :: struct {
    ProcessId: u32 #align 4;
    ProcessHandle: HANDLE #align 4;
    CallbackType: MINIDUMP_CALLBACK_TYPE #align 4;
    union {
        Status: HRESULT;

        SecondaryFlags: u32;

    }
}
```
### `MINIDUMP_CALLBACK_OUTPUT`

```jai
MINIDUMP_CALLBACK_OUTPUT :: struct {
    union {
        ModuleWriteFlags: u32 #align 4;
        ThreadWriteFlags: u32 #align 4;
        SecondaryFlags: u32 #align 4;
        struct {
            MemoryBase: u64 #align 4;
            MemorySize: u32 #align 4;
        }
        struct {
            CheckCancel: BOOL #align 4;
            Cancel: BOOL #align 4;
        }
        Handle: HANDLE #align 4;
        struct {
            VmRegion: MINIDUMP_MEMORY_INFO #align 4;
            Continue: BOOL #align 4;
        }
        struct {
            VmQueryStatus: HRESULT #align 4;
            VmQueryResult: MINIDUMP_MEMORY_INFO #align 4;
        }
        struct {
            VmReadStatus: HRESULT #align 4;
            VmReadBytesCompleted: u32 #align 4;
        }
        Status: HRESULT #align 4;
    }
}
```
### `MINIDUMP_CALLBACK_TYPE`

```jai
MINIDUMP_CALLBACK_TYPE :: enum u32 {
    ModuleCallback;
    ThreadCallback;
    ThreadExCallback;
    IncludeThreadCallback;
    IncludeModuleCallback;
    MemoryCallback;
    CancelCallback;
    WriteKernelMinidumpCallback;
    KernelMinidumpStatusCallback;
    RemoveMemoryCallback;
    IncludeVmRegionCallback;
    IoStartCallback;
    IoWriteAllCallback;
    IoFinishCallback;
    ReadMemoryFailureCallback;
    SecondaryFlagsCallback;
    IsProcessSnapshotCallback;
    VmStartCallback;
    VmQueryCallback;
    VmPreReadCallback;
    VmPostReadCallback;
}
```
### `MINIDUMP_EXCEPTION_INFORMATION`

```jai
MINIDUMP_EXCEPTION_INFORMATION :: struct {
    ThreadId : DWORD #align 4;
    ExceptionPointers: *EXCEPTION_POINTERS #align 4;
    ClientPointers: BOOL #align 4;
}
```
### `MINIDUMP_EXCEPTION_INFORMATION64`

```jai
MINIDUMP_EXCEPTION_INFORMATION64 :: struct {
    ThreadId: DWORD #align 4;
    ExceptionRecord: u64 #align 4;
    ContextRecord: u64 #align 4;
    ClientPointers: BOOL #align 4;
}
```
### `MINIDUMP_MEMORY_INFO`

```jai
MINIDUMP_MEMORY_INFO :: struct {
    BaseAddress: u64 #align 4;
    AllocationBase: u64 #align 4;
    AllocationProtect: u32 #align 4;
    __alignment1: u32 #align 4;
    RegionSize: u64 #align 4;
    State: u32 #align 4;
    Protect: u32 #align 4;
    Type: u32 #align 4;
    __alignment2: u32 #align 4;
}
```
### `MINIDUMP_TYPE`

```jai
MINIDUMP_TYPE :: enum_flags u32 {
    MiniDumpNormal                         :: 0x00000000;
    MiniDumpWithDataSegs                   :: 0x00000001;
    MiniDumpWithFullMemory                 :: 0x00000002;
    MiniDumpWithHandleData                 :: 0x00000004;
    MiniDumpFilterMemory                   :: 0x00000008;
    MiniDumpScanMemory                     :: 0x00000010;
    MiniDumpWithUnloadedModules            :: 0x00000020;
    MiniDumpWithIndirectlyReferencedMemory :: 0x00000040;
    MiniDumpFilterModulePaths              :: 0x00000080;
    MiniDumpWithProcessThreadData          :: 0x00000100;
    MiniDumpWithPrivateReadWriteMemory     :: 0x00000200;
    MiniDumpWithoutOptionalData            :: 0x00000400;
    MiniDumpWithFullMemoryInfo             :: 0x00000800;
    MiniDumpWithThreadInfo                 :: 0x00001000;
    MiniDumpWithCodeSegs                   :: 0x00002000;
    MiniDumpWithoutAuxiliaryState          :: 0x00004000;
    MiniDumpWithFullAuxiliaryState         :: 0x00008000;
    MiniDumpWithPrivateWriteCopyMemory     :: 0x00010000;
    MiniDumpIgnoreInaccessibleMemory       :: 0x00020000;
    MiniDumpWithTokenInformation           :: 0x00040000;
    MiniDumpWithModuleHeaders              :: 0x00080000;
    MiniDumpFilterTriage                   :: 0x00100000;
    MiniDumpWithAvxXStateContext           :: 0x00200000;
    MiniDumpWithIptTrace                   :: 0x00400000;
    MiniDumpScanInaccessiblePartialPages   :: 0x00800000;
    MiniDumpFilterWriteCombinedMemory      :: 0x01000000;
    MiniDumpValidTypeFlags                 :: 0x01ffffff;
}
```
### `MINIDUMP_USER_STREAM`

```jai
MINIDUMP_USER_STREAM :: struct {
    Type : u32 #align 4;
    BufferSize : u32 #align 4;
    Buffer : *void #align 4;
}
```
### `MINIDUMP_USER_STREAM_INFORMATION`

```jai
MINIDUMP_USER_STREAM_INFORMATION :: struct {
    UserStreamCount: u32 #align 4;
    UserStreamArray: *MINIDUMP_USER_STREAM #align 4;
}
```
### `MMTIME`

```jai
MMTIME :: struct {
    wType: u32;
    u: union {
        ms: u32;
        sample: u32;
        cb: u32;
        ticks: u32;
        smpte: struct {
            hour: u8;
            min: u8;
            sec: u8;
            frame: u8;
            fps: u8;
            dummy: u8;
            pad: [2]u8;
        };
        midi: struct {
            songptrpos: u32;
        };
    };
}
```
### `MONITORINFO`

```jai
MONITORINFO :: struct {
    cbSize: s32;
    rcMonitor: RECT;
    rcWork: RECT;
    dwFlags: s32;
}
```
### `MONITORINFOEXA`

```jai
MONITORINFOEXA :: struct {
    #as using entity: MONITORINFO;
    szDevice: [CCHDEVICENAME] u8;
}
```
### `MONITOR_DPI_TYPE`

```jai
using MONITOR_DPI_TYPE :: enum s32 {
    MDT_EFFECTIVE_DPI;
    MDT_ANGULAR_DPI;
    MDT_RAW_DPI;
    MDT_DEFAULT;
}
```
### `MSG`

```jai
MSG :: struct {
    hwnd:     HWND;
    message:  u32;
    wParam:   WPARAM;
    lParam:   LPARAM;
    time:     s32;
    pt:       POINT;
    lPrivate: s32;
}
```
### `Msvc_Destructor_Flags`

```jai
Msvc_Destructor_Flags :: enum_flags u32 {
    NONE :: 0;
    FREE_MEMORY :: 1;
    ARRAY_DESTRUCTOR :: 2;
}
```
### `NUMA_NODE_RELATIONSHIP`

```jai
NUMA_NODE_RELATIONSHIP :: struct {
    NodeNumber:    u32;
    Reserved: [18] u8;
    GroupCount:    u16;
    GroupMask: GROUP_AFFINITY;
}
```
### `OPENFILENAME`

```jai
OPENFILENAME :: struct (T: Type) {
    lStructSize :        u32;
    hwndOwner :          HWND;
    hInstance :          HINSTANCE;
    lpstrFilter :       *T;
    lpstrCustomFilter : *T;
    nMaxCustFilter :     u32;
    nFilterIndex :       u32;
    lpstrFile :         *T;
    nMaxFile :           u32;
    lpstrFileTitle :    *T;
    nMaxFileTitle :      u32;
    lpstrInitialDir :   *T;
    lpstrTitle :        *T;
    Flags :              u32;
    nFileOffset :        u16;
    nFileExtension :     u16;
    lpstrDefExt :       *T;
    lCustData :          LPARAM;
    lpfnHook :          *void;
    lpTemplateName :    *T;
    pvReserved :        *void;
    dwReserved :         u32;
    FlagsEx :            u32;
}
```
### `OSVERSIONINFOA`

```jai
OSVERSIONINFOA   :: struct {
    dwOSVersionInfoSize: u32;
    dwMajorVersion: u32;
    dwMinorVersion: u32;
    dwBuildNumber: u32;
    dwPlatformId: u32;
    szCSDVersion: [128]u8;
}
```
### `OSVERSIONINFOEXA`

```jai
OSVERSIONINFOEXA :: struct {
    #as using osversioninfo: OSVERSIONINFOA;
    wServicePackMajor: u16;
    wServicePackMinor: u16;
    wSuiteMask: u16;
    wProductType: u8;
    wReserved: u8;
}
```
### `OSVERSIONINFOEXW`

```jai
OSVERSIONINFOEXW :: struct {
    #as using osversioninfo: OSVERSIONINFOW;
    wServicePackMajor: u16;
    wServicePackMinor: u16;
    wSuiteMask: u16;
    wProductType: u8;
    wReserved: u8;
}
```
### `OSVERSIONINFOW`

```jai
OSVERSIONINFOW   :: struct {
    dwOSVersionInfoSize: u32;
    dwMajorVersion: u32;
    dwMinorVersion: u32;
    dwBuildNumber: u32;
    dwPlatformId: u32;
    szCSDVersion: [128]u16;
}
```
### `OVERLAPPED`

```jai
OVERLAPPED :: struct {

    Internal:                 u64;
    #overlay (Internal) Status: u64;

    InternalHigh:             u64;
    #overlay (InternalHigh) NumberOfBytesTransferred: u64;

    union {
        struct {
            Offset:     u32;
            OffsetHigh: u32;
        }

        Offset64: u64;
        Pointer: *void;
    }

    hEvent: HANDLE;
}
```
### `PIXELFORMATDESCRIPTOR`

```jai
PIXELFORMATDESCRIPTOR :: struct {
    nSize:  s16;
    nVersion:  s16;
    dwFlags:  s32;
    iPixelType:  u8;
    cColorBits:  u8;
    cRedBits:  u8;
    cRedShift:  u8;
    cGreenBits:  u8;
    cGreenShift:  u8;
    cBlueBits:  u8;
    cBlueShift:  u8;
    cAlphaBits:  u8;
    cAlphaShift:  u8;
    cAccumBits:  u8;
    cAccumRedBits:  u8;
    cAccumGreenBits:  u8;
    cAccumBlueBits:  u8;
    cAccumAlphaBits:  u8;
    cDepthBits:  u8;
    cStencilBits:  u8;
    cAuxBuffers:  u8;
    iLayerType:  u8;
    bReserved:  u8;
    dwLayerMask:  s32;
    dwVisibleMask:  s32;
    dwDamageMask:  s32;
}
```
### `POINT`

```jai
POINT :: struct {
    x, y: s32;
}
```
### `POINTL`

```jai
POINTL :: struct {
    x, y: s32;
}
```
### `PROCESSOR_CACHE_TYPE`

```jai
using PROCESSOR_CACHE_TYPE :: enum {
    CacheUnified;
    CacheInstruction;
    CacheData;
    CacheTrace;
}
```
### `PROCESSOR_GROUP_INFO`

```jai
PROCESSOR_GROUP_INFO :: struct {
    MaximumProcessorCount: u8;
    ActiveProcessorCount:  u8;
    Reserved:         [38] u8;
    ActiveProcessorMask:   u64;
}
```
### `PROCESSOR_RELATIONSHIP`

```jai
PROCESSOR_RELATIONSHIP :: struct {
    Flags:           u8;
    EfficiencyClass: u8;
    Reserved:   [20] u8;
    GroupCount:      u16;
    GroupMask: GROUP_AFFINITY;
}
```
### `PROCESS_DPI_AWARENESS`

```jai
using PROCESS_DPI_AWARENESS :: enum s32 {
    PROCESS_DPI_UNAWARE             :: 0;
    PROCESS_SYSTEM_DPI_AWARE        :: 1;
    PROCESS_PER_MONITOR_DPI_AWARE   :: 2;
}
```
### `PROCESS_INFORMATION`

```jai
PROCESS_INFORMATION :: struct {
    hProcess: HANDLE;
    hThread:  HANDLE;
    dwProcessId: u32;
    dwThreadId:  u32;
}
```
### `RAWHID`

```jai
RAWHID :: struct {
    dwSizeHid: u32;
    dwCount: u32;
    bRawData: [1]u8;
}
```
### `RAWINPUT`

```jai
RAWINPUT :: struct {
    header: RAWINPUTHEADER;

    data: union {
        mouse: RAWMOUSE;
        keyboard: RAWKEYBOARD;
        hid: RAWHID;
    };
}
```
### `RAWINPUTDEVICE`

```jai
RAWINPUTDEVICE :: struct {
    usUsagePage: u16;
    usUsage: u16;
    dwFlags: s32;
    hwndTarget: HWND;
}
```
### `RAWINPUTHEADER`

```jai
RAWINPUTHEADER :: struct {
    dwType:  u32;
    dwSize:  u32;
    hDevice: HANDLE;
    wParam:  WPARAM;
}
```
### `RAWKEYBOARD`

```jai
RAWKEYBOARD :: struct {
    MakeCode: u16;
    Flags: u16;
    Reserved: u16;
    VKey: u16;
    Message: u32;
    ExtraInformation: u32;
}
```
### `RAWMOUSE`

```jai
RAWMOUSE :: struct {
    usFlags: u16;

    union {
        ulButtons: u32;

        struct {
            usButtonFlags: u16;
            usButtonData:  u16;
        }
    }

    ulRawButtons: u32;
    lLastX: s32;
    lLastY: s32;
    ulExtraInformation: u32;
}
```
### `RECT`

```jai
RECT :: struct {
    left, top, right, bottom: s32;
}
```
### `RGBQUAD`

```jai
RGBQUAD :: struct {
    rgbBlue: u8;
    rgbGreen: u8;
    rgbRed: u8;
    rgbReserved: u8;
}
```
### `ROP`

```jai
ROP :: enum u32 {
    SRCCOPY :: 0x00CC0020;
}
```
### `RTL_CRITICAL_SECTION`

```jai
RTL_CRITICAL_SECTION :: struct {
    DebugInfo : *RTL_CRITICAL_SECTION_DEBUG;
    LockCount : u32;
    RecursionCount : u32;
    OwningThread : HANDLE;
    LockSemaphore : HANDLE;
    SpinCount : u64;
}
```
### `RTL_CRITICAL_SECTION_DEBUG`

```jai
RTL_CRITICAL_SECTION_DEBUG :: struct {
    Type: u16;
    CreatorBackTraceIndex: u16;
    CriticalSection: * RTL_CRITICAL_SECTION;
    ProcessLocksList: LIST_ENTRY;
    EntryCount: u32;
    ContentionCount: u32;
    Flags: u32;
    CreatorBackTraceIndexHigh: u16;
    SpareWORD: u16;
}
```
### `SECURITY_ATTRIBUTES`

```jai
SECURITY_ATTRIBUTES :: struct {
    nLength: u32;
    lpSecurityDescriptor: *void;
    bInheritHandle: BOOL;
}
```
### `SEM`

```jai
using SEM :: enum_flags u32 {
    SEM_FAILCRITICALERRORS :: 0x0001;
    SEM_NOALIGNMENTFAULTEXCEPT :: 0x0004;
    SEM_NOGPFAULTERRORBOX :: 0x0002;
    SEM_NOOPENFILEERRORBOX :: 0x8000;
}
```
### `SHFILEOPSTRUCTA`

```jai
SHFILEOPSTRUCTA :: struct {
    hwnd:   HWND;
    wFunc:  u32;
    pFrom: *u8;
    pTo:   *u8;
    fFlags: FILEOP_FLAGS;
    fAnyOperationsAborted: BOOL;
    hNameMappings:     *void;
    lpszProgressTitle: *u8;
}
```
### `SHFILEOPSTRUCTW`

```jai
SHFILEOPSTRUCTW :: struct {
    hwnd:   HWND;
    wFunc:  u32;
    pFrom: *u16;
    pTo:   *u16;
    fFlags: FILEOP_FLAGS;
    fAnyOperationsAborted: BOOL;
    hNameMappings:     *void;
    lpszProgressTitle: *u16;
}
```
### `SIZE`

```jai
SIZE :: struct {
    cx: s32;
    cy: s32;
}
```
### `SMALL_RECT`

```jai
SMALL_RECT :: struct {
    left, top, right, bottom: s16;
}
```
### `SP_DEVICE_INTERFACE_DATA`

```jai
SP_DEVICE_INTERFACE_DATA :: struct {
    cbSize: u32;
    InterfaceClassGuid: GUID;
    Flags: u32;
    Reserved: *void;
}
```
### `SP_DEVICE_INTERFACE_DETAIL_DATA_W`

```jai
SP_DEVICE_INTERFACE_DETAIL_DATA_W :: struct {
    cbSize: u32;
    DevicePath: [1] u16;
}
```
### `SP_DEVINFO_DATA`

```jai
SP_DEVINFO_DATA :: struct {
    cbSize: u32;
    ClassGuid: GUID;
    DevInst: u32;
    Reserved: *void;
}
```
### `STACKFRAME64`

```jai
STACKFRAME64 :: struct {
    AddrPC: ADDRESS64;
    AddrReturn: ADDRESS64;
    AddrFrame: ADDRESS64;
    AddrStack: ADDRESS64;
    AddrBStore: ADDRESS64;
    FuncTableEntry: *void;
    Params: [4]u64;
    Far: BOOL;
    Virtual: BOOL;
    Reserved: [3]u64;
    KdHelp: KDHELP64;
}
```
### `STARTUPINFOA`

```jai
STARTUPINFOA :: struct {
    cb: s32;
    lpReserved: *u8;
    lpDesktop: *u8;
    lpTitle: *u8;
    dwX: s32;
    dwY: s32;
    dwXSize: s32;
    dwYSize: s32;
    dwXCountChars: s32;
    dwYCountChars: s32;
    dwFillAttribute: s32;
    dwFlags: s32;
    wShowWindow: s16;
    cbReserved2: s16;
    lpReserved2: *u8;
    hStdInput: HANDLE;
    hStdOutput: HANDLE;
    hStdError: HANDLE;
}
```
### `STARTUPINFOW`

```jai
STARTUPINFOW :: struct {
    cb: s32;
    lpReserved: *u16;
    lpDesktop: *u16;
    lpTitle: *u16;
    dwX: s32;
    dwY: s32;
    dwXSize: s32;
    dwYSize: s32;
    dwXCountChars: s32;
    dwYCountChars: s32;
    dwFillAttribute: s32;
    dwFlags: s32;
    wShowWindow: s16;
    cbReserved2: s16;
    lpReserved2: *u8;
    hStdInput: HANDLE;
    hStdOutput: HANDLE;
    hStdError: HANDLE;
}
```
### `STICKYKEYS`

```jai
STICKYKEYS :: struct {
    cbSize: u32 = 8;
    dwFlags: STICKYKEY_FLAGS;
}
```
### `STICKYKEY_FLAGS`

```jai
using STICKYKEY_FLAGS :: enum_flags u32 {
    SKF_STICKYKEYSON ::    0x00000001;
    SKF_AVAILABLE ::       0x00000002;
    SKF_HOTKEYACTIVE ::    0x00000004;
    SKF_CONFIRMHOTKEY ::   0x00000008;
    SKF_HOTKEYSOUND ::     0x00000010;
    SKF_INDICATOR ::       0x00000020;
    SKF_AUDIBLEFEEDBACK :: 0x00000040;
    SKF_TRISTATE ::        0x00000080;
    SKF_TWOKEYSOFF ::      0x00000100;
}
```
### `SYMBOL_INFO`

```jai
SYMBOL_INFO :: struct {
    SizeOfStruct : u32;
    TypeIndex : u32;
    Reserved : [2]u64;
    Index : u32;
    Size : u32;
    ModBase : u64;
    Flags : u32;
    Value : u64;
    Address : u64;
    Register : u32;
    Scope : u32;
    Tag : u32;
    NameLen : u32;
    MaxNameLen : u32;
    Name : [1]u8;
}
```
### `SYMOPT`

```jai
SYMOPT :: enum_flags u32 {
    CASE_INSENSITIVE ::          0x00000001;
    UNDNAME ::                   0x00000002;
    DEFERRED_LOADS ::            0x00000004;
    NO_CPP ::                    0x00000008;
    LOAD_LINES ::                0x00000010;
    OMAP_FIND_NEAREST ::         0x00000020;
    LOAD_ANYTHING ::             0x00000040;
    IGNORE_CVREC ::              0x00000080;
    NO_UNQUALIFIED_LOADS ::      0x00000100;
    FAIL_CRITICAL_ERRORS ::      0x00000200;
    EXACT_SYMBOLS ::             0x00000400;
    ALLOW_ABSOLUTE_SYMBOLS ::    0x00000800;
    IGNORE_NT_SYMPATH ::         0x00001000;
    INCLUDE_32BIT_MODULES ::     0x00002000;
    PUBLICS_ONLY ::              0x00004000;
    NO_PUBLICS ::                0x00008000;
    AUTO_PUBLICS ::              0x00010000;
    NO_IMAGE_SEARCH ::           0x00020000;
    SECURE ::                    0x00040000;
    NO_PROMPTS ::                0x00080000;
    OVERWRITE ::                 0x00100000;
    IGNORE_IMAGEDIR ::           0x00200000;
    FLAT_DIRECTORY ::            0x00400000;
    FAVOR_COMPRESSED ::          0x00800000;
    ALLOW_ZERO_ADDRESS ::        0x01000000;
    DISABLE_SYMSRV_AUTODETECT :: 0x02000000;
    READONLY_CACHE ::            0x04000000;
    SYMPATH_LAST ::              0x08000000;
    DISABLE_FAST_SYMBOLS ::      0x10000000;
    DISABLE_SYMSRV_TIMEOUT ::    0x20000000;
    DISABLE_SRVSTAR_ON_STARTUP :: 0x40000000;
    DEBUG ::                     0x80000000;
}
```
### `SYSTEMTIME`

```jai
SYSTEMTIME :: struct {
    wYear         : u16;
    wMonth        : u16;
    wDayOfWeek    : u16;
    wDay          : u16;
    wHour         : u16;
    wMinute       : u16;
    wSecond       : u16;
    wMilliseconds : u16;
}
```
### `SYSTEM_CPU_SET_INFORMATION`

```jai
SYSTEM_CPU_SET_INFORMATION :: struct {
    Size: u32;
    Type: enum u32 { CpuSetInformation; };
    union {
        CpuSet: struct {
            Id:                    u32;
            Group:                 u16;
            LogicalProcessorIndex: u8;
            CoreIndex:             u8;
            LastLevelCacheIndex:   u8;
            NumaNodeIndex:         u8;
            EfficiencyClass:       u8;
            union {
                using AllFlags: enum_flags u8 {
                    Parked                   :: 0x1;
                    Allocated                :: 0x2;
                    AllocatedToTargetProcess :: 0x4;
                    RealTime                 :: 0x8;
                    ReservedFlags            :: 0xF0;
                }
            }
            union {
                Reserved: u32;
                SchedulingClass: u8;
            }
            AllocationTag: u64;
        }
    }
}
```
### `SYSTEM_INFO`

```jai
SYSTEM_INFO :: struct {

    wProcessorArchitecture      : u16;
    wReserved                   : u16;

    dwPageSize                  : u32;
    lpMinimumApplicationAddress : *void;
    lpMaximumApplicationAddress : *void;
    dwActiveProcessorMask       : u64;
    dwNumberOfProcessors        : u32;
    dwProcessorType             : u32;
    dwAllocationGranularity     : u32;
    wProcessorLevel             : u16;
    wProcessorRevision          : u16;
}
```
### `SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX`

```jai
SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX :: struct {
    Relationship: LOGICAL_PROCESSOR_RELATIONSHIP;
    Size: u32;
    union {
        Processor: PROCESSOR_RELATIONSHIP;
        NumaNode:  NUMA_NODE_RELATIONSHIP;
        Cache:     CACHE_RELATIONSHIP;
        Group:     GROUP_RELATIONSHIP;
    }
}
```
### `SetFilePointer_MoveMethod`

```jai
using SetFilePointer_MoveMethod :: enum u32 {
    FILE_BEGIN       :: 0;
    FILE_CURRENT     :: 1;
    FILE_END         :: 2;
}
```
### `TIMECAPS`

```jai
TIMECAPS :: struct {
    wPeriodMin: u32;
    wPeriodMax: u32;
}
```
### `TIME_ZONE_INFORMATION`

```jai
TIME_ZONE_INFORMATION :: struct {
    Bias: s32;
    StandardName: [32] u16;
    StandardDate: SYSTEMTIME;
    StandardBias: s32;
    DaylightName: [32] u16;
    DaylightDate: SYSTEMTIME;
    DaylightBias: s32;
}
```

The seven members above follow the public Windows SDK `TIME_ZONE_INFORMATION`
declaration in `timezoneapi.h`: SDK `LONG` is signed 32-bit and SDK `WCHAR` is
unsigned 16-bit. Existing facade spellings are `s32` and `u16`; no public `LONG`
or `WCHAR` alias is defined here. Both embedded dates use the existing
canonical `SYSTEMTIME`. Ordinary member layout produces size 172/alignment 4;
there is no packing override, explicit padding, or opaque size substitution.
See the [Windows SDK declaration](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information).

### `TOGGLEKEYS`

```jai
TOGGLEKEYS :: struct {
    cbSize: u32 = 8;
    dwFlags: TOGGLEKEY_FLAGS;
}
```
### `TOGGLEKEY_FLAGS`

```jai
using TOGGLEKEY_FLAGS :: enum_flags u32 {
    TKF_TOGGLEKEYSON ::    0x00000001;
    TKF_AVAILABLE ::       0x00000002;
    TKF_HOTKEYACTIVE ::    0x00000004;
    TKF_CONFIRMHOTKEY ::   0x00000008;
    TKF_HOTKEYSOUND ::     0x00000010;
    TKF_INDICATOR ::       0x00000020;
}
```
### `TOUCHINPUT`

```jai
TOUCHINPUT :: struct {
    x: s32;
    y: s32;
    hSource: HANDLE;
    dwID: DWORD;
    dwFlags: DWORD;
    dwMask: DWORD;
    dwTime: DWORD;
    dwExtraInfo: u64;
    cxContact: DWORD;
    cyContact: DWORD;
}
```
### `ULARGE_INTEGER`

```jai
ULARGE_INTEGER :: union {
    struct {
        LowPart : u32;
        HighPart : u32;
    }

    QuadPart: u64;
}
```
### `WIN32_FIND_DATAW`

```jai
WIN32_FIND_DATAW :: struct {
    dwFileAttributes: s32;

    ftCreationTime:   FILETIME;
    ftLastAccessTime: FILETIME;
    ftLastWriteTime:  FILETIME;

    nFileSizeHigh:    DWORD;
    nFileSizeLow:     DWORD;

    dwReserved0:      DWORD;
    dwReserved1:      DWORD;

    cFileName:        [MAX_PATH] u16;

    cAlternateFileName: [14] u16;
}
```
### `WNDCLASSEXA`

```jai
WNDCLASSEXA :: struct {}
```
### `WNDCLASSEXW`

```jai
WNDCLASSEXW :: struct {
    cbSize: u32;
    style: u32;
    lpfnWndProc: (hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> s64 #foreign;
    cbClsExtra: s32;
    cbWndExtra: s32;
    hInstance: HINSTANCE;
    hIcon: HICON;
    hCursor: HCURSOR;
    hbrBackground: HBRUSH;
    lpszMenuName: *u16;
    lpszClassName: *u16;
    hIconSm: HICON;
}
```
### `XSAVE_FORMAT`

```jai
XSAVE_FORMAT :: struct {
    ControlWord: u16;
    StatusWord: u16;
    TagWord: u8;
    Reserved1: u8;
    ErrorOpcode: u16;
    ErrorOffset: u32;
    ErrorSelector: u16;
    Reserved2: u16;
    DataOffset: u32;
    DataSelector: u16;
    Reserved3: u16;
    MxCsr: u32;
    MxCsr_Mask: u32;
    FloatRegisters: [8]M128A;

    #if 1  {
        XmmRegisters: [16]M128A;
        Reserved4: [96]u8;
    }
    else {
        XmmRegisters: [8]M128A;
        Reserved4: [224]u8;
    }
}
```
## Contracts and constraints

This module is a broad Windows SDK binding and utility layer. It exposes native handles, constants, structures, and calls; callers must follow each Win32 API’s status conventions, ownership rules, and required library initialization. Types and layouts are Windows ABI declarations and are not portable; optional system libraries and API availability depend on the Windows version and SDK represented by this binding.

`M128A` and AMD64 `CONTEXT` require 16-byte alignment in the Microsoft SDK ABI. Their Jai declarations therefore retain `#align 16`; this is required even though the default alignment of their member types may be smaller. The Windows x64 SDK header uses `DECLSPEC_ALIGN(16)` on both records ([pinned Microsoft `winnt.h`](https://github.com/microsoft/win32metadata/blob/0f012441d654d3bb7de898fdc664b5fcad52bdcc/generation/WinSDK/RecompiledIdlHeaders/um/winnt.h)).

The published `CONTEXT` shape in this surface is the AMD64 layout: its members are the `P1Home`–`P6Home`, segment/debug/general-purpose registers, `Rip`, and floating-point/vector state shown above. There is no ARM64 `CONTEXT` alternative here, and the `CONTEXT_AMD64` flag constants do not establish ARM64 support. Procedures accepting `*CONTEXT` therefore have an architecture-sensitive contract; the ARM64 layout is not specified.
