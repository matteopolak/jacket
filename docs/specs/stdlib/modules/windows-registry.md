# Windows_Registry


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- [OS == .WINDOWS] `KEY_QUERY_VALUE :: 0x1`
- [OS == .WINDOWS] `KEY_SET_VALUE :: 0x2`
- [OS == .WINDOWS] `KEY_CREATE_SUB_KEY :: 0x4`
- [OS == .WINDOWS] `KEY_ENUMERATE_SUB_KEYS :: 0x8`
- [OS == .WINDOWS] `KEY_NOTIFY :: 0x0010`
- [OS == .WINDOWS] `KEY_WOW64_64KEY :: 0x0100`
- [OS == .WINDOWS] `KEY_WOW64_32KEY :: 0x0200`
- [OS == .WINDOWS] `KEY_WRITE :: 0x20006`
- [OS == .WINDOWS] `KEY_READ :: 0x20019`
- [OS == .WINDOWS] `KEY_EXECUTE :: KEY_READ`
- [OS == .WINDOWS] `REGSAM :: u32`
- [OS == .WINDOWS] `LSTATUS :: s32`
- [OS == .WINDOWS] `HKEY_CLASSES_ROOT :: cast(HKEY) 0x80000000`
- [OS == .WINDOWS] `HKEY_CURRENT_USER :: cast(HKEY) 0x80000001`
- [OS == .WINDOWS] `HKEY_LOCAL_MACHINE :: cast(HKEY) 0x80000002`
- [OS == .WINDOWS] `HKEY_USERS :: cast(HKEY) 0x80000003`
- [OS == .WINDOWS] `HKEY_PERFORMANCE_DATA :: cast(HKEY) 0x80000004`
- [OS == .WINDOWS] `HKEY_PERFORMANCE_TEXT :: cast(HKEY) 0x80000050`
- [OS == .WINDOWS] `HKEY_PERFORMANCE_NLSTEXT :: cast(HKEY) 0x80000060`
- [OS == .WINDOWS] `HKEY_CURRENT_CONFIG :: cast(HKEY) 0x80000005`
- [OS == .WINDOWS] `HKEY_DYN_DATA :: cast(HKEY) 0x80000006`
- [OS == .WINDOWS] `HKEY_CURRENT_USER_LOCAL_SETTINGS :: cast(HKEY) 0x80000007`
- [OS == .WINDOWS] `RRF_RT_REG_SZ :: 0x00000002`
- [OS == .WINDOWS] `RRF_RT_REG_DWORD :: 0x00000010`
- [OS == .WINDOWS] `RRF_RT_DWORD :: 0x00000018`
- [OS == .WINDOWS] `RRF_RT_QWORD :: 0x00000048`
- [OS == .WINDOWS] `RegOpenKeyExA :: (hKey: HKEY, lpSubKey: *u8, ulOptions: DWORD, samDesired: REGSAM, phkResult: *HKEY) -> LSTATUS #foreign advapi`
- [OS == .WINDOWS] `RegQueryValueExA :: (hkey: HKEY, lpValueName: *u8, lpReserved: *DWORD, lpType: *DWORD, lpData: *u8, lpcbData: *DWORD) -> LSTATUS #foreign advapi`
- [OS == .WINDOWS] `RegOpenKeyExW :: (hKey: HKEY, lpSubKey: *u16, ulOptions: DWORD, samDesired: REGSAM, phkResult: *HKEY) -> LSTATUS #foreign advapi`
- [OS == .WINDOWS] `RegQueryValueExW :: (hkey: HKEY, lpValueName: *u16, lpReserved: *DWORD, lpType: *DWORD, lpData: *u8, lpcbData: *DWORD) -> LSTATUS #foreign advapi`
- [OS == .WINDOWS] `RegCloseKey :: (hKey: HKEY) -> LSTATUS #foreign advapi`
- [OS == .WINDOWS] `RegEnumKeyExA :: (hKey: HKEY, dwIndex: DWORD, lpName: *u8, lpcchName: *DWORD, lpReserved: *DWORD, lpClass: *u8, lpcchClass: *DWORD, lpftLastWriteTime: *FILETIME) -> LSTATUS #foreign advapi`
- [OS == .WINDOWS] `RegGetValueA :: (hkey: HKEY, lpSubKey: *u8, lpValue: *u8, dwFlags: DWORD, pdwType: *DWORD, pvData: *void, pcbData: *DWORD) -> LSTATUS #foreign advapi`


## Contracts and constraints

This module provides ANSI and wide-character Windows Registry API bindings and constants. Calls return `LSTATUS`; callers must inspect it, manage opened `HKEY` handles with `RegCloseKey`, and supply correctly sized buffers (query calls use in/out byte or character counts). The declarations target Advapi32 and Windows ABI types; this is a low-level binding layer with no automatic string conversion or buffer resizing.

The ten `HKEY_*` predefined values above are each explicitly formed by casting a positive `0x800000xx` integer literal directly to `HKEY`; there is no intermediate signed 32-bit cast. On 64-bit Jai targets this denotes the zero-extended pointer-sized value (for example, `HKEY_CLASSES_ROOT` is `0x0000000080000000`). The Microsoft SDK's public `winreg.h` macros instead cast through signed `LONG` before `ULONG_PTR`, yielding `0xffffffff80000000` on 64-bit targets. That is a source/API discrepancy: retain the values spelled by the Jai declarations rather than silently replacing them with the SDK macro values. On 32-bit targets their low 32-bit values coincide. SDK definition: [Microsoft winreg.h](https://raw.githubusercontent.com/microsoft/win32metadata/main/generation/WinSDK/RecompiledIdlHeaders/um/winreg.h).
