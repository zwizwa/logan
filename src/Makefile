
# For dev, bypasses cargo.


ALL :=	libla.rlib \
	../examples/saleae.elf \
	../examples/uart.elf \
	../examples/diff.elf \
	../tests/test_uart.elf \
	../tests/test_syncser.elf \
	../examples/uart.dasm \
	../examples/uart.ll \


all: $(ALL)


RUSTC := rustc
RUSTC_FLAGS :=  -C opt-level=3

# Equivalent to #[inline] for everything. This is actually slower.
# RUSTC_LDFLAGS :=  -C lto

clean:
	rm -f $(ALL) *~ *.ll
	make -C ../examples clean

mrproper: clean
	make -C ../examples mrproper

lib%.rlib: %.rs
	$(RUSTC) --crate-name $* --crate-type=lib $(RUSTC_FLAGS) $<

%.elf: %.rs libla.rlib
	RUST_BACKTRACE=1 $(RUSTC) $(RUSTC_FLAGS) $(RUST_LDFLAGS) -L . $< -o $@

%.elf: %.rs libla.rlib
	RUST_BACKTRACE=1 $(RUSTC) $(RUSTC_FLAGS) $(RUST_LDFLAGS) -L . $< -o $@

%.dasm: %.elf
	objdump -d $< >$@

%.ll: %.rs
	$(RUSTC) --emit=llvm-ir -L . $< -o $@

../examples/saleae.elf:
	make -C ../examples saleae.elf

test: ../tests/test_uart.elf
	$<

run_uart: ../examples/uart.elf ../examples/saleae.elf
	cd ../examples ; ./saleae.elf 8000000 | ./uart.elf

