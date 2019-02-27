# sr-bonded-token

Bonded Token implementation as a Substrate runtime module.

## [Tutorial](./Tutorial.md)

This repository has an accompanying [tutorial](./Tutorial.md).

## Quickstart

Clone this repository locally.

```shell
$ git clone <repo>
$ cd sr-bonded-token
$ ./build.sh
$ cargo build --release
$ ./target/release/sr-bonded-token purge-chain --dev (may be required)
$ ./target/release/sr-bonded-token --dev

# In a new terminal window
$ cd bonded-token-ui
$ yarn && yarn dev
```

Navigate to `localhost:8000` in a web browser. Scroll to the bottom to see a UI element for interacting with the module. To interact you will need to restore the `sudo` key, ie Alice.

In the top Wallet box insert the following:
```
Seed - 0x416c696365202020202020202020202020202020202020202020202020202020
Name - Alice
```
