all: main run

debug:
	cargo build --features 'disp_debug'
	screen -S c4_display sudo ./target/debug/c4_display

breakpoint:
	cargo build --features 'sbs_debug disp_debug'
	screen -S c4_display sudo ./target/debug/c4_display

main:
	cargo build --release

run:
	screen -S c4_display sudo ./target/release/c4_display
	# sudo ./target/release/c4_display

