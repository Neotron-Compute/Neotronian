# Neotronian

A very simple scripting language. Each line can be parsed in isolation and
stored as compressed tokens. This makes it more memory efficient than Python
or Lua, while it has more modern features than BASIC (like maps).

```bash
# We have functions
fn foo( x )
    # Simple logical expressions
    if bar( x ) > 0
        return 0
    end
    # We have dynamic typing
    # Variables are created with the `var` statement
    # This creates a new variable in this scope, shadowing any variables
    # with the same name created in a different (earlier) scope
    var z = x + 1
    # We update an existing variable with the `let` statement
    let z = z + 1
    # We have hashmaps
    var m = map()
    m.set("key2", "hi")
    # If we read a key that doesn't exist, we get `nil`
    print("getting unknown key returns {}", m.get("unknown"))
    # We have a special map called `globals`, which is always in scope
    globals.set("x", x)
    # We have vectors, which are growable ordered lists of data
    var v = vec()
    # We have 'method' calls, so this...
    v.push( 1 )
    # ...is the same as
    vector.push(v, 1)
    # Note that function arguments are always copied, but vectors and hashmaps are
    # reference counted, and so the function gets a copy of the reference. This
    # is similar to Python.
    v.push( format( "Hi {}", z ) )
    v.push( m )
    # We have BASIC style for loops
    for idx = 0 to len( v ) - 1
        # Variables have block scope
        var y = idx + 1
        # We can print things, using `{}` to insert values.
        # We can also index a vec with `[]`, and we can get the type
        # of a variable with the `type` function.
        print( "v[{}] = {} ({})", idx, v[ idx ], type( v[ idx ] ) )
        if v[ idx ] == 1
            # We can break out of loops too
            break
        end
    end
    # The variable y no longer exists here
    return z
end

# We also have modules
module baz
    fn foo()
        # Accessed as baz.foo()
    end
end

# We even have classes
# Internally each class has a Hashmap (for class variables), and each Object has a Hashmap (for object members)
class Point
   var classVariable = "Some String"

   fn __init__(self, x, y)
      # This sets the `x` member of the new object's internal hashmap
      self.x = x
      self.y = y
   end

   # This is called automatically when we convert objects of this class to a string
   fn __str__(self)
      var s = format("Point(x={},y={}", self.x, self.y)
      return s
   end
end

fn testPoint()
   var m = Point.new(1, 2)
   # Look ups happen in the object dictionary first, then the class dictionary
   print("m = {} ({})", string(m), m.classVariable)
end
```

## Types and Variables

A variable can be of the following types:

* Scalars:
  * Integer (signed, 32-bit)
  * Float (32-bit)
  * String (immutable, UTF-8 encoded)
  * Bytes (immutable ordered collection of 8-bit bytes)
  * Boolean (true or false)
  * Nil
* Collections:
  * Vector (an ordered collection of values of any type)
  * Map (an associative array, or dictionary, where the keys are String, the
    values any type)
  * Classes (a collection of class variables and methods)
  * Objects (an instance of a class, with its own variables)

To save memory, internally a `String` distinguishes between a pointer to a
string-literal in the program source, and a heap-allocated string which has
been created during program execution (e.g. with the `format` function).

The language is dynamically typed, so a variable remembers at run time both
its type and its value. Scalars are passed to functions by value, and have
Copy semantics. Collections are passed by reference, and have
reference-counting semantics (i.e. a copy of the reference is passed).

Variables have block scope (that is inside function, for loop, loop and
if/elsif/else statement blocks) and collections are only freed when they hit a
reference count of zero.

You cannot create global variables. Instead, there is a single global variable
called `globals`, which is a map.

## Editing

Rather than entering plain-text source code into a file and then running a
separate compilation step as part of the script execution (like Python or
JavaScript), Neotrotronian instead is a line-based language. Each line is
entered, parsed and stored in tokenised form, line by line. You can ask the
interface to list the program as it stands, which will convert the tokenised
form back into the canonical source code representation, including
indentation. You can also ask the interface to delete lines, delete a range of
lines, edit an existing line or insert a new line. This minimises the use of
memory and avoids a separate pre-compilation step. It also means that a disk
drive, or indeed any kind of filing system, is not required to use this
language.

When filesystem support is available, programs can be loaded and saved in
their plain-text format, or as tokenised data.

## Valid Statements

### if <expr>

The block is entered if `bool(<expr>)` is true.

### elif <expr>

An optional extra checked block for an `if` statement.

### else

The optional final block for an `if` statement.

### for <var> = <expr1> to <expr2> [ step <expr> ]

Starts a finite loop.

### loop

