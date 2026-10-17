// Exact reviewed scalar/type ABI; no registry access in compile-time checking.
R :: #import "Windows_Registry";
W :: #import "Windows";

#assert OS == .WINDOWS;
#assert size_of(R.REGSAM) == 4 && size_of(R.LSTATUS) == 4;
#assert R.KEY_QUERY_VALUE == 1 && R.KEY_SET_VALUE == 2 && R.KEY_CREATE_SUB_KEY == 4;
#assert R.KEY_ENUMERATE_SUB_KEYS == 8 && R.KEY_NOTIFY == 16;
#assert R.KEY_WOW64_64KEY == 256 && R.KEY_WOW64_32KEY == 512;
#assert R.KEY_READ == 0x20019 && R.KEY_WRITE == 0x20006 && R.KEY_EXECUTE == R.KEY_READ;
#assert R.RRF_RT_REG_SZ == 2 && R.RRF_RT_REG_DWORD == 16;
#assert R.RRF_RT_DWORD == 24 && R.RRF_RT_QWORD == 72;
#assert !#exists(R.ABI) && !#exists(R.W) && !#exists(R.advapi);
#assert type_of(R.HKEY_CURRENT_USER) == W.HKEY;
#assert cast(u64) R.HKEY_CLASSES_ROOT == 0x80000000;
#assert cast(u64) R.HKEY_CURRENT_USER == 0x80000001;
#assert cast(u64) R.HKEY_LOCAL_MACHINE == 0x80000002;
#assert cast(u64) R.HKEY_USERS == 0x80000003;
#assert cast(u64) R.HKEY_PERFORMANCE_DATA == 0x80000004;
#assert cast(u64) R.HKEY_CURRENT_CONFIG == 0x80000005;
#assert cast(u64) R.HKEY_DYN_DATA == 0x80000006;
#assert cast(u64) R.HKEY_CURRENT_USER_LOCAL_SETTINGS == 0x80000007;
#assert cast(u64) R.HKEY_PERFORMANCE_TEXT == 0x80000050;
#assert cast(u64) R.HKEY_PERFORMANCE_NLSTEXT == 0x80000060;

main :: () {}
