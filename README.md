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

Note that Glacier is Work-in-Progress, so this example will change rapidly with the introduction of subroutines and
suffix operators. (i++)

### Credits:

Glacier cannot grow without people finding and fixing bugs!

- [Just For Fun](https://github.com/techguy940) for finding out bugs in examples/fizzbuzz.glc