Starts an infinite loop. Same as `while true`

### break

Exits out of the innermost `for` or `loop` block and moves past the
corresponding `end`.

### let <var> = <expr>

Assigns the value of `<expr>` to the pre-existing variable called `<var>`.

### var <var> [ = <expr> ]

Creates a new variable called `var`. Defaults to `nil` if `<expr>` isn't supplied.

### fn <name>([ <param> [, <param> ]+ [,  ... ] ])

Start a new function block. You can have zero or more parameters and an
optional final "..." which means any number of parameters are allow - these
are bundled into a Vec called `args`.

### end

Closes out the most recent `if`, `loop`, `module` or `fn` block.

### module

Starts a new module. A module is really just a namespace for functions. It can contain further modules.

### expression-statement

Any line which doesn't start with one of the above, is treated as an
expression-statement. Typically used to call a function where you don't care
about the return value. Broadly equivalent to `let _ = <expr>`, except `_`
isn't a thing.

## Standard Library

The following functions are always in scope.

### print(string, ...)

Print takes a string, then an arbitrary number of arguments. The string is
taken as a format string, and any `{}` sub-strings are replaced with the
following arguments, in order. Any non-string argument is converted to a
string for display using the `string()` function.

### println(string, ...)

Like `print`, but adds a new-line character at the end.

### moveto(x, y)

Moves the text cursor to column x and row y.

### cursor(x)

If `bool(x)` is true, the cursor is enabled. Otherwise, the cursor is disabled.

### rows()

Returns the number of rows of text on the screen as an integer. Typically 25.

### cols()

Returns the number of columns of text on the screen as an integer. Typically 40, 48 or 80.

### foreground(c)

Sets the foreground colour (i.e. the text colour) to the given value.

* Black: 0 or "black"
* Blue: 1 or "blue"
* Green: 2 or "green"
* Cyan: 3 or "cyan"
* Red: 4 or "red"
* Magenta: 5 or "magenta"
* Yellow: 6 or "yellow"
* White: 7 or "white"

### background(c)

Sets the background colour. See `foreground` for the list of colours.

### string(any)

Converts a variable of any type to a string. If `any` is a collection, the
collections contents are rendered into the string using `[x, y]` or
`{key: value}` syntax familar from Python or JavaScript.

### int(any)

Converts integers, strings, booleans and floats to their integer equivalent.
Nil, Vec and Map convert to 0. The prefixes `0x` and `0b` on a String change
the base to 16 or 2 respectively.

### float(any)

Converts integers, strings, booleans and floats to their floating-point equivalent. Nil, Vecs and
Maps convert to 0.0.

### bool(any)

* Integers and Floats are true if they are non-zero.
* Strings, Vecs and Maps convert to false if they are empty (i.e. have length zero), and true otherwise.
* Nil is false.

### len(any)

* Returns the length, in bytes, of a String
* Returns the length, of a Vec
* Returns the number of values in a Map
* The length of a non-String scalar is zero.

### keys(map)

Returns all the keys in a Map as a Vec.

NB: We might make this more efficient in future if we get _iterator_ support.

### delete(collection, key_or_idx)

Removes a value from a collection (and returns it). If the collection is a
Map, `key_or_idx` should be a String. If the collection is a Vec, then
`key_or_idx` should be an Integer.

### push(vec, value)

Adds a new value to the end of a Vec.

### pop(vec, value)

Adds a new value to the end of a Vec.

### format(string, ...)

Like print, but returns a heap-allocated String.

### sin, cos, tan, etc

The usual selection of mathematical functions are available, which take
floating point values (or integers, which are converted to floats
automatically). The trigonometric functions take angles in Radians.

### mode(x)

Change the screen mode. Support for various text/graphics modes depends on your OS.

### width() and height()

Get the width and height of a bitmap display, in pixels.

### plot(x, y)

Draw a point on a bitmap screen at position x, y in the current foreground
colour. Position 0, 0 is the top left of the screen.

### move(x, y)

Move to position x, y without drawing anything.

### draw(x, y)

Draw a line from the current x, y position to the given x, y position, in the foreground colour.

### rectangle(x1, y1, x2, y2, fill=0)

Draw (and optionally fill in) a rectangle.

### circle(x, y, r, arc_start=0, arc_end=360, fill=0)

Draw (and optionally) fill a circle, or segment of a circle, with the given centre point and radius.

### circle(x, y, r1, r2, arc_start=0, arc_end=360, fill=0)

Draw (and optionally) fill an ellipse, or segment of a ellipse, with the given centre point, major radius and minor radius.

### fill(x, y)

