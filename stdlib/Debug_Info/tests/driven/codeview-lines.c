/* Original ABI fixture using the public CV_Line_t bit widths from Microsoft's
 * cvinfo.h. Windows unsigned long is 32 bits on both target ABIs below. */
struct Original_Codeview_Line {
    unsigned long offset;
    unsigned long first_line : 24;
    unsigned long end_delta : 7;
    unsigned long statement : 1;
};
_Static_assert(sizeof(struct Original_Codeview_Line) == 8, "CodeView line ABI");
struct Original_Codeview_Line original_line = {0x12345678, 0x654321, 0x35, 1};
