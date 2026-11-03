#import "Basic";

main :: () {
    volume := 140;
    volume = max(0, min(100, volume));
    print("%\n", volume);
}
