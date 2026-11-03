#import "Basic";

main :: () {
    volume := 140;
    volume = min(0, max(100, volume));
    print("%\n", volume);
}
