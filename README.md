# Benthic
Crypto Exchange in Rust

[![Watch My Video!](https://img.youtube.com/vi/plTm7eEDebw/0.jpg)](https://youtu.be/plTm7eEDebw&list=PLAetEEjGZI7OUBYFoQvI0QcO9GKAvT1xT&index=1)

## Why Benthic?
"Benthic" directly refers to the bottom of the sea, where crabs live, establishing a clear link to the crab theme.

## Example Test Output

Benthic just kicked-off, and we implemented order manager and order book with execution policy, which allows us to
place and execute orders on the book. We support IOC, Limit and Market. Idea is to support Stop and OCO.
The execution policy needs work, will need to check available balance/margin of the participants when
placing/execution orders. Order manager needs to support Cancel order. Additionally execution policy should
route placements/executions into market data stream.

```console
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
