# Gacha Lab web showcase

The interface runs the repository's Rust probability engine in the browser through WebAssembly.

## Development

Requirements: Bun, Rust, `wasm32-unknown-unknown`, and `wasm-pack`.

```sh
bun install
bun run dev
```

`bun run dev` rebuilds the WASM adapter before starting the site. Use `bun run test` for the production build plus rendered-page and WASM integration tests.
