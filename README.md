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

## Resources

- [WebAssembly Specification][1]
- [WASI Documentation][2]
- [WebAssembly Reference Manual][3]
- [Fermyon (YouTube) - A WebAssembly Deep Dive][4]

[1]: https://webassembly.github.io/spec/
[2]: https://github.com/WebAssembly/WASI
[3]: https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md
[4]: https://www.youtube.com/watch?v=VGLnqkegX-g
