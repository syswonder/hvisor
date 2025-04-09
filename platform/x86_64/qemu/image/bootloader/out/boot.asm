
platform/x86_64/qemu/image/bootloader/out/boot.elf:     file format elf64-x86-64


Disassembly of section .text:

0000000000008000 <entry16>:
    8000:	fa                   	cli
    8001:	fc                   	cld
    8002:	66 89 c1             	mov    ecx,eax
    8005:	31 c0                	xor    ax,ax
    8007:	8e d8                	mov    ds,ax
    8009:	8e c0                	mov    es,ax
    800b:	8e d0                	mov    ss,ax
    800d:	0f 01 16 58 80       	lgdtw  ds:0x8058
    8012:	0f 20 c0             	mov    eax,cr0
    8015:	66 83 c8 01          	or     eax,0x1
    8019:	0f 22 c0             	mov    cr0,eax
    801c:	ea 21 80 08 00       	jmp    0x8:0x8021

0000000000008021 <entry32>:
    8021:	66 b8 10 00 8e d8    	mov    eax,0xd88e0010
    8027:	8e c0                	mov    es,ax
    8029:	8e d0                	mov    ss,ax
    802b:	8e e0                	mov    fs,ax
    802d:	8e e8                	mov    gs,ax
    802f:	ff e1                	jmp    cx
    8031:	2e 8d b4 26 00       	lea    si,cs:[si+0x26]
    8036:	00 00                	add    BYTE PTR [bx+si],al
    8038:	00 8d b4 26          	add    BYTE PTR [di+0x26b4],cl
	...
    8048:	ff                   	(bad)
    8049:	ff 00                	inc    WORD PTR [bx+si]
    804b:	00 00                	add    BYTE PTR [bx+si],al
    804d:	9b                   	fwait
    804e:	cf                   	iret
    804f:	00 ff                	add    bh,bh
    8051:	ff 00                	inc    WORD PTR [bx+si]
    8053:	00 00                	add    BYTE PTR [bx+si],al
    8055:	93                   	xchg   bx,ax
    8056:	cf                   	iret
    8057:	00 17                	add    BYTE PTR [bx],dl
    8059:	00 40 80             	add    BYTE PTR [bx+si-0x80],al
	...
