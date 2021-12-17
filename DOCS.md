# Learn Yex

## NOTE:
This tutorial assumes that you have previous experience with programming and
already has the yex language installed.

* [Basics](#basics)
  * [Primitives](#Primitives)
    * [Numbers](#Numbers)
    * [Strings](#Strings)
    * [Bool](#Booleans)
    * [Nulls](#Nulls)
  * [Structuring a program](#structuring-a-program)
* [Variables](#variables)
  * [Globals](#globals)
  * [Locals](#locals)
* [Lists](#lists)
  * [Creating lists](#creating-lists)
  * [Operating on lists](#operating-on-lists)
* [Tables](#tables)
  * [Creating tables](#creating-tables)
* [Functions](#Functions)
  * [Creating functions](#creating-functions)
    * [Named functions](#named-functions)
    * [Anonymous functions](#anonymous-functions)
  * [Tail calls](#tail-calls)
  * [Partial application](#partial-application)
* [Control flow](#control-flow)
  * [Conditional execution](#conditional-execution)
    * [If and else](#if-and-else)
  * [Sequential execution](#sequential-execution)
    * [The sequence operator](#the-sequence-operator)
  * [Pipes](#pipes)
  * [Modules](#modules)
    * [The open keyword](#the-open-keyword)
* [Builtin functions](#builtin-functions)

## Basics

### Primitives

Yex has the following primitive types:
  * `fn` - Functions
  * `num` - 64 bits floating-point numbers
  * `str` - Strings
  * `nil` - Null values
  * `sym` - Compile-time hashed strings
  * `bool` - Booleans (true and false)
  * `list` - Singly linked lists

Them all support the equality `==` operator and the `#` len operator.

#### Numbers
Open the repl and start typing:

```ml
yex> type(1)
>> "num"
yex> 2+2
>> 4
yex> 2-2
>> 0
yex> 2*2
>> 4
yex> 2/2
>> 1
```

As you can see, numbers support all the basic math operations, they also support
the xor, shift-left, shift-right, and and or, bitwise operators:

```ml
yex> 2 ^^^ 3
>> 1
yex> 2 >>> 3
>> 0
yex> 2 <<< 3
>> 16
yex> 2 &&& 3
>> 2
yex> 2 ||| 3
>> 3
```

#### Strings

Strings in yex are represented as UTF-8 encoded strings, they support
concatenation, on the repl:

```ml
yex> type("Example")
>> "str"
yex> "Hello"
>> "Hello"
yex> "Hello " + "World"
>> "Hello World"
```

#### Symbols

Symbols in yex are represented as 64 bit unsigned integers. They are created
using `:name` and are hashed at compile time, on the repl:

```ml
yex> type(:symbol)
>> "sym"
yex> :sym
>> :sym
```

They don't support any operators except for comparison.

#### Booleans

Booleans in yex are just `true` and `false`, there is no magic behind the
scenes.

```ml
yex> type(true)
>> "bool"
yex> true
>> true
yex> false
>> false
```

#### Nulls

Null values in yex can be created using the `nil` keyword.

```ml
yex> type(nil)
>> "nil"
yex> nil
>> nil
```

### Structuring a program

A yex program is just a lot of `let`s. There is no way of using expressions in
the top-level and there isn't any main function, so we usually create a
`let _ = ...` to denotate the entry point, since it's going to
be evaluated when the code runs.

But, what is a `let`? Let's see it now.

## Variables

In yex everything is immutable, so, when you assign a variable, you can't change
it's value, (but shadowing is still supported).

### Globals

Global variables are created using the `let` keyword, open a file and type this:

```ml
let number = 42
let _ = puts(number)
```

Run it with the yex binary. It should print 42.

### Locals

Since, everything in yex is an expression, local variables declarations also
need to be one. For this we use the `let name = expression in expression` constructor, which explicity
defines a expression to be runned after the declaration.

Open a file and type:

```ml
let _ =
  let number = 42
  in puts(number)
```

Run it with the yex binary. It should print 42.

You can create multiple locals just using a lot of `let ... in` expression,
like:

```ml
let _ =
  let a = 21
  in let b = 21
  in puts(a + b)
```

And, yes, this prints 42.

## Lists

In yex, lists are data types that let you have a collection of values of divergent types.

### Creating lists

Like in most other languages, lists can be instantiated using brackets, open the
repl and type:

```ml
yex> type([])
>> "list"
yex> [1, "hello", :symbol, [3, 4], true, nil]
>> [1, "hello", :symbol, [3, 4], true, nil]
```

### Operating on lists

Lists support the following operations:
  * `head()` - returns the first element of the list
  * `tail()` - returns the tail of the list, (all elements except for the first)
  * `::` - This is the cons operator, it add a new element at the start of the
    list without mutating it.
  * `#` - Returns the list length

On the repl:

```ml
yex> head([1, 2, 3])
>> 1
yex> tail([1, 2, 3])
>> [2, 3]
yex> 0 :: [1, 2, 3]
>> [0, 1, 2, 3]
yex> #[1, 2, 3]
>> 3
```

## Tables

In yex, tables are implemented as HashMaps.

### Creating tables

Tables can be instantiated using curly braces, open the repl and type:

```ml
yex> type({})
>> "table"
yex> {:key = "value", :other_key = "other value"}
>> {:key = "value", :other_key = "other value"}
yex> {:key = "value", :other_key = "other value"}[:key]
>> "value"
```

## Functions

### Creating functions

#### Named Functions

Named functions are created using the `let` keyword, like variables. Open the
repl and type:

```ml
yex> let mul a b = a * b
>> nil
yex> mul(2, 3)
>> 6
yex> type(mul)
>> "fn"
```

So, let me explain, first, we declare the function `mul`, receiving `a` and `b`
as parameters. After the `=` it specifies the function's body.

#### Anonymous Functions

You can create anonymous functions using the `fn` keyword. Open the repl and
type:

```ml
yex> let mul = fn a b = a * b
>> nil
yex> mul(2, 3)
>> 6
yex> type(mul)
>> "fn"
```

### Tail calls

Tail calls are an specific type of recursion where it just jump to some
instructions before, you can create them using the `become` keyword, like:

```ml
let until_0 num =
  if num == 0 then
    0
  else
    become until_0(num - 1)
```

Tail calls just use a jump instruction, so they are faster than normal recursive
functions. A important detail about tail calls is that they can only be used to
do recursion, they can't call any arbitrary function.

### Partial application

Functions in yex support partial application, or, in other words, you can call a
function with missing arguments and it will return another function.

In the repl, type:

```ml
yex> let mul a b = a * b
>> nil
yex> let double = mul(2) // returns mul() with the `a` argument already applied
>> nil
yex> double(5)
>> 10
```

## Controw flow

### Conditional execution

#### If and else

Yex supports if and else control flow structures, open the repl and type:

```ml
yex> if true then 1 else 2
>> 1
yex> if false then 1 else if true then 2 else 3
>> 2
```

The format itself is just: `ìf condition then expression else expression`, since
if itself is an expression, the `else if` pattern is just a `if` expression
after the `else`.

### The sequence operator

Since everything in yex is an expression, it isn't really a easy thing to run
multiple side-effect-only functions chained, you could probably think about
using let to concat them, but this looks really bad. So, yex provides the `>>`
operator, which ignores the result of a computation and runs the next
computation.

Open a file and type:

```ml
let _ =
  puts("Hello")
  >> puts("World")
```

This should print:


```
Hello
World
```

### Pipes

When working with high-order-functions, you might need to do something like:
`fold(map(rev(...), ...), ...)`, which doesn't look good, so yex provides the
`|>` operator, that allows the use of a pipeline-like flow.

```ml
rev([1, 2, 3])
|> map(fn x = x * 2)
|> fold(fn acc x = acc + x)
// the same as `fold(fn acc x = acc + x, map(fn x = x * 2, rev([1, 2, 3])))`
```

## Modules

NOTE: Modules aren't yet implemented, there are going to be changes in the
import system in the future.

### The open keyword

You can import from a file using the `open` keyword, a simple example would be:

```
// FILE: a.yex
let greets p = puts("Hello " + p + "!")
```

```
// FILE: b.yex
open "./a.yex"
let _ = greets("nonamescm") // prints "Hello nonamescm!"
```

Make sure the two files are in the same directory and run b.yex.

## Builtin functions


|    Name   |                        Description                       |
|:---------:|:--------------------------------------------------------:|
| `print`   | prints a value without adding the new line               |
| `puts`    | prints a value with a newline at the end                 |
| `str`     | converts a value to string                               |
| `input`   | Reads the input from the console                         |
| `head`    | Returns the first element of a list                      |
| `tail`    | Returns the tail of a list                               |
| `type`    | Returns the string representation of value type's        |
| `inspect` | Returns the intern representation of the value           |
| `getargs` | Returns the args that this program was started with      |
| `fread`   | Read's a file content                                    |
| `fwrite`  | Write to a file                                          |
| `remove`  | delete a file                                            |
| `exists`  | check if a file exists                                   |
| `system`  | run a shell command, returning the stdout and the stderr |
| `getenv`  | get an environment variable                              |
| `setenv`  | Set an environment variable, `setenv("cool", "true")`    |
| `split`   | Splits a string                                          |

