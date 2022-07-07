## Selendra-testing

### how to run

```
cargo build --release

./target/release/selendra --dev \
--ws-port=9944 \
--ws-external \
--rpc-port=9933 \
--rpc-external \
--rpc-cors=all \
--rpc-methods=unsafe
```

## Connect metamask
goto [eth-rpc-adapter](https://github.com/AcalaNetwork/bodhi.js/tree/master/eth-rpc-adapter) and run service. after you use any api that work with evm

## bind account (to get balance in evm account)
goto [Evm playground](https://evm.acala.network/#/Bind%20Account)

and insert following data:
Substrate address: address you to bind with evm address
Chain id: 597
Genesis hash: [here](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/settings/metadata)


after that goto **extrinsic** and them select **evmAccount** and selendra **claimAccount** and fill data that get from [Evm playground](https://evm.acala.network/#/Bind%20Account)
