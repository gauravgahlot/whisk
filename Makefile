.PHONY: build
build:
	@cargo install -q --path .
	@echo "Successfully installed 'whisk' in $(HOME)/.cargo/bin/"

.PHONY: wat2wasm
wat2wasm: # Convert wasm from text to binary format
	wat2wasm samples/wat/empty.wat -o bin/wat/empty.wasm
	wat2wasm samples/wat/hello.wat -o bin/wat/hello.wasm
	wat2wasm samples/wat/plus_one.wat -o bin/wat/plus_one.wasm

.PHONY: rust2wasm
rust2wasm: # Build rust samples to wasm binary format
	cd samples/rust/plus_one && \
		cargo build --target wasm32-unknown-unknown --release && \
		cp target/wasm32-unknown-unknown/release/plus_one.wasm ../../../bin/rust/
