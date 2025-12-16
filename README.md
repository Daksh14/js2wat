# js2wat

A basic js to wat compiler that compiles a subset of javascript to the WAT format which is an intermediate wasm format following the spec
https://webassembly.github.io/spec/core/text/index.html

## Dependecies

This project only uses `clap` for command line interface, other than that, there are 0 dependencies with a hand written lexer, parser and
code generator

## Usage

Compile basic programs inside the `programs` folder
 
```
cargo r -- --path=programs/gcd.js

wasmtime run --invoke gcd output.wat 48 18

6
```

or use the exported _start function

```
wasmtime output.wat --invoke _start

6
```

## Extra program

Try recursive_gcd.js

```js
function gcd(a, b) {
  if (b == 0) return a;
  return gcd(b, a % b);
}
```

## Bonus: Basic dead code folding inside top level scope in functions

```
cargo r -- --path=programs/dead_code.js

cat output.wat 
```

The dead code elimination pass is very basic, it keep tracks of usages of variables
but doesn't make a dependency graph, for complete dead code elimintaion a dependency graph
implementation is needed, this is a fast approach to dead code elimination that I did using a simple
pass and symbol table.

## Limitations

If I had more time I will try to implement these features

1. Does not support complex binary expressions like `n * fact(n)` or `n + 1 + 2`
2. Does not support complex return values like `n + 2 + 3`
3. All functions only return i32
4. No in-depth intermediate analysis for optimizations like tail call, provenance, etc.
5. No symbol call table to detect wrong variable names or function calls (should be easy to implement)
6. Dead code elimination with dependency inclusion
6. Parser is a bit janky and can break on edge cases

## Testing

There are unit tests for the lexer in the lexer module, and parser and codegen module.

No integration tests yet.

**NOTE: Ideally we can import wasmtime as a dev dependency and run the generated code and directly check the output of programs in the integration tests!**


