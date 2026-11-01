// Shared, pure DPI rules. Loaded privately by Window_Creation and its regression test.
#scope_module

// Preserve fractional desktop scales (120 DPI is 1.25), reject implausible values,
// and never shrink below 100%. Physical-screen estimates alone round to whole scales.
dpi_scale :: (dpi: float, physical_estimate := false) -> float {
    if !(dpi >= 48 && dpi <= 768) return 1;

    scale := dpi / 96;
    if scale < 1 return 1;
    if physical_estimate return cast(float) cast(s32) (scale + 0.5);

    return scale;
}

// Only the exact resource name is accepted; comments, unrelated resources and malformed
// values do not turn into a scale. A later valid setting takes precedence.
xft_dpi :: (resources: string) -> float {
    result: float;
    start := 0;

    for end: 0..resources.count {
        if end < resources.count && resources[end] != #char "\n" continue;

        line := dpi_trim(string.{ end - start, resources.data + start });

        start = end + 1;
        if line.count < 8 continue;

        name := string.{ 7, line.data };
        if name != "Xft.dpi" continue;

        rest := dpi_trim(string.{ line.count - 7, line.data + 7 });
        if !rest.count || rest[0] != #char ":" continue;

        value := dpi_trim(string.{ rest.count - 1, rest.data + 1 });
        number: float;
        divisor: float = 1;
        decimal := false;
        valid := value.count > 0;
        digits := 0;

        for value {
            if it == #char "." && !decimal {
                decimal = true;
                continue;
            }

            if it < #char "0" || it > #char "9" {
                valid = false;
                break;
            }

            digits += 1;
            if decimal divisor *= 10;

            number = number * 10 + cast(float) (it - #char "0");
        }

        number /= divisor;
        if valid && digits > 0 && number >= 48 && number <= 768 result = number;
    }

    return result;
}

dpi_trim :: (text: string) -> string {
    result := text;
    while result.count && (result[0] == #char " " || result[0] == #char "\t" || result[0] == #char "\r") {
        result.data += 1;
        result.count -= 1;
    }

    while result.count && (result[result.count - 1] == #char " " || result[result.count - 1] == #char "\t" || result[result.count - 1] == #char "\r") result.count -= 1;

    return result;
}
