
all: saleae.elf
clean:
	rm -f *.elf

mrproper: clean
	rm -rf SaleaeDeviceSdk*

SALEAE_VER := 1.1.14

SaleaeDeviceSdk-$(SALEAE_VER).zip:
	wget http://downloads.saleae.com/SDK/$@

SaleaeDeviceSdk-$(SALEAE_VER): SaleaeDeviceSdk-$(SALEAE_VER).zip
	unzip $<

CFLAGS := -ISaleaeDeviceSdk-$(SALEAE_VER)/include -g
LDFLAGS := -L SaleaeDeviceSdk-$(SALEAE_VER)/lib/ -lSaleaeDevice64 -Xlinker -rpath -Xlinker SaleaeDeviceSdk-$(SALEAE_VER)/lib/

saleae.elf: saleae.cpp $(OBJS) SaleaeDeviceSdk-$(SALEAE_VER)
	g++ $(CFLAGS) $< -o $@ $(OBJS) $(LDFLAGS) 

