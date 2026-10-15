/* Original record using the public DEFRANGESYMREGISTERREL flag widths. */
struct Original_Register_Flags {
    unsigned short spilled_member : 1;
    unsigned short reserved : 3;
    unsigned short parent_offset : 12;
};
_Static_assert(sizeof(struct Original_Register_Flags) == 2, "CodeView register flags");
struct Original_Register_Flags original_flags[] = {
    {0, 0, 0}, {1, 0, 0}, {0, 7, 0}, {1, 7, 0},
    {0, 5, 1}, {1, 5, 0xabc}, {0, 2, 0xfff}, {1, 7, 0xfff}
};
