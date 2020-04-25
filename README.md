# Neotronian

A very simple scripting language. Each line can be parsed in isolation and stored as compressed tokens. This makes it more memory efficient than Python or Lua.

```
# We have functions
fn foo(x)
    # Simple logical expressions
    if bar(x) > 0
        return 0
    end
    # We have dynamic typing
    let z = x + 1
    # We have hashmaps
    let m = map()
    m.key = z
    return z
end
```

## Licence

This Rust-language intepreter for the Neotronian language is licensed under the GPL v3.

The language specification is placed in the public domain.
