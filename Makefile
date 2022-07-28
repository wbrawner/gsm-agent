build:
	cargo build

release:
	cargo build --release

release_all: release_linux release_win32

release_linux:
	cargo build --release --target x86_64-unknown-linux-gnu
	strip target/x86_64-unknown-linux-gnu/release/gsm-agent

release_win32:
	cargo build --release --target x86_64-pc-windows-gnu
	strip target/x86_64-pc-windows-gnu/release/gsm-agent.exe

run:
	cargo run

watch:
	cargo-watch -x run
