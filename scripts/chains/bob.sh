#!/usr/bin/env bash

set -e

cd $(dirname ${BASH_SOURCE[0]})/../..


while getopts p flag
do
    case "${flag}" in
        p) purge=true;;
    esac
done

if [ "$purge" = true ] ; then
./target/release/compound-chain purge-chain --base-path /tmp/chainz/bob --chain ./compoundChainSpec.json --database paritydb -y
fi

export MINER="ETH:0x66613A2d4908D130C908ccF2f298b235bACD427a"
./target/release/compound-chain \
  --base-path /tmp/chainz/bob \
  --chain ./compoundChainSpec.json \
  --port 30334 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --no-mdns \
  --rpc-methods Unsafe \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWQoFU7AqYZE5cCFdHVHgN4M25dnwZJEjUihV4FD3UmZhZ