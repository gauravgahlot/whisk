.PHONY: wat2wasm
wat2wasm: # Convert wasm from text to binary format
	wat2wasm samples/wats/empty.wat -o bin/empty.wasm
	wat2wasm samples/wats/hello.wat -o bin/hello.wasm
	wat2wasm samples/wats/plus_one.wat -o bin/plus_one.wasm

