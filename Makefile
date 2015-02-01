
all: la.elf saleae.elf

RUSTC = rustc

clean:
	rm -f *.elf *~ *.ll

%.elf: %.rs
	$(RUSTC) $< -o $@

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

test1: all
# head /dev/urandom | ./logic.elf
	echo 'ABCDE' | ./logic.elf

test2: all
	./saleae.elf | ./logic.elf # | ./column.elf | head -n 100

