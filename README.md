# CoinJar

Double-entry accounting with contacts profiles. Inspired by [beancount](https://github.com/beancount/beancount).

## Features

- More concise ways to write your ledger.
- Handles split transactions out-of-box.

For example, you can write:

```
currency
    USD $ -- US dollars
    EUR € -- Euro
    CJM -- CoinJar Airline Miles

today
    Coffee @Starbucks
    expense/food/drinks  $3.50
    asset/cash

    #[split(@John)]
    Lunch with John @McDonald's
    expense/food/dine out 10 usd
    liability/@Bank of America/credits

yesterday
    #[split(by @John)]
    Dinner with John @Fuku Ramen
    expense/food/dine out 20 eur

    Gift for John @LA Airport
    expense/gifts/@John 20 eur
    asset/cash

today
    Bonus flight @CoinJar Airline
    asset/airline-miles/@CoinJar Airline 5000 CJM
    income/gifts
```

Now format it with

```sh
$ coinjar fmt
```

to give you (today is 2024-01-06):

```
currency
    CJM  -- CoinJar Airline Miles
    EUR € -- Euro
    USD $ -- US dollars

2024-01-05
    Dinner with John @Fuku Ramen
    liability/@John/payable                                        -€10.00
    expense/food/dine out                                           €10.00

    Gift for John @LA Airport
    expense/gifts/@John                                             €20.00
    asset/cash                                                     -€20.00

2024-01-06
    Coffee @Starbucks
    expense/food/drinks                                              $3.50
    asset/cash                                                      -$3.50

    Lunch with John @McDonald's
    asset/@John/receivable                                           $5.00
    expense/food/dine out                                            $5.00
    liability/@Bank of America/credits                             -$10.00

    Bonus flight @CoinJar Airline
    asset/airline-miles/@CoinJar Airline                        5000.00 CJM
    income/gifts                                                -5000.00 CJM
```

Show the money you owe/own @John:

```bash
$ coinjar contact John

Contact John
┌────────────┬──────────────────┬─────────┬─────────┐
│ date       │ desc             │ change  │ balance │
├────────────┼──────────────────┼─────────┼─────────┤
│ 2024-01-05 │ Dinner with John │ -€10.00 │ -€10.00 │
├────────────┼──────────────────┼─────────┼─────────┤
│ 2024-01-06 │ Lunch with John  │ $5.00   │ -€10.00 │
│            │                  │         │ $5.00   │
└────────────┴──────────────────┴─────────┴─────────┘
```

Show all the money you dealt with @John (including gifts)

```bash
$ coinjar contact John --all

Contact John
┌────────────┬──────────────────┬─────────┬─────────┐
│ date       │ desc             │ change  │ balance │
├────────────┼──────────────────┼─────────┼─────────┤
│ 2024-01-05 │ Dinner with John │ -€10.00 │ -€10.00 │
├────────────┼──────────────────┼─────────┼─────────┤
│ 2024-01-05 │ Gift for John    │ €20.00  │ €10.00  │
├────────────┼──────────────────┼─────────┼─────────┤
│ 2024-01-06 │ Lunch with John  │ $5.00   │ €10.00  │
│            │                  │         │ $5.00   │
└────────────┴──────────────────┴─────────┴─────────┘
```
