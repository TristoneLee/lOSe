# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/lOSe
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
CPUS := 1

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80000000

# Binutils
OBJCOPY := rust-objcopy --binary-architecture=riscv64
OBJDUMP := rust-objdump --arch-name=riscv64

# Disassembly
DISASM ?= -x

build: env $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
#	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@cargo build --release

clean:
	@cargo clean

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-smp $(CPUS) \
		-nographic \
		-bios none \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -smp $(CPUS) -nographic -bios none -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) "  \
		tmux split-window -h "riscv64-linux-gnu-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'"  \
		tmux -2 attach-session -d
