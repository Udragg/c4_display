all: main run

debug:
	cargo build --features sbs_debug
	screen -S c4_display cargo run

main:
	cargo build --release

run:
	screen -S c4_display sudo ./target/release/c4_display

