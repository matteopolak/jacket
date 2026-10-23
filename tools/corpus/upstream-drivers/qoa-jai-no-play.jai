#import "Basic";
#import "File";
#import,file "666rayen999--qoa-jai/qoa.jai"(QOA_PLAY = false);

main :: () {
    original, ok := read_entire_file("original.qoa");
    if !ok return;

    decoded:, desc:, ok = Qoa.decode(original);

    if !ok return;

    encoded:, ok = Qoa.encode(decoded, *desc);
    if ok write_entire_file("out.qoa", encoded);
}
