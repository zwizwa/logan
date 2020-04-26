
# For dev, bypasses cargo.


ALL :=	libla.rlib \
	saleae.elf \
	uart.elf \
	slip.elf \
	diff.elf \
	uart.dasm \
	uart.ll \
	../tests/test_uart.elf \
	../tests/test_slip.elf \
	../tests/test_syncser.elf \

.PHONY: all clean mrproper test run_uart

all: $(ALL)

# RUSTC := ~/armdev/bin/rustc-arm.sh
RUSTC := rustc
RUSTC_FLAGS :=  -C opt-level=3

# Equivalent to #[inline] for everything. This is actually slower.
# RUSTC_LDFLAGS :=  -C lto

clean:
	rm -f $(ALL) *~ *.ll *.elf *.so *.rlib *.dasm

mrproper: clean
	rm -rf SaleaeDeviceSdk*


libla.rlib: ../src/lib.rs
	$(RUSTC) --crate-name la --crate-type=lib $(RUSTC_FLAGS) $<

%.elf: %.rs libla.rlib
	RUST_BACKTRACE=1 $(RUSTC) $(RUSTC_FLAGS) $(RUST_LDFLAGS) -L . $< -o $@

%.elf: %.rs libla.rlib
	RUST_BACKTRACE=1 $(RUSTC) $(RUSTC_FLAGS) $(RUST_LDFLAGS) -L . $< -o $@

%.dasm: %.elf
	objdump -d $< >$@

%.ll: %.rs
	$(RUSTC) --emit=llvm-ir -L . $< -o $@


test: ../tests/test_uart.test ../tests/test_syncser.test

%.test: %.elf
	$<

run_uart: uart.elf saleae.elf
	./saleae.elf 8000000 | ./uart.elf




SALEAE_VER := 1.1.14

SaleaeDeviceSdk-$(SALEAE_VER).zip:
	wget http://downloads.saleae.com/SDK/$@

SaleaeDeviceSdk-$(SALEAE_VER): SaleaeDeviceSdk-$(SALEAE_VER).zip
	unzip $<

CFLAGS := -ISaleaeDeviceSdk-$(SALEAE_VER)/include -g
LDFLAGS := -L SaleaeDeviceSdk-$(SALEAE_VER)/lib/ -lSaleaeDevice64 -Xlinker -rpath -Xlinker SaleaeDeviceSdk-$(SALEAE_VER)/lib/

saleae.elf: saleae.cpp $(OBJS) SaleaeDeviceSdk-$(SALEAE_VER)
	g++ $(CFLAGS) $< -o $@ $(OBJS) $(LDFLAGS) -lpthread

