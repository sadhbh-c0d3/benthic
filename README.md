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

* **Order Matching**: Implementation of a matching engine for efficiently
  pairing buy and sell orders.
* **Margin Component**: Architectural design and structure of a margin handling
  component.
* **Account Management**: Real-time updating of trader balances and asset
  positions with subaccount support for managing open and closed lots.
* **Lot Management**: Tracking the lifecycle of trading lots, including the
  handling of "inflight lots" from executed transactions.
* **Decimal Handling**: Efficient representation of decimal prices and
  quantities using `u64`.
* **Polynomial Decomposition for Multiplication**: Demonstration of an optimized
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
