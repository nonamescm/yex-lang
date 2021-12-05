# Yex

## Contents:

  * [About](#about)
  * [How to](#how-to)
    * [Hello World](#hello-world)
    * [Variables](#variables)
    * [Functions](#functions)
      * [Named](#named-functions)
      * [Anonymous](#anonymous-functions)
	* [Lists](#lists)
  * [TODO](#todo)
  * [Contributing](#contributing)

## About

Yex is a functional scripting language written in rust. <!--TODO: More information-->

## How to

### Hello World

```ml
puts("Hello, World!")
```

### Variables

Bind is made using the `let ... in` constructor. Like so:

```ml
puts(
  let how = "how "
  in let are = "are "
  in let you = "you"
  in how + are + you
)
```

### Functions

#### Named Functions

Functions are created using the `let` keyword, like:

```ml
let say_hello name =
  puts("Hello " + name)
end

let _ = say_hello("foo")
```

#### Anonymous Functions

You can create anonymous functions using the `fn` keyword.

```elixir
puts((fn n = n * n)(20))
```

### Lists

Lists are created using `[]` like in most other languages:

```ml
[1, 2, 3]
```

## TODO
  * [ ] Closures
  * [ ] Garbage collection
  * [x] Lists
  * [ ] Modules

## Contributing
  * Open a issue if you find any bug
  * Submit a PR if you want to implement a new feature
