# Hasty programming language

Hasty programming language is a modern language that compiles to JVM bytecode. It's syntax is inspired by languages like `rust` and `swift`.

## Keywords

At the core of every programming language there are keywords, they allow us to tell the programming language that something is a structure or a function, without the need to have a specific one character symbol for them. In Hasty, there are following keywords:

- `fn` - Function definition.
- `if`, `else` - Conditional flow.
- `true`, `false` - Boolean literals.
- `while`, `for`, `in`, `loop`, - Repeating code segments.
- `break`, `continue` - Early break or next iteration.
- `return` - Function return.
- `self`, `Self` - Refers to the current class / structure in implementation.
- `let` - Defines new variable.
- `nil` - Sometimes called `null` in other languages, defines `nothing` type
- `guard` - Inverse of an if statement, that early returns from the block.
- `pub`, `const`, `static` - Modifiers.
- `import`, `as` - Imports from other modules.
- `module` - Module definition.
- `super`, `pkg` - Used for paths.
- `match` - More advanced switch.
- `struct`, `trait`, `impl`, `enum` - Data structures.
- `getter`, `setter` - Getters and setters for data structures.
- `override` - Operator overrides.
- `where` - Generic constraints

## Language basics

### Modules

Hasty language is divided into packages and modules, every package is like a `dependency` of our source code, and every module is it's own internal namespace. For example, when we define the structure like this:

```
module hello {
    module world {
        pub fn my_function() {}
    }
}

// Point where we want to reference the function
```

and we want to reference `my_function`, we can either use fully qualified path like `hello::world::my_function()`, or use import statement to import the function first `import hello::world::my_function` to make the function available in our current module. Worth noting is that child module is separated from it's parent by the `super` keyword, and can reference the main package by using `pkg` keyword.

### Functions

Programming languages main building block is a function, these are defined in Hasty using the `fn` keyword.

Each function can be prefixed with a modifiers such as `pub` to make the function available from outside the current module, has some arguments, return type and a body.

Function arguments are not like in many other languages as Hasty uses named arguments by default, so if you define `fn greet(name: String)` you would need to call it using `greet(name: "Developer")`. This is done mainly for readibility purposes, but is not always better, as it would be bad if you had to write `print(text: "Hello world!")` every time you wanted to print something. For this purpose function arguments have the following structure: `[usage_name] body_name: type`, where `usage_name` can be ommited to use `body_name`, and can be set to `_` to become ordered argument. As this may be hard to understand, here are a few examples:

```
fn greet(name: String) { println("Hello {name}") }
greet(name: "Developer");
```

```
fn greet(name text: String) { println("Hello {text}") }
greet(name: "Developer");
```

```
fn greet(_ name: String) { println("Hello {name}") }
greet("Developer")
```

Just like in Rust programming language last statement in the block is it's return value, so instead of writing `fn add(_ a: i32, _ b: i32) { return a + b; }` we can write `fn add(_ a: i32, _ b: i32) { a + b }`. The return type for these functions will be automatically infered, however sometimes you may want to define it yourself, when you for example use a trait as return type, this can be done by using `-> type` between arguments and function body.

### Data structures

What would a programming language without data structures look like? I actually don't care because Hasty has them.

#### Structs

Basic data structure in hasty is a struct, it is defined using `struct` keyword followed by a name and fields, structs CANNOT contain methods themselves, but they can be defined using `impl` blocks, so if you want to define struct called `User` with a method `greet`, you would do it like so:

```
pub struct User {
    name: String
}

impl User {
    pub fn new(username: String) {
        User {
            name: username
        }
    }

    pub fn greet(self) {
        println("Hello {self.name}")
    }
}
```

and to create and greet new user, you can now use `User::new(username: "Developer").greet()`;

This syntax is really similar to rust, so if you've ever used it, you would definately like it here.

Structure implementations can only be defined in the current module! Everything else should be defined using traits.

#### Traits

traits are like interfaces that can be defined on existing structures, so you can for example extend existing `User` with rename method.

