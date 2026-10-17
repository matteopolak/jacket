int original_absolute_frame(int value) { return value + 9; }
int original_default_frame(int value) { return value - 4; }

/* Independently authored standard .eh_frame records. The native assembler and
   linker resolve real C function symbols; no compiler-private labels are used. */
__asm__(
    ".pushsection .eh_frame,\"a\",@progbits\n"
    ".p2align 3\n"
    ".Loriginal_cie_abs:\n"
    ".long .Loriginal_cie_abs_end-.Loriginal_cie_abs_payload\n"
    ".Loriginal_cie_abs_payload:\n"
    ".long 0\n.byte 1\n.asciz \"zR\"\n"
    ".uleb128 1\n.sleb128 -8\n.byte 16\n.uleb128 1\n.byte 0\n"
    ".byte 0x0c,7,8,0x90,1\n.p2align 3\n"
    ".Loriginal_cie_abs_end:\n"
    ".long .Loriginal_fde_abs_end-.Loriginal_fde_abs_payload\n"
    ".Loriginal_fde_abs_payload:\n"
    ".long .-.Loriginal_cie_abs\n"
    ".quad original_absolute_frame\n.quad 1\n.uleb128 0\n"
    ".p2align 3\n.Loriginal_fde_abs_end:\n"
    ".Loriginal_cie_default:\n"
    ".long .Loriginal_cie_default_end-.Loriginal_cie_default_payload\n"
    ".Loriginal_cie_default_payload:\n"
    ".long 0\n.byte 1\n.asciz \"\"\n"
    ".uleb128 1\n.sleb128 -8\n.byte 16\n"
    ".byte 0x0c,7,8,0x90,1\n.p2align 3\n"
    ".Loriginal_cie_default_end:\n"
    ".long .Loriginal_fde_default_end-.Loriginal_fde_default_payload\n"
    ".Loriginal_fde_default_payload:\n"
    ".long .-.Loriginal_cie_default\n"
    ".quad original_default_frame\n.quad 1\n"
    ".p2align 3\n.Loriginal_fde_default_end:\n.popsection\n"
);
