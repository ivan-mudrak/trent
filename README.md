# trent

## Overview

**Tr**ansaction**En**gine**T**

## Design

### Types

#### Account

Main type keeping single *AccountState* under thread safe Arc<Mutex<...>> wrapping and thus responsible for settling
transactions.

**NOTE**: such thread safe wrapping might not be needed since we need to '*pin*' each client to a dedicated thread in
order
to process transactions in order.

#### AccountState

Core type holding account balances and lock info.

#### Clients

Type holding all clients accounts.

#### ClientAccount

Helper type to write out account state.

#### ClientTx

Representation of clients transaction.

#### Tx

Enum representing all possible transactions.

### Engine

Simple transaction engine having two modes:

+ single-threaded: just settle all transactions sequentially
+ multi-threaded:
    + spawn workers equal to the number of threads available on underlying machine
    + '*pin*' each potential client to a dedicated worker using simple deterministic sharding function - *client_id %
      number_of_shards*

### Readers

Helper module to read input file either all at once or stream row-by-row:

+ single-threaded: file read at once
+ multi-threaded: streamed row-by-row
  **NOTE**: reading file at once will fail if unknown transaction type or wrong format encountered, it is not a problem
  for
  steaming approach since there each row parsed separately

## Test

There a bunch of self-explanatory tests to ensure engine correctness:

+ simple_settle_tx
+ not_enough_funds
+ double_dispute
+ double_resolve
+ try_withdraw_when_all_under_dispute
+ try_deposit_when_locked
+ try_withdraw_when_locked

plus test ensuring that multi-threaded mode gives the same results as single-threaded:

+ single_vs_multi

## Benchmark

Interestingly single threaded mode shows better performance with large and small files.

Although there is a switch in the code which will use multi-threaded mode if input file size is more than **10 Mb**.

### 100 Mb input file

+ single-threaded ~ 36s
+ multi-threaded ~ 40s

### 10 Mb input file

+ single-threaded ~ 4.7s
+ multi-threaded ~ 5.4s

### 5 Kb input file

+ single-threaded ~ 3.2ms
+ multi-threaded ~ 4.2ms