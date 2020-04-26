
.PHONY: all test clean
all:
	cargo build --release
	if [ ! -z "$$ELF" ]; then ln -f target/release/lars "$$ELF"; fi

test:
	cargo test --verbose

clean:
	rm -rf target *~ *.elf

%.dasm: %.elf
	arm-none-eabi-objdump -d $< >$@.tmp
	mv $@.tmp $@

%.readelf: %.elf
	readelf -a $< >$@.tmp
	mv $@.tmp $@


