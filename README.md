## The Glacier Programming Language

Glacier is a dynamically typed programming language continued from [Gorilla](https://github.com/SnowballSH/Gorilla).

### Features

- Elegant and Clean

- Prefect for Code Golfing

- Optimized Speed

### Example: FizzBuzz

```ruby
i = 1
while i <= 30
    print(
        if i % 15 == 0
            "FizzBuzz"
        else if i % 3 == 0
            "Fizz"
        else if i % 5 == 0
            "Buzz"
        else i
    )
    i = i + 1
end
```

### Example: Fibonacci Number

```ruby
fn F(i)
    if i <= 1 i
    else F(i - 1) + F(i - 2)
```

Note that Glacier is Work-in-Progress, so this example will change rapidly with the introduction of subroutines and
suffix operators. (i++)

### Benchmark

- Fibonacci

| bench\language | **Glacier DEV** | Gorilla 1.0 | Python 3.8.3 | Ruby 2.7.2 |
|----------------|-----------------|-------------|--------------|------------|
| 21th - Time    | **0.35s**       | 2.98s       | 0.06s        | 0.13s      |
| 21th - CPU     | **1%**          | 80%         | 22%          | 1%         |
| 26th - Time    | **4.97s**       | DNF         | 0.11s        | 0.15s      |
| 26th - CPU     | **1%**          | DNF         | 14%          | 11%        |

As you can see, compared to its brother Gorilla, Glacier is a LOT faster.

### Improvement Areas

- GC Speed

- Use more CPU

- Variable Definition Speed

- Use jumping for functions

- Further reduce object copying/cloning

### Credits:

Glacier cannot grow without people finding and fixing bugs!

- [Just For Fun](https://github.com/techguy940) for finding out bugs in examples/fizzbuzz.glc
