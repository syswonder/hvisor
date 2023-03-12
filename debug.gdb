set disassemble-next-line on
set confirm off
add-symbol-file target/aarch64-unknown-linux-gnu/debug/armv8-baremetal-demo-rust
target remote tcp::1234
set arch aarch64
layout regs