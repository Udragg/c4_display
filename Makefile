target = armv7-unknown-linux-gnueabihf
usr = udragg
pass = udragg
ip_home = 192.168.0.29
dest = code/rust/c4_display/src
source = ./src

all: main

main:
	cargo build --release --target=$(target)

write:
	sftp $(usr)@$(ip_home)
	$(pass)
	cd $(dest)
	put -r $(source)
	exit
