#import "Basic";

main :: () {
    volume := 140;
    volume = min(100, max(0, volume));
    print("%\n", volume);
}
