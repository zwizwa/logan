
all: saleae.elf libla.rlib run_uart.elf run_diff.elf test_uart.elf libstuff.rlib


RUSTC = rustc

clean:
	rm -f *.elf *~ *.ll *.rlib

lib%.rlib: %.rs
	$(RUSTC) --crate-name $* --crate-type=lib -C opt-level=3 $<

%.elf: %.rs libla.rlib
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


test: test_uart.elf
	./test_uart.elf

run_uart: run_uart.elf saleae.elf
	./saleae.elf 8000000 | ./run_uart.elf

