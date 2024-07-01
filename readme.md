# Kitchen

A web assembly experiment in Meal Planning and Shopping List management.

# Building

Ensure you have rust installed with support for the web assembly target. You can see instructions here: [Rust wasm book](https://rustwasm.github.io/docs/book/game-of-life/setup.html).

```sh
git clone https://github.com/zaphar/kitchen
cd kitchen
```

Assuming you have installed everything correctly, then you are ready to build.

```sh
make release
```

# Hacking on kitchen

The run script will run build the app and run it for you.

```sh
./run.sh
```

By default, it will use the `examples` directory in this repository to populate the recipes for testing. You can override this by setting `EXAMPLES=/full/path/to/recipes` and it will use that location instead.

# Nix support.

If all of the above looks like too much work, and you already use the nix package manager, then there is a handy nix flake available for you to use.

```sh
nix run github:zaphar/kitchen
```
