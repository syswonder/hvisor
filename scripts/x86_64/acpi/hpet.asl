/*
* HPET template
*/
[0004] Signature : "HPET"
[0004] Table Length : 00000000
[0001] Revision : 01
[0001] Checksum : 00
[0006] Oem ID : "DM "
[0008] Oem Table ID : "DMHPET  "
[0004] Oem Revision : 00000001
/* iasl will fill in the compiler ID/revision fields */
[0004] Asl Compiler ID : "xxxx"
[0004] Asl Compiler Revision : 00000000

/* 
[31:16] = PCI Vendor ID of 1st Timer Block (0x8086)
[15] = LegacyReplacement IRQ Routing Capable (0)
[14] = Reserved (0)
[13] = COUNT_SIZE_CAP counter size (32-bit=0)
[12:8] = Number of Comparators in 1st Timer Block (3-1=2)
[7:0] = Hardware Rev ID (1)
*/
[0004] Hardware Block ID : 80860201

[0012] Timer Block Register : [Generic Address Structure]
    [0001] Space ID : 00 [SystemMemory]
    [0001] Bit Width : 00
    [0001] Bit Offset : 00
    [0001] Encoded Access Width : 00 [Undefined/Legacy]
    [0008] Address : 00000000fed00000

[0001] Sequence Number : 00
[0002] Minimum Clock Ticks : 0000
[0004] Flags (decoded below) : 00000001
    4K Page Protect : 1
    64K Page Protect : 0