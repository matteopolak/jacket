#import "Basic";
#load "../../internal/window/dpi.jai";

main :: () {
    assert(dpi_scale(96) == 1);
    assert(dpi_scale(120) == 1.25);
    assert(dpi_scale(144) == 1.5);
    assert(dpi_scale(192) == 2);
    assert(dpi_scale(72) == 1);
    assert(dpi_scale(0) == 1 && dpi_scale(-1) == 1 && dpi_scale(99999) == 1);
    assert(dpi_scale(143, true) == 1 && dpi_scale(144, true) == 2);
    assert(xft_dpi("Xft.dpi: 192\n") == 192);
    assert(xft_dpi("! Xft.dpi: 200\nOther: 5\n\tXft.dpi : 144.5\r\n") == 144.5);
    assert(xft_dpi("Xft.dpi: 96\nXft.dpi: 192") == 192);
    for string.["", "Xft.dpixels: 192", "Xft.dpi: nope", "Xft.dpi: 0", "Xft.dpi: -192", "Xft.dpi: 192junk", "Xft.dpi: .", "Xft.dpi: 1.2.3", "Xft.dpi: 999999"] assert(xft_dpi(it) == 0);

    print("ok\n");
}
