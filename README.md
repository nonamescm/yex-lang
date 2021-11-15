# Yex

## Contents:

  * [About](#about)
  * [How to](#how-to)
    * [Hello World](#hello-world)
    * [Variables](#variables)
    * [Functions](#functions)
  * [Contributing](#contributing)

## About

Yex is a functional scripting language written in rust. <!--TODO: More information-->

## How to

### Hello World

```ml
puts("Hello, World!")
```

### Variables

Bind is made using the `val ... in` constructor. Like so:

```
puts(
  val how = "how "
    in val are = "are "
      in val you = "you" in how + are + you
)
```

### Functions

Functions are created using the `fun` keyword, like:

```
val my_func = (fun n -> n * n) in puts(my_func(40))
```

## Contributing
  * Open a issue if you find any bug
  * Submit a PR if you want to implement a new feature
