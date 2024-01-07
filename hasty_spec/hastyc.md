# Hasty Compiler Specification

Hasty language is a modern programming language that compiles to JVM bytecode, but more targets are possible in the future.

## Hasty compiler

`hastyc` - the compiler for hasty programming language - is mostly inspired by the architecture of rustc compiler. This means that it uses 4 stages:

- First, we generate the AST, which is a one-to-one representation of the source code. It comes straight from the parser.
- Second, we lower AST to HIR (High-level Intermediate Representation), which itself comes into multiple stages, which are described later on.
- Last step before generating the target bytecode is HIR to MIR (Mid-level Intermediate Representation) lowering, this form contains all information necessary to create the bytecode, however it is there to provide multi-target compilation and some target-specific optimizations.
- Now, finally, we can generate the bytecode from all the information that MIR provides.

`hastyc` compiler utilizes concept called "demand-driven compilation", this means that it uses the minimum required amount of normal passes and the remaining information is computed only when required by the compiler.

### AST Generation

This is the simplest operation in terms of concept. Firstly, the code is tokenized into token streams, this means that when provided with the following code:

```
fn increment(_ a: i32) {
    a + 1
}
```

This will become the following tokens: `Keyword(fn)`, `Ident(increment)`, `LParen`, `Underscore`, `Ident(a)`, `Colon`, `Ident(i32)`, `RParen`, `LBrace`, `Ident(a)`, `Plus`, `ILiteral(1)`, `RBrace`.
This is done, because for the parser, operating on the whole "words" is easier than working on single letters, just like in the natural language.

After the tokens are generated, we come the the parsing stage, where we finally generate the AST. This process is a bit complicated, so we won't get into details with it. All you need to know is that for example above, parser would yield the following AST:

```
Item(Fn (name: increment, arguments: [_ a: i32]) {
    BinaryOperation(addition) {
        Path(a)
        ILiteral(1)
    }
})
```

Also worth noting is that at this point the compiler has no information about variables or types, so `a` and `i32` are just identifiers that represent... something.

### Lowering to HIR

Now we come to the HIR lowering, HIR initially contains all information about name references to allow for type inference and constant resolution. However, this information needs to come from somewhere, so here comes the AST to HIR lowering stage.

This stage works like typical compiler would - It runs multiple passes.

#### 1. Import resolution

First, the compiler needs to know where the types even could be defined, so we start with import resolution stage. This stage is responsible for creating the module tree, that contains all information about possible "routes".

#### 2. Name resolution

At this stage, the resolver tries to work out all the identifiers that are used in the program, this means that after we attempt to reference a variable called for example `hello`, the resolver would try to check where this variable is defined.

Typical approach is to use scopes to limit where the variable can be accessed and where it cannot. This approach would also work here, but it is much easier to use concept from `rustc` called ribs, it works by creating a stack, where every change in what is available is pushed onto the stack as a rib.

To work out these names, we first need to go over the whole module's AST and create a rib for what we call `items`, these are top-level things, that are available everywhere in the module, for example structs and functions are items. In the example that we are going with from the beginning, the only item would be the `increment` function. After we worked out these names, we can get into every one of them and resolve names for its bodies. The example of such resolution in our example would look like the following:

```
// We start with knowing all the items, so let's say they are stored in a Rib(0).

fn increment(_ a: i32) { // Push new Rib(1) onto the stack.
    // `a` is known in Rib(1) from the function signature

    a + 1 // We traverse the stack top-to-bottom until we see `a`
          // and we mark what `Path(a)` refers to.
} // Pop Rib(1) from the stack.
```

#### 3. Basic type inference

This step is really simple as it just computes the types of primitives and marks unknown types as needing inference.

#### 4. Simplifications

As we now know which name referes to what, we can try to simplify our AST. This step allows us to remove ambiguities, where multiple AST's can mean exactly the same. Simplifications that are done are:

- Replace `for` and `while` loops with generic `loop` expression.
- Convert `guard` statements to early return `if` statements.
- Convert `getter` and `setter` into standard method calls.
- Convert AST node id's to more meaningful ones, that contain package from which the node comes from.

More simplifications may be added later on when the compiler is implemented.

### HIR to MIR lowering

At this point there are no specific order at which everthing happens as the demand-driven compilation method kicks in, for example, when we need to know the type of `a` in our `a + 1`, we call `get_type` query on it, it is only then that the compiler computes that type from the initial information that it has.

**Note on type inference** \
During compilation some types, like integer type of our `1` is marked as `InferedInt`, because we don't know whether it is a 32-bit or 64-bit integer. When we try to compute the type of `a + 1` or `1 + a` this type would fall to the specific one, like 32-bit integer in our example, but sometimes it'd will be left unknown, where it will finally fall back to default for integer, which is also 32-bit one.

There are of course many more queries, but these are unnecessary for this explanation.

### MIR to target bytecode

This is the final stage, where the MIR is compiled into JVM bytecode, as this stage already has all necessary information and is pretty simple, It should be possible to support other targets in the future.
