# Rust Transaction System

This repository holds a simple transaction system written in Rust. The objective
is to process a list of transactions (given in a CSV format) into a list of
final balances.

The input's IDs are unique, but not necessarily in order. For example:

```csv
type,       client,         tx,         amount
deposit,    1,              2,          1.7
deposit,    1,              2,          1.7
deposit,    3,              3,          5.0
withdrawal, 2,              5,          4.2
deposit,    2,              8,          7.0
withdrawal, 4,              4,          10.0
deposit,    2,              6,          1.0
deposit,    1,              7,          0.5
withdrawal, 1,              1,          2.0
```

Which would result in:

```csv
client,     available,      held,       total,      locked
1,          0.2,            0.0,        0.2,        false
2,          8.0,            0.0,        8.0,        false
3,          5.0,            0.0,        5.0,        false
```

The following operations are possible:

* Deposit: increase the available funds.
* Withdrawal: decrease the available funds, if possible.
* Dispute: mark a transaction as possibly erroneous, but don't do anything yet.
  For a deposit, subtract the amount from the user's available funds, and add it
  to the held funds. For a withdrawal, don't do anything, as the bank can't give
  currency away before it's resolved.
* Resolve: return the held funds to the available ones. In the case of
  withdrawals this doesn't do anything either because nothing changed in the
  dispute step.
* Chargeback: reverse a transaction by returning the held amount and decreasing
  or increasing the available funds. Finally, lock the account.

There are multiple tests for all the edge cases taken into account. The script
`gen_large.sh` can be used to generate an arbitrarily large file to measure
performance more accurately.

## Design decisions

* Disputes can only occur once; once a resolve or chargeback occurs, it may not
  start the dispute process again.
* Errors are handled with the `anyhow` crate, which makes the task quite easy.
  It attempts to be resilient to errors, e.g., if a line in the CSV is invalid,
  the error is reported, and it continues its execution.
* No `unsafe` usage at all, as it wasn't considered necessary for this simple
  implementation.

For the sake of simplicity, the following parts have left in a suboptimal state:

* The algorithm is written with CPU efficiency in mind, but it could be better
  in terms of memory. For example, there is no need to actually have
  `Balance::client`, because its ID is already known from the clients map.
* No parallelism is implemented, but one possible approach would be to divide
  the clients in ranges, each for one thread. It would need some experimentation
  because the operations are actually somewhat simple, and multithreading could
  just not make it more performant.
