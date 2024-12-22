<div align="center">
  <h1><code>whisk</code></h1>

  <p>
    <strong>A minimal <a href="https://webassembly.org/">WebAssembly</a> runtime</strong>, created for fun (and learning)
  </p>

  <p>
    <a href="https://github.com/gauravgahlot/whisk/actions?query=workflow%3ACI"><img src="https://github.com/gauravgahlot/whisk/actions/workflows/ci.yaml/badge.svg" alt="build status" /></a>
    <!-- <a href="https://docs.rs/whisk"><img src="https://docs.rs/whisk/badge.svg" alt="Documentation Status" /></a> -->
  </p>
</div>

## How to use?

```sh
# clone the GitHub repository
git clone https://github.com/gauravgahlot/whisk.git
cd whisk

# build whisk
make
```

### Examples

#### **WebAssembly Text Format** (`.wat`)

- Create a file with the following content and save it as `hello.wat`:

```wat
(module
  (func (export "main")
      (result i32)
    i32.const 1
    return))
```

- Build a WebAssembly module using `wat2wasm`:

```sh
wat2wasm hello.wat -o hello.wasm
```

- Run the module with `whisk`

```sh
whisk hello.wasm
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
