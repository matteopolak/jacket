// Path helpers from String: file names, directories and extensions.
#import "Basic";
#import "String";

main :: () {
    checksum: u64 = 1469598103934665603;

    mix :: (value: int) #expand {
        checksum = (checksum ^ cast,no_check(u64) value) * 1099511628211;
    }

    paths := string.["/usr/local/lib/libthing.so.1", "relative/dir/file.tar.gz", "C:/Program Files/app/run.exe", "noext", "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p.txt"];

    for 0..16_000_000 - 1 {
        path := paths[it % paths.count];

        mix(path_filename(path).count);
        mix(path_strip_filename(path).count);
        extension, found := path_extension(path);

        mix(extension.count + cast(int) found);
        mix(path_strip_extension(path).count);
    }

    print("%\n", checksum);
}
