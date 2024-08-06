// This file is forked from uboot's source code for booting 2K1000.

// Note: the Address mapping window setting up already.
// 		 so, use the Mapped address 0x8xxx... or 0x9xxx...
/*
 * Simple character printing routine used before full initialization
 */

.globl init_serial
init_serial:
    or     $a4, $ra, $zero

	li.d	$a0, {CONSOLE_BASE_ADDR}
	li.w	$r18, 0x80
	// [UART0_BASE+3, LCR] = 1000_0000, dlab=1, enable access to freq divisor registers
	st.b	$r18, $a0, 3

	// CAUTION: now we are accessing the divisor registers! not the regular uart registers

	li.w	$r18, {UART_DIV_HI}
	// [UART0_BASE+1, DIV1] = {UART_DIV_HI}
	st.b	$r18, $a0, 1

	li.w	$r18, {UART_DIV_LO}
	// [UART0_BASE+0, DIV0] = {UART_DIV_LO}
	st.b	$r18, $a0, 0

	li.w	$r18, 3
	// [UART0_BASE+3, LCR] = 0000_0011, bec=2'b11, 8bit data
	st.b	$r18, $a0, 3

	li.w	$r18, 71
	// [UART0_BASE+2, FCR] = 71 = 0100_0111, TL=2'b01, reset rx/tx fifo, and we also stop access to divisor registers
	st.b	$r18, $a0, 2

	// enable receive data available interrupt
	// li.w	$r18, 0x0f
	// [UART0_BASE+1, IER][0] = 1 (IRxE)
	// st.b	$r18, $a0, 1

	or      $ra, $a4, $zero
	jirl	$zero, $ra, 0
    
.globl print_char
print_char:
	li.d	$r18, {CONSOLE_BASE_ADDR}
1:
	ld.bu	$r17, $r18, 0x5 // [UART0_BASE+5, LSR]
	andi	$r17, $r17, 0x20 // & 0x20(0010_0000), TFE=1, transmit fifo empty, so can send data!
	beqz	$r17, 1b // if TFE=0, loop and wait for TFE=1

	st.b	$a0, $r18, 0 // [UART0_BASE+0, DATA] = a0 which is the char to print
	
	// buffer at 0x9000_0003_5000_0008
	// buffer_tot at 0x9000_0003_6000_0000
	// li.d	$r19, 0x350000000
	// li.d	$r20, 0x350000008
	// ld.d	$r21, $r19, 0 // r[21] = buffer_tot(u64)
	// add.d	$r20, $r20, $r21 // r[20] = buffer + buffer_tot
	// st.b	$a0, $r20, 0 // buffer[buffer_tot] = a0
	// addi.d	$r21, $r21, 1
	// st.d	$r21, $r19, 0 // buffer_tot += 1

	jirl	$zero, $ra, 0


// after enabled uart confregs's interrupt, we can use this function to get char from uart.
// wheatfox 2024.3.11
.globl get_char
get_char:
	li.d	$r18, {CONSOLE_BASE_ADDR}

1:
	ld.bu	$r17, $r18, 0x5
	// check [UART0_BASE+5, LSR][0](DR,Data Received) bit
	andi	$r17, $r17, 0x01
	beqz	$r17, 1b

	// read [UART0_BASE+0, DATA] to $a0 which is the char to return
	ld.bu	$r4, $r18, 0

	jirl	$zero, $ra, 0