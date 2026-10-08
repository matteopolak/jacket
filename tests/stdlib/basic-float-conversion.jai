// Prepared actual Basic fixture. It must execute through the real module.
#import "Basic";

main :: () -> s64 {
    if tprint("%", formatFloat(1234567.89, mode = .SHORTEST)) != "1.23457e+06" return 1;
    if tprint("%", formatFloat(2.5, trailing_width = 0)) != "3" return 2;
    if tprint("%", formatFloat(0.125, trailing_width = 2, zero_removal = .NO)) != "0.13" return 3;
    if tprint("%", formatFloat(9.999, trailing_width = 2, mode = .SCIENTIFIC, zero_removal = .NO)) != "1.00e+01" return 4;
    infinity := cast,force(float64) cast(u64) 0x7ff0_0000_0000_0000;
    nan := cast,force(float64) cast(u64) 0x7ff8_0000_0000_0001;
    if tprint("%", formatFloat(infinity)) != "Inf" || tprint("%", formatFloat(-infinity)) != "-Inf" || tprint("%", formatFloat(nan)) != "NaN" return 5;
    if formatFloatRoundtrip(-0.0) != "-0" return 6;
    minimum := cast,force(float64) cast(u64) 1;
    if formatFloatRoundtrip(minimum) != "5e-324" return 7;
    if formatFloatRoundtrip(0.1) != "0.1" return 8;
    next_one := cast,force(float64) cast(u64) 0x3ff0_0000_0000_0001;
    if formatFloatRoundtrip(next_one) != "1.0000000000000002" return 9;
    maximum := cast,force(float64) cast(u64) 0x7fef_ffff_ffff_ffff;
    if formatFloatRoundtrip(maximum) != "1.7976931348623157e+308" return 10;
    return 0;
}
