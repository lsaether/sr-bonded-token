# sr-bonded-token

Bonded Token implementation as a Substrate runtime module.

## [Tutorial](./Tutorial.md)

This repository has an accompanying [tutorial](./Tutorial.md).

## Quickstart

Clone this repository locally.

```shell
$ git clone <repo>
$ cd sr-bonded-token
$ cargo build --release
$ ./build.sh
$ ./target/release/sr-bonded-token --dev

# In a new terminal window
$ cd bonded-token-ui
$ yarn && yarn dev
```

Navigate to `localhost:8000` in a web browser. Scroll to the bottom to see a UI element for interacting with the module.
