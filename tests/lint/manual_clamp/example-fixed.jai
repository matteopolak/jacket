#import "Basic";

main :: () {
    volume := 140;
    volume = clamp(volume, 0, 100);
    print("%\n", volume);
}
