// Self-authored contract for the six reviewed VMA 3.3 virtual-allocation symbols.
VMA :: #library,no_dll "reviewed-vma-virtual";
Block_Info :: struct { size:u64; flags:u32; allocation_callbacks:*void; }
Allocation_Info :: struct { size:u64; alignment:u64; flags:u32; user_data:*void; }
Info :: struct { offset:u64; size:u64; user_data:*void; }
vmaCreateVirtualBlock :: (info:*Block_Info, result:**void)->s32 #foreign VMA;
vmaDestroyVirtualBlock :: (block:*void) #foreign VMA;
vmaVirtualAllocate :: (block:*void, info:*Allocation_Info, result:**void, offset:*u64)->s32 #foreign VMA;
vmaVirtualFree :: (block:*void, allocation:*void) #foreign VMA;
vmaGetVirtualAllocationInfo :: (block:*void, allocation:*void, info:*Info) #foreign VMA;
vmaIsVirtualBlockEmpty :: (block:*void)->u32 #foreign VMA;
main :: ()->int {
    block_info:Block_Info;
    block_info.size=4096;
    block:*void;
    if vmaCreateVirtualBlock(*block_info,*block)!=0 { return 11; }
    token:int=7;
    allocation_info:Allocation_Info;
    allocation_info.size=42;
    allocation_info.alignment=16;
    allocation_info.user_data=cast(*void)*token;
    allocation:*void;
    offset:u64;
    if vmaVirtualAllocate(block,*allocation_info,*allocation,*offset)!=0 { return 12; }
    info:Info;
    vmaGetVirtualAllocationInfo(block,allocation,*info);
    result:=cast(int)info.size;
    if info.offset!=offset || offset%16!=0 || info.user_data!=cast(*void)*token { result=13; }
    if vmaIsVirtualBlockEmpty(block)!=0 { result=14; }
    vmaVirtualFree(block,allocation);
    if vmaIsVirtualBlockEmpty(block)!=1 { result=15; }
    vmaDestroyVirtualBlock(block);
    return result;
}
