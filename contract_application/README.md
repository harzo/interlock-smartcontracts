# Port/Socket Application Template Contract

Description goes here.

## How to get setup and build:

#### See [[DOCUMENTATION]](https://interlock-network.github.io/interlock-smartcontracts/contract_application/docs/application/).

## How to test on testnet: 

##### To deploy a contract to testnet, `upload metadata.json` and `ilockmvp.wasm` (in `target`) to [testnet.alephzero.org](https://testnet.alephzero.org).

## How to build and run tests

##### To view debug prints and assertion failures run test via:
```
cargo +nightly test --features e2e-tests -- --show-output
```
##### To view debug for specific method run test via:
```
cargo +nightly test <test_function_here> -- --nocapture
```
##### To run end-to-end tests, first make sure you have the substrate dev node capabilities installed via:
```
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git
```
##### Then run the node:
```
substrate-contracts-node --log info,runtime::contracts=debug 2>&1
```
##### Finally, you can run the end-to-end and unit test suite:
```
cargo +nightly test --features e2e-tests -- --show-output
```