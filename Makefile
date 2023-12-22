
run:
	@RUST_LOG=info cargo run

watch:
	@watchexec --restart --exts rs --ignore public -- make run
