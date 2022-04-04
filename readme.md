# Kitchen

A web assembly experiment in Meal Planning and Shopping List management.

# Building

Ensure you have rust installed with support for the web assembly target. You can see instructions here: [Rust wasm book](https://rustwasm.github.io/docs/book/game-of-life/setup.html).

You will also want to have trunk installed. You can see instructions for that here: [trunk](https://trunkrs.dev/)

Then obtain the source. We do not at this time publish kitchen on [crates.io](https://crates.io/).

```sh
git clone https://github.com/zaphar/kitchen
cd kitchen
```

Assuming you have installed everything correctly, then you are ready to build.

```sh
make release
```

# Hacking on kitchen

If you want to hack on kitchen, then you may find it useful to use trunk in dev mode. The run script will run build the app and run trunk with it watching for changes and reloading on demand in your browser.

```sh
./run.sh
```

By default, it will use the `examples` directory in this repository to populate the recipes for testing. You can override this by setting `EXAMPLES=/full/path/to/recipes` and it will use that location instead.

# Nix support.

If all of the above looks like too much work, and you already use the nix package manager, then there is a handy nix flake available for you to use.

```sh
nix run github:zaphar/kitchen
```