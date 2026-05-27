default_target := "x86_64-unknown-linux-musl"

build-release target=default_target:
    cargo build --release --target {{target}}

install: (build-release)
    cp target/{{default_target}}/release/mxr ~/.local/bin/mxr

ci-release:
    just build-release x86_64-unknown-linux-musl
    just build-release aarch64-unknown-linux-musl
