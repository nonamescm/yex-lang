# Yex Programming Language

- [What is yex](#what-is-yex)
- [Getting Started](#getting-started)
	- [Interactive Shell](#interactive-shell)
	- [Hello World](#hello-world)
- [Basic Types](#basic-types)
	- [Booleans](#booleans)
	- [Symbols](#symbols)
	- [Strings](#strings)
	- [Lists](#lists)
	- [Structs](#structs)
- [Functions](#functions)
	- [Named](#named)
	- [Anonymous](#anonymous)
	- [The pipe operator](#the-pipe-operator)
- [Modules and traits](#modules-and-traits)
	- [Modules](#modules)
	- [Traits](#traits)

## What is yex?

Yex is a toy programming language, I use it as a playground to try out some new ideas and see how they interact with each other. Therefore, you should not expect any stability from it or rely on it anywhere. Please don't use this in an production envinronment.

## Getting Started

First, make sure both rust and git are installed, then run this command:

```bash
cargo install --git https://github.com/nonamescm/yex-lang.git
```

Yex is installed now, great!

### Interactive Mode

Let's start this tutorial with the REPL, type `yex` on your current terminal session, it will open a shell that evaluates any valid *expression*. So, type some basic code on it:

```scala
yex> "Hello, " + "World!"
"Hello, World!"
yex> 1 + 1
2
```

When you're done, just press `C-c` or `C-d` to exit.

### Hello World

Create a file called `hello.yex` and type the following code in it:

```scala
println("Hello World!")
```

Run it with `yex hello.yex` 

## Basic Types

Yex support a handful of types, some of them are:

```scala
yex> 1.0         # Num (64 bit floating point number)
yex> ""          # Str
yex> []          # List (Cons List)
yex> ()          # Tuple
yex> :sym        # Sym (Compile-time hashed string)
yex> fn() do end # Function
```

### Booleans

Yex supports `true` and `false` as booleans:

```scala
yex> true
true
yex> true == false
false
yex> true != false
true
yex> type(true)
module 'Bool'
```

### Symbols

A symbol is a string hashed at compile-time. They're often useful as a named alternative to strings and numbers in enumerations. You can declare them this way:

```scala
yex> :haskell
:haskell
yex> :elixir
:elixir
yex> :javascript
:javascript
yex> :elixir == :haskell
false
yex> :javascript == :javascript
true
yex> type(:elixir)
Sym
```

### Strings

In yex, strings are always double-quoted and UTF-8 encoded:

```scala
yex> "ƒ"
"ƒ"
yex> "f" == "ƒ"
false
yex> "Hel" + "lo"
"Hello"
yex> "\x49" // the same as "\u0049" or "\U00000049"
"I"
```

the `Str` module contains some useful functions for operating on strings:

```scala
yex> Str#len("Hello")
5
yex> Str#split("Hello, World", ", ")
["Hello", "World"]
yex> Str#toList("foo,bar,baz")
["f", "o", "o", ",", "b", "a", "r", ",", "b", "a", "z"]
yex> Str#get(0, "hi")
"h"
```

### Lists

(Linked) Lists in yex are a data structure just like any other that holds a collection of values of any type.

```scala
yex> [1, "hello", true, :apple, nil]
[1, "hello", true, :apple, nil]
yex> 4 :: [3, 2, 1]
[4, 3, 2, 1]
```

If you need functions that operate on lists, you can use the `List` module.

```scala
yex> let xs = [1, 2, 3]
yex> List#map(fn(x): x * 2, xs)
[2, 4, 6]
yex> List#head(xs)
1
yex> List#tail(xs)
[2, 3]
yex> List#filter(fn(x): x mod 2 == 0, xs)
[2]
yex> List#fold(fn(sum, n): sum + n, 0, xs)
6
yex> List#join(" ", ["how", "are", "you?"])
"how are you?"
yex> List#find(fn(x): x > 4, [3, 4, 5, 6, 7])
5
yex> List#len([1, 2, 3])
3
yex> List#get(0, [1, 2, 3])
1
```

### Structs

Structs are data structures that store key-value pairs, you can create and use them like so:

```scala
yex> let john = %Struct{name: "John", age: 20} // the `Struct` is optional
nil
yex> john
%Struct{name: "John", age: 20}
yex> john.name
"John"
yex> Struct#insert(:country, "Brazil", john)
%Struct{country: "Brazil", age: 20, name: "John"}
yex> Struct#get(:country, john)
nil // john was not mutated by the Struct#insert call
yex> Struct#get(:age, john)
20
yex> Struct#toList(john)
[(:age, 20), (:name, "John")]
```

## Functions

In yex, functions are impure, so they can have side effects and this is not enforced by the compiler in any way. They're also curried by default, so, if you apply a function to less arguments than what it requires, it will return a new function, but with the arguments already applied.

### Named

Named functions can be created with the `def` keyword, they work both at the module and the local scope. To declare one, open a file and type:

```scala
def fibonacci(number) do
	if n <= 1 then
		n
	else
		fib(n - 1) + fib(n - 2)
	end
end

// one liners can also use `:`
def reduce(f, xs): List#fold(f, List#head(xs), List#tail(xs))

fibonacci(25)
|> println() // more on this later
```

you can run this file with `yex file-name.yex`

### Anonymous

Anonymous functions can be created with the `fn` keyword, they can be used in expression contexts as you've seen above in the [Lists](#lists) examples.

Open the repl and type:

```scala
yex> let f = fn(x): x * 2
nil
yex> f
fn(1) // 1 is the function arity
yex> f(2)
4
yex> let f = fn(x) do x * 2 end // they also support `do`
nil
yex> f(2)
4
```

### The pipe operator

As you've seen [here](#named), yex has an operator called `|>`, which takes the value on the left and apply it to the function on the right, so, you can think of it as a reverse function application operator, `expr |> func()` = `func(expr)`. Just some examples:

```scala
yex> println("hello world!")
hello world!
nil
yex> "hello world!" |> println()
hello world!
nil
```

## Modules and traits

Modules and traits are two ways of achieving modularity and polymorfism, and they can be used together.

### Modules

Modules are groups of functions that you consider to have something in common, like the datatype that they operate on, or their domain, you can create a module with the `module` keyword. On a file, type:

```scala
module Person
	struct [:name, :age]
end

%Person{name: "Maria", age: 17}
|> println()
```

### Traits

Traits are a way of specifying the behaviour of any modules which implements it. You can define them the following way:

```scala
trait Enum
	// these methods should be implemented by the type
	def toList(x)
	
	// these methods are provided by the trait
	def map(f, xs): List#map(f, Enum#toList(xs))
	def reduce(f, xs) do
		let xs = Enum#toList(xs)
		List#fold(f, List#head(xs), List#tail(xs))
	end
end

[1, 2, 3]
|> Enum#reduce(fn(sum, x): sum + x)
|> println()

"Hello"
|> Enum#reduce(fn(sum, x): sum + x)
|> println()
```

