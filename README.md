<div align="center">
  <h1><code>whisk</code></h1>

  <p>
    <strong>A minimal <a href="https://webassembly.org/">WebAssembly</a> runtime</strong>, create for fun (and learning)
  </p>

  <p>
    <a href="https://github.com/gauravgahlot/whisk/actions?query=workflow%3ACI"><img src="https://github.com/gauravgahlot/whisk/actions/workflows/ci.yaml/badge.svg" alt="build status" /></a>
    <a href="https://docs.rs/whisk"><img src="https://docs.rs/whisk/badge.svg" alt="Documentation Status" /></a>
  </p>
</div>

## Usage Example

```sh
# generate wasm modules
make wat2wasm

# run a wasm module
cargo run -- bin/wat/hello.wasm
```
