
all: saleae.elf la.rlib run_uart.elf run_diff.elf test_uart.elf stuff.rlib


RUSTC = rustc

clean:
	rm -f *.elf *~ *.ll *.rlib

%.rlib: %.rs
	$(RUSTC) --crate-name la --crate-type=lib -C opt-level=3 $<

%.elf: %.rs la.rlib
	RUST_BACKTRACE=1 $(RUSTC) -C opt-level=3 -L . $< -o $@

%.ll: %.rs
	$(RUSTC) --emit=llvm-ir $< -o $@


SALEAE_VER := 1.1.14

SaleaeDeviceSdk-$(SALEAE_VER).zip:
	wget http://downloads.saleae.com/SDK/$@

SaleaeDeviceSdk-$(SALEAE_VER): SaleaeDeviceSdk-$(SALEAE_VER).zip
	unzip $<

CFLAGS := -ISaleaeDeviceSdk-$(SALEAE_VER)/include -g
LDFLAGS := -L SaleaeDeviceSdk-$(SALEAE_VER)/lib/ -lSaleaeDevice64 -Xlinker -rpath -Xlinker SaleaeDeviceSdk-$(SALEAE_VER)/lib/

saleae.elf: saleae.cpp $(OBJS) SaleaeDeviceSdk-$(SALEAE_VER)
	g++ $(CFLAGS) $< -o $@ $(OBJS) $(LDFLAGS) 


test: all
	./test_uart.elf

run_uart: all
	./saleae.elf 8000000 | ./run_uart.elf

