#add_context late_value: int = 31;
#add_context late_text: string = "late";

read_late :: () -> int {
    return context.late_value;
}

write_late :: (value: int) {
    context.late_value = value;
}

text_late :: () -> string {
    return context.late_text;
}
