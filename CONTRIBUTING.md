# Contributing

Pull requests are welcome! Please follow these guidelines:

## Code Quality

Before submitting, make sure you:

- **Run Clippy** to check for linting issues:

```shell
cargo clippy --workspace --all-features -- -D warnings
```

- **Format the code** to ensure consistency within the project:

```shell
cargo fmt --all
```

## Message Guidelines

- Use **sentence case** and **present tense**
- Start with **imperative verbs** (e.g., "Add", "Fix")
- Keep the message **concise**, **direct**, and **focused on intent** (e.g., avoid file names or code excerpts)
- For **non-functional** changes, include keywords like `docs`, `flake`, `locale`, or `workflow`, depending on the area affected
- Optionally, use a `for ...` suffix to clarify intent when it adds value

### Example

```
Add contribution guidelines and reference them in docs
```

## Packaging Notes

The officially maintained builds are the Nix flake and the `iwmenu-git` AUR package. If you're contributing packaging for other systems, please treat these as the reference implementations in terms of structure, dependencies, and build behavior.
