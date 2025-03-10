/*
* MADT template
*/
[0004] Signature : "APIC"
[0004] Table Length : 00000000
[0001] Revision : 01
[0001] Checksum : 00
[0006] Oem ID : "DM "
[0008] Oem Table ID : "DMMADT  "
[0004] Oem Revision : 00000001
/* iasl will fill in the compiler ID/revision fields */
[0004] Asl Compiler ID : "xxxx"
[0004] Asl Compiler Revision : 00000000
[0004] Local Apic Address : fee00000
[0004] Flags (decoded below) : 00000001
    PC-AT Compatibility : 1

/* Processor Local APIC */
[0001] Subtable Type : 00
[0001] Length : 08
[0001] Processor ID : 00
[0001] Local Apic ID : 00
[0004] Flags (decoded below) : 00000001
    Processor Enabled : 1
    Runtime Online Capable : 0

/* IO APIC */
[0001] Subtable Type : 01
[0001] Length : 0C
[0001] I/O Apic ID : 00
[0001] Reserved : 00
[0004] Address : fec00000
[0004] Interrupt : 00000000

/* Interrupt Source Override */
/* Legacy IRQ0 is connected to pin 2 of the IOAPIC 
[0001] Subtable Type : 02
[0001] Length : 0A
[0001] Bus : 00
[0001] Source : 00
[0004] Interrupt : 00000002
[0002] Flags (decoded below) : 0000
    Polarity : 0
    Trigger Mode : 0 */

/* Local APIC NMI Structure */
/* Connected to LINT1 on all CPUs 
[0001] Subtable Type : 04
[0001] Length : 06
[0001] Processor ID : ff
[0002] Flags (decoded below) : 0000
    Polarity : 0
    Trigger Mode : 0
[0001] Interrupt Input LINT : 01 */