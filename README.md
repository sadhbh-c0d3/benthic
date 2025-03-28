# Benthic
Crypto Exchange in Rust

## Why Benthic?
"Benthic" directly refers to the bottom of the sea, where crabs live, establishing a clear link to the crab theme.

## Example Test Output

Benthic just kicked-off, and we implemented order manager and order book with execution policy, which allows us to
place and execute orders on the book. We support IOC, Limit and Market. Idea is to support Stop and OCO.
The execution policy needs work, will need to check available balance/margin of the participants when
placing/execution orders. Order manager needs to support Cancel order. Additionally execution policy should
route placements/executions into market data stream.

```console
New: 1.0BTC on: BTC/USDT Order(1001:1): Limit buy 1.0BTC @ 50000.0USDT
New: 1.0BTC on: BTC/ETH Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH
Execute: 0.50000BTC on: BTC/ETH Order(1002:3): Limit buy 0.50000BTC @ 12.5000ETH Aggressor
Execute: 0.50000BTC on: BTC/ETH Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH
Cancel: 0.0BTC on: BTC/ETH Order(1002:3): Limit buy 0.50000BTC @ 12.5000ETH - Reason: Not enough quantity
```
