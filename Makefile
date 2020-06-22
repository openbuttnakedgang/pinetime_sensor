	
dev:
	cargo build

o:
	Ozone $(CURDIR)/scripts/conf_dev.jdebug &

check c:
	cargo check

