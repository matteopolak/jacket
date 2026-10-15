// Reviewed Windows-only exports; portable import and private facade isolation.
#import "Basic";
R :: #import "Windows_Registry";

#assert !#exists(R.ABI) && !#exists(R.W) && !#exists(R.advapi);

#if OS == .WINDOWS {
    #assert size_of(R.LSTATUS) == 4 && size_of(R.REGSAM) == 4;
    #assert R.KEY_READ == 0x20019 && R.KEY_EXECUTE == R.KEY_READ;
} else {
    #assert !#exists(R.RegOpenKeyExA) && !#exists(R.RegOpenKeyExW);
    #assert !#exists(R.KEY_READ) && !#exists(R.REGSAM);
}

main :: () {
    print("ok\n");
}
