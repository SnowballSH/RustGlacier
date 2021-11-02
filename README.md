## The Glacier Programming Language

Glacier is an imperative programming language continued from [Gorilla](https://github.com/SnowballSH/Gorilla).

### Features

- Elegant and Clean

- Prefect for Code Golfing

- Optimized Speed

### Example: FizzBuzz

```ruby
fn fizzbuzz(from, to)
    i = from
    while i <= to
        print(
            if i % 15 == 0
                "FizzBuzz"
            else:if i % 3 == 0
                "Fizz"
            else:if i % 5 == 0
                "Buzz"
            else: i
        )
        i = i + 1
    end
end
```

### Example: Fibonacci Number

```ruby
fn F(i)
    if i <= 1: i
    else: F(i - 1) + F(i - 2)
end
```

### Benchmark

- Fibonacci

| bench\language | **Glacier DEV** | Gorilla 1.0 | Ruby 2.7.2 |
|----------------|-----------------|-------------|------------|
| 21th - Time    | **0.05s**       | 2.98s       | 0.13s      |
| 26th - Time    | **0.19s**       | DNF         | 0.15s      |
| 30th - Time    | **1.07s**       | DNF         | 0.27s      |
| 33th - Time    | **4.55s**       | DNF         | 0.58s      |

Compared to its brother Gorilla, Glacier is a LOT faster.

### Improvement Areas

- [ ] GC Speed

- [x] Variable Definition Speed

- [x] Use jumping for functions

- [ ] Further reduce object copying/cloning

### Credits:

Glacier cannot grow without people finding and fixing bugs!

- [Just For Fun](https://github.com/techguy940) for finding out bugs in examples/fizzbuzz.glc
