# EnvSentry terminal demo

These fixtures are synthetic and contain no real credentials.

Run from the repository root:

```sh
envsentry compare demo/reference.env demo/production.env
envsentry compare demo/reference.env demo/production.env --format json
envsentry compare demo/reference.env demo/production.env --strict
```

Record the VHS tape from the repository root:

```sh
vhs demo/demo.tape
```

The tape writes `demo/demo.gif`.

## Build and install the CLI

From the repository root:

```sh
cargo install --path crates/envsentry-cli --locked
```

Ensure Cargo's binary directory is on `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

Add that export to `~/.zshrc` if it is not already present, then open a new
terminal or run `source ~/.zshrc`.

Verify:

```sh
command -v envsentry
envsentry --help
```

If you do not want a global install, use Cargo directly instead:

```sh
cargo run --quiet -p envsentry-cli -- compare demo/reference.env demo/production.env
```
