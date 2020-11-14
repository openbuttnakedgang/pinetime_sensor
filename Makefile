	
OPT = -Z build-std

dev:
	cargo build $(OPT)

o:
	Ozone $(CURDIR)/scripts/conf_dev.jdebug &

check c:
	cargo check $(OPT)