Perform a flood fill at the given x, y position. The flood fill will move
outwards until it reaches a pixel of a different colour to that at the given
x, y position. Only the four compass points are checked (up, down, left and
right), not the four diagonals. The screen is filled with the current
foreground colour.

### sound(channel, waveform, volume, duration)

Makes a sound, on the given channel (typically 1..3), using the given waveform
(typically "square", "sine" or "triangle"), at the given volume (0..255) for a
given duration (in 60 Hz frame ticks).

### sleep(ticks)

Sleep for the given number of 60 Hz frame ticks.

### wfvbi()

Wait for the next vertical-blanking interval, when it should be safe to draw
on the screen without tearing.

### getclock()

Get the current POSIX time as a float. The time is in local-time, and
time-zones are an OS matter.

### getdatetime()

Get the current Gregorian calendar date/time as a Map. The time is in
local-time, and time-zones are an OS matter.

* year (e.g. 2020)
* month (1..12)
* day (1..31)
* hour (0..23)
* minute (0..59)
* second (0..60)
* dow (0..6 where 0 is Monday)

### setdatetime(dt)

Set the current Gregorian calendar date/time, using the given Map. It must
have the following Integer values:

* year (e.g. 2020)
* month (1..12)
* day (1..31)
* hour (0..23)
* minute (0..59)
* second (0..60)

### input(prompt)

Prints the prompt string, then reads a string from standard-input until Enter
is pressed or a control character (Ctrl-A through Ctrl-Z) is entered.

### readkey()

Returns a character (as a single character string) if a key has been pressed
and a character is in standard-input buffer, otherwise returns Nil.

### rawmode(x)

If bool(x) is true, puts the console in `raw` mode, otherwise leaves `raw` mode. In `raw` mode,
you must read keyboard events with `readevent()` rather than using `readkey()`. This is a much
better mode for writing games.

### readevent()

Read a raw keyboard event. If an event is available, it is returned as an integer, otherwise returns Nil.

### open(filename, mode)

Open a file. Returns an integer file handle, or Nil if there was an error.

### close(handle)

Closes a previously opened file.

### read(handle, length)

Reads bytes from a file, at the current offset, as a Vec of Integers, each 0..255.

### readstring(handle, length)

Reads UTF-8 bytes from a file and returns a String. If the bytes aren't valid
UTF-8, you get Nil.

### iseof(handle)

Returns True if the current offset is at the end of the file.

### readline(handle)

Reads UTF-8 bytes from a file until a new-line character (or EOF) is found,
and returns a String.

### write(handle, data)

Writes bytes to a file. If data is a Vec, every item in the Vec is converted
to an integer and then only the bottom 8 bits written. If data is a String,
the String is written as UTF-8 encoded bytes. If any other type is provided,
it is converted to a String first.

### seek(handle, offset, whence)

Seeks to a byte offset in the file. Whence should be the string "set", "end"
or "current", and controls how the offset is interpreted (as absolute,
relative to the end of the file, or relative to the current offset). Offset
will be converted to an integer.

### opendir(directory)

Open a directory

### readdir(handle)

Read a directory entry

### closedir(handle)

Closes a directory.

### stat(filename)

Get stats for a file as a Map.

### lasterror()

Get the most recent error code from the OS as a String.

## Constants

The following constants exist everywhere as part of the standard library.

### STDOUT

The file handle for standard output.

### STDIN

The file handle for standard input.

### STDERR

The file handle for standard error.

### PI

The floating point value 3.141592...

## Outstanding Questions

1. Which is better `let x = expr()`, `x = expr()`, or `x := expr()`?
1. Which is better `if x == 1`, or `if x = 1`?
1. Which is better `if expr()`, or `if expr() then`?
1. Should we support tuples?
1. Should you create a Vec with `vec()` or `[]`?
1. Should you create a Map with `map()` or `{}`? 
1. Should we support dot-notation (`my_variable.function()`)? How does that map to a function?
1. Can functions be stored in variables?
1. Do we have lambdas?
1. Do lambdas capture local scope?
1. If so, is that implicit or explicit?
1. Should we distinguish between procedures and functions?
1. Is a bare expression a valid statement?
1. Should we support modules, or prefix stdlib functions with `module_`, or just lump it all together like C and PHP?

## Licence

This Rust-language intepreter for the Neotronian language is licensed under
the GPL v3.

The language specification, this README, and any example programs in this
repository, are available under an MIT licence, Apache-2.0 or under CC0, as
your option.

## Contributions

Any PRs to this repository will only be accepted if they are compatible the
licensing terms above. You will retain the copyright in any contributions, and
must confirm that you have the right to place the contribution under the
licences above.