```
trait UserExt {
    fn rename(self, username: String);
}

impl UserExt for User {
    fn rename(self, username: String) {
        self.name = username;
    }
}
```

#### Custom getters and setters

sometimes you want to have something happen when you get or set a value, you can do this using `getter` and `setter` modifiers on functions in implementations. They cannot however be defined on traits.

```
impl User {
    pub getter fn name(self) -> String {
        self.name + "something"
    }

    // Setters only accept one ordered argument, anything else
    // will throw an error
    pub setter fn name(self, _ new_name: String) -> String {
        self.name = new_name;
    }
}
```

#### Operator overrides

operators can be overriden for user-defined types using `override` modifier, this modifier takes operator to override in parentheses after itself. Compiler will then replace all calls to that operator for the type with method calls.

```
impl User {
    // Binary operator overrides take one ordered argument
    // and unary operator overrides take none.
    pub override(+) fn add_string(self, _ suffix: String) -> User {
        User {
            name: self.name + suffix
        }
    }
}
```

To avoid readability problems, operator overrides can only be public.

#### Enumeration types

Common data structure in programming is an enum. These are structures that can be one of some known set. Example of such data structure we can create could be `AnimalKind`, let's say this can be either `Cat` or a `Dog`, we would define such data structure in Hasty using the following syntax:

```
enum AnimalKind {
    Cat, Dog
}
```

In Hasty enums can hold some data, for example `Option<T>` can store data of type `T` when it's something. Definition of such structure would be:

```
enum Option<T> {
    Some(T),
    None
}
```

When the type is known, instead of writing `AnimalKind::Dog` you can just use `:Dog` syntax and the compiler will work out what this `Dog` refers to.

### Conditional flow

as in many other programming languages, you can define that code should run only when some condition is met. This is exactly what control flow does.

Basics are no other then in rust, you have `if` and `else` keywords followed by condition and a block.

One thing that changes is new `guard` keyword, that returns from the block when condition is not met, for example if you want your function to early exit if user age is <18 you can use one of the following:

```
fn protected_by_age(age: i32) {
    guard age >= 18; // Default behavior is to return with default value

    // ...
}
```

```
fn protected_by_age(age: i32) {
    guard age >= 18 else { return }

    // ...
}
```

```
fn protected_by_age(age: i32) {
    if age < 18 { return }

    // ...
}
```

It may seem as if this syntax is unnecessary, and the truth is that it really is, it is even changed by the compiler to standard if, but it can allow the programmer to think of when his function should run instead of when it shouldn't.

#### Match expression

Hasty has a thing called match expression instead of more common switch. This expression can be used for a bit more complicated things, most common use case of match expression is to check whether some value is certain enum variant:

```
// Let's say `value` has type `Option<T>`
match value {
    Some(x) => println(x),
    None => println("None")
}
```

Other match patterns are subject to change in the future so they won't be described here.

### Loops

There are three types of loops in Hasty: `for`, `while`, and `loop`. Actually, the compiler only knows one of them: `loop`, which is the most basic, infinitely repeating, loop. For and while loops are converted to loop when your code is compiled.

#### For loop

For loop is probably the most common one in your programs, it allows you to iterate over any iterator, this is not common, as typical for loop allows you to specify init, condition, and increment expressions.

Most common use case would be to iterate over some range of numbers, Hasty has special notation for such range, and It looks like this: `0..5` (repeat from 0 to 4) or `0..=5` (repeat from 0 to 5).

```
for i in 0..5 {
    println(i); // Prints 0, 1, 2, 3, 4
}
```

#### While loop

While loops repeat until the condition returns false, their syntax is simple:

```
while condition {
    // ...
}
```

#### Loop loop

This is really the simplest loop, it does not take any arguments, it can be used like the following:

```
loop {
    // ...
}
```

### Generic types

functions, traits, and structures can be generic, this means that compiler will generate code for every typed variant of it, generic types are defined using `<A, B, C, ...>` and can be constrained using `where` keyword. The convention is to use `<T>` if you have one generic type, but the name can be anything. I won't describe generic types here, but I know how they'll be implemented in hastyc.
