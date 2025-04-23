# Benthic 🦀 Rust-based Core for a Crypto-Exchange 💹

[![Watch My Video!](https://img.youtube.com/vi/plTm7eEDebw/0.jpg)](https://youtu.be/plTm7eEDebw&list=PLAetEEjGZI7OUBYFoQvI0QcO9GKAvT1xT&index=1)

**Dive into the Rust-powered core of a cryptocurrency exchange!**

This project, ***Benthic*** (referring to the bottom of the sea, home to crabs
🦀), showcases *the* fundamental components for order execution and more, as
featured in my "C++ *vs* Rust" series on YouTube.

## Why Benthic?

"Benthic" directly refers to the bottom of the sea, where crabs live,
establishing a clear link to the Rustacean theme.

## Key Functionalities

* **Order Matching:** Implementation of a matching engine for efficiently
  pairing buy and sell orders.
* **Margin Component:** Architectural design and structure of a margin handling
  component.
* **Account Management:** Real-time updating of trader balances and asset
  positions with subaccount support for managing open and closed lots.
* **Lot Management:** Tracking the lifecycle of trading lots, including the
  handling of "inflight lots" from executed transactions.
* **Decimal Handling:** Efficient representation of decimal prices and
  quantities using `u64`.
* **Polynomial Decomposition for Multiplication:** Demonstration of an optimized
  multiplication technique for decimal values.

##  Getting Started
Get started by exploring the core order execution flow with the `order_execution` example:

```bash
cargo run --example order_execution
```

This example demonstrates placing and executing orders (supporting IOC, Limit,
and Market orders) using the implemented order manager and order book with its
execution policy. You'll see how orders are processed and the initial impact on
simulated margin accounts.

## License

This project is licensed under the **MIT License**. See the [LICENSE](./LICENSE)
file for more details.

## Order Execution Example

The output from running `cargo run --example order_execution` provides a
step-by-step view of the system in action:

1.  **Initial Account & Lot Creation:** You'll see the creation of margin
accounts for users (e.g., `Account(1001)`) and the opening of initial asset
positions ("lots") based on deposits (e.g., `Lot(1001:BTC): open Long 2.0`).
2.  **Order Placement and Promising:** When a user places an order (e.g.,
`Order(1001:1) Limit buy 1.0BTC ...`), the system acknowledges it with a
"Promise." This promise, managed through the `ExecutionPolicy`, `MarginManager`,
`MarginAccount`, `MarginAssetAccount`, and ultimately stored as `open_quantity`
on the `MarginSide`, reserves the intent to acquire or deliver the asset.
3.  **Market Depth Update:** Placed orders are reflected in the market depth for
the respective trading pair.
4.  **Order Matching and Execution:** When compatible orders exist, a match
occurs, leading to trade executions. This involves adjusting the open and closed
lots for both traders.
5.  **Account Updates:** Executions result in updates to the traders' account
balances and their open/closed positions in different assets.
6.  **Lot Lifecycle:** The output demonstrates the opening and closing of "lots"
associated with specific orders and their impact on the overall asset positions
of the traders.
7.  **Potential Cancellations:** In scenarios where the full order cannot be
executed due to insufficient opposing quantity, cancellation messages may
appear.

By observing this output, you can trace the journey of orders through the
Benthic exchange, from initial placement to potential execution and the
resulting changes in user accounts and asset holdings.

### Example Output

```output
Margin  -->  create Account(1001)
Margin   <-- Lot(1001:BTC):  open Long   2.0                      <- (Order(1001:101): Deposit 2.0BTC at 50000.0USDT)
Margin  -->  create Account(1002)
Margin   <-- Lot(1002:ETH):  open Long   20.0                     <- (Order(1002:102): Deposit 20.0ETH at 4000.0USDT)

Account  1001         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)        0.0 |        2.0        (0.0)
          ETH          (0.0)        0.0 |        0.0        (0.0)
         USDT          (0.0)        0.0 |        0.0        (0.0)

Account  1002         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)        0.0 |        0.0        (0.0)
          ETH          (0.0)        0.0 |       20.0        (0.0)
         USDT          (0.0)        0.0 |        0.0        (0.0)

User --->    Order(1001:1) Limit buy 1.0BTC @ 50000.0USDT
User    <--- Promise(BTC/USDT):           1.0BTC                   <- (Order(1001:1): Limit buy 1.0BTC @ 50000.0USDT)
Market   <-- Depth(BTC/USDT):             1.0BTC                   <- (Order(1001:1): Limit buy 1.0BTC @ 50000.0USDT)
User --->    Order(1001:2) Limit sell 1.0BTC @ 12.5000ETH
User    <--- Promise(BTC/ETH):           1.0BTC                   <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)
Market   <-- Depth(BTC/ETH):             1.0BTC                   <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)

Account  1001         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (1.0)        0.0 |        2.0        (1.0)
          ETH          (0.0)        0.0 |        0.0  (12.500000)
         USDT      (50000.0)        0.0 |        0.0        (0.0)

Account  1002         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)        0.0 |        0.0        (0.0)
          ETH          (0.0)        0.0 |       20.0        (0.0)
         USDT          (0.0)        0.0 |        0.0        (0.0)

User --->    Order(1002:3) Limit buy 0.50000BTC @ 12.5000ETH
Margin   <-- Lot(1002:BTC):  open Long   0.5000000                <- (Order(1002:3): Limit buy 0.50000BTC @ 12.5000ETH at 12.5000ETH)
Margin   <-- Lot(1002:ETH): close Long   13.750000  (20.0)        <- (Order(1002:3): Limit buy 0.50000BTC @ 12.5000ETH at 12.5000ETH)
Margin   <-- Lot(1001:BTC): close Long   1.5000000  (2.0)         <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH at 12.5000ETH)
Margin   <-- Lot(1001:ETH):  open Long   6.250000                 <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH at 12.5000ETH)
User    <--- Execute(BTC/ETH:Aggressor): 0.50000BTC               <- (Order(1002:3): Limit buy 0.50000BTC @ 12.5000ETH)
User    <--- Execute(BTC/ETH:Book):      0.50000BTC               <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)
Market   <-- Trade(BTC/ETH):             0.50000BTC               <- (Order(1002:3): Limit buy 0.50000BTC @ 12.5000ETH) x (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)
User    <--- Cancel(BTC/ETH):            0.0BTC                   <- (Order(1002:3): Limit buy 0.50000BTC @ 12.5000ETH) - Reason: Not enough quantity

Account  1001         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC    (0.5000000)        0.0 |  1.5000000        (1.0)
          ETH          (0.0)        0.0 |   6.250000   (6.250000)
         USDT      (50000.0)        0.0 |        0.0        (0.0)

Account  1002         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)        0.0 |  0.5000000        (0.0)
          ETH          (0.0)        0.0 |  13.750000        (0.0)
         USDT          (0.0)        0.0 |        0.0        (0.0)

User --->    Order(1002:4) Limit buy 1.0BTC @ 12.0ETH
User    <--- Promise(BTC/ETH):           1.0BTC                   <- (Order(1002:4): Limit buy 1.0BTC @ 12.0ETH)
Market   <-- Depth(BTC/ETH):             1.0BTC                   <- (Order(1002:4): Limit buy 1.0BTC @ 12.0ETH)
User --->    Order(1002:5) Limit buy 1.0BTC @ 14.0ETH
Margin   <-- Lot(1002:BTC):  open Long   1.0                      <- (Order(1002:5): Limit buy 1.0BTC @ 14.0ETH at 12.5000ETH)
Margin   <-- Lot(1002:ETH): close Long   1.250000   (20.0)        <- (Order(1002:5): Limit buy 1.0BTC @ 14.0ETH at 12.5000ETH)
Margin   <-- Lot(1001:BTC): close Long   0.5000000  (2.0)         <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH at 12.5000ETH)
Margin   <-- Lot(1001:ETH):  open Long   12.500000                <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH at 12.5000ETH)
User    <--- Execute(BTC/ETH:Aggressor): 1.0BTC                   <- (Order(1002:5): Limit buy 1.0BTC @ 14.0ETH)
User    <--- Execute(BTC/ETH:Book):      1.0BTC                   <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)
Market   <-- Trade(BTC/ETH):             1.0BTC                   <- (Order(1002:5): Limit buy 1.0BTC @ 14.0ETH) x (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)
User    <--- Cancel(BTC/ETH):            0.0BTC                   <- (Order(1002:5): Limit buy 1.0BTC @ 14.0ETH) - Reason: Not enough quantity

Account  1001         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)        0.0 |  0.5000000        (1.0)
          ETH          (0.0)        0.0 |  18.750000        (0.0)
         USDT      (50000.0)        0.0 |        0.0        (0.0)

Account  1002         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)        0.0 |  1.5000000        (1.0)
          ETH         (12.0)        0.0 |   1.250000        (0.0)
         USDT          (0.0)        0.0 |        0.0        (0.0)

User --->    Order(1002:6) Limit buy 1.0BTC @ 15.0ETH
Margin   <-- Lot(1002:BTC):  open Long   1.0                      <- (Order(1002:6): Limit buy 1.0BTC @ 15.0ETH at 12.5000ETH)
Margin   <-- Lot(1002:ETH):  open Short  11.250000                <- (Order(1002:6): Limit buy 1.0BTC @ 15.0ETH at 12.5000ETH)
Margin   <-- Lot(1001:BTC):  open Short  0.5000000                <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH at 12.5000ETH)
Margin   <-- Lot(1001:ETH):  open Long   12.500000                <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH at 12.5000ETH)
User    <--- Execute(BTC/ETH:Aggressor): 1.0BTC                   <- (Order(1002:6): Limit buy 1.0BTC @ 15.0ETH)
User    <--- Execute(BTC/ETH:Book):      1.0BTC                   <- (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)
Market   <-- Trade(BTC/ETH):             1.0BTC                   <- (Order(1002:6): Limit buy 1.0BTC @ 15.0ETH) x (Order(1001:2): Limit sell 1.0BTC @ 12.5000ETH)
User    <--- Cancel(BTC/ETH):            0.0BTC                   <- (Order(1002:6): Limit buy 1.0BTC @ 15.0ETH) - Reason: Not enough quantity

Account  1001         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)  0.5000000 |        0.0        (1.0)
          ETH          (0.0)        0.0 |  31.250000        (0.0)
         USDT      (50000.0)        0.0 |        0.0        (0.0)

Account  1002         (Open)      Short |       Long       (Open)
----------------------------------------------------------------
          BTC          (0.0)        0.0 |  2.5000000        (1.0)
          ETH         (12.0)  11.250000 |        0.0        (0.0)
         USDT          (0.0)        0.0 |        0.0        (0.0)
```

## Performance

We measured performance using benchmark built on top of Criterion.

### Running the benchmark

```bash
cargo bench
```

### Machine

* **CPU:** AMD Ryzen 9 7900X3D @ 5.6GHz
* **RAM:** PNY 6000 32GB @ 4800 MT/s
* **MB:** Gigabyte Aorus X670 ELITE AX, BIOS: FA1

### Results

We have performed number of tests with varying number of orders and traders.

#### 100'000 orders and 10'000 traders, only Order Book

During warm-up we placed 10'000'000 limit orders on the book, and we've got 5'005'111 executions within <1s.

In this test we achieve time 20.822ms per one 100'000 order batch, which is equivalent to 4'802'612 orders per second.

```output
Warm-up: time 0s, orders 0, executions 0
Ready: time 1s, orders 10000000, executions 5005111
Finished: time 12s, orders 65500000, executions 32783416

order_execution_mixed   time:   [19.798 ms 20.822 ms 22.076 ms]
```


#### 1'000'000 orders and 10'000 traders, only Order Book

During warm-up we placed 100'000'000 limit orders on the book, and we've got 49'924'912 executions within 27s.

In this test we achieve time 283.34ms per one 1'000'000 order batch, which is equivalent to 3'529'328 orders per second.

```output
Warm-up: time 0s, orders 0, executions 0
Ready: time 27s, orders 100000000, executions 49924912
Finished: time 60s, orders 215000000, executions 107338547

order_execution_mixed   time:   [278.12 ms 283.34 ms 289.82 ms]
```

#### 10'000 orders and 1'000 traders, with Margin Accounts

During warm-up we placed 999'400 limit orders on the book, and we've got 489'307 executions within <1s.

In this test we achieve time 22.020ms per one 10'000 order batch, which is equivalent to 454'132 orders per second.

```output
Warm-up: time 0s, orders 0, executions 0
Ready: time 0s, orders 999400, executions 489307
Finished: time 18s, orders 11103334, executions 5436130

order_execution_mixed   time:   [21.434 ms 22.020 ms 22.641 ms]
```

#### 10'000 orders and 10'000 traders, with Margin Accounts

During warm-up we placed 999'900 limit orders on the book, and we've got 491'811 executions within <1s.

In this test we achieve time 7.0406ms per one 10'000 order batch, which is equivalent to 1'419'244 orders per second.

```output
Warm-up: time 0s, orders 0, executions 0
Ready: time 0s, orders 999900, executions 491811
Finished: time 10s, orders 15108489, executions 7431109

order_execution_mixed   time:   [6.7742 ms 7.0406 ms 7.3967 ms]
```

#### 100'000 orders and 10'000 traders, with Margin Accounts

During warm-up we placed 9'999'499 limit orders on the book, and we've got 5'005'111 executions within 9s.

In this test we achieve time 123.76ms per one 100'000 order batch, which is equivalent to 808'015 orders per second.

```output
Warm-up: time 0s, orders 0, executions 0
Ready: time 9s, orders 9999499, executions 5005111
Finished: time 24s, orders 23098844, executions 11561792

order_execution_mixed   time:   [121.58 ms 123.76 ms 126.14 ms]
```

#### 100'000 orders and 1'000 traders, with Margin Accounts

During warm-up we placed 9'995'798 limit orders on the book, and we've got 4'984'809 executions within 19s.

In this test we achieve time 459.07ms per one 100'000 order batch, which is equivalent to 211'304 orders per second.

```output
Warm-up: time 0s, orders 0, executions 0
Ready: time 19s, orders 9995798, executions 4984809
Finished: time 71s, orders 21490968, executions 10717329

order_execution_mixed   time:   [459.07 ms 473.25 ms 487.07 ms]
```

#### 100'000 orders and 100 traders, with Margin Accounts

During warm-up we placed 9'953'399 limit orders on the book, and we've got 4'981'018 executions within 107s.

In this test we achieve time 3.2504s per one 100'000 order batch, which is equivalent to 30'769 orders per second.

```output
Warm-up: time 0s, orders 0, executions 0
Ready: time 107s, orders 9953399, executions 4981018
Finished: time 439s, orders 20205401, executions 10111448

order_execution_mixed   time:   [3.1103 s 3.2504 s 3.3888 s]
```

#### 50'000 orders and 1'000 traders, with Margin Accounts, Lots flushing, and static Lots handler
```output

Branch: perf/lots-vecdeque-with-flush
Config: NUM_TRADERS = 1000, NUM_ORDERS = 50000, BENCHMARK_VERSION = Static Lots Handler (VecDeque)

Ready: time 2s, orders 4997898, executions 2493009
Finished: time 13s, orders 21341031, executions 10645119

order_execution_mixed   time:   [31.949 ms 32.734 ms 33.729 ms]
```

#### 500'000 orders and 1'000 traders, with Margin Accounts, Lots flushing, and static Lots handler
```output
Branch: perf/lots-vecdeque-with-flush
Config: NUM_TRADERS = 1000, NUM_ORDERS = 500000, BENCHMARK_VERSION = Static Lots Handler (VecDeque)

Ready: time 49s, orders 49973698, executions 24963209
Finished: time 103s, orders 103445557, executions 51673833

order_execution_mixed   time:   [486.52 ms 501.11 ms 519.65 ms]
memory used ~11GB
```

#### 500'000 orders and 1'000 traders, with Margin Accounts and Lots flush, and static Lots Handler
```output
Branch: perf/lots-intrusive-list-with-flush
Config: NUM_TRADERS = 1'000, NUM_ORDERS = 500'000, BENCHMARK_VERSION = Static Lots Handler (intrusive LinkedList)

Ready: time 63s, orders 49973698, executions 24963209
Finished: time 137s, orders 103445557, executions 51673833

order_execution_mixed   time:   [684.73 ms 689.97 ms 696.37 ms]
memory used ~22GB
```
