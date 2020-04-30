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
    # All assignments use the `let` keyword
    let z = x + 1
    # We have hashmaps
    let m = map()
    let m.key1 = z
    let m[ "key2" ] = "hi"
    # We have a special map called `globals`, which is always in scope
    let globals.x = x
    # We have vectors, which can contain different types of value
    let v = vec()
    push( v, 1 )
    push( v, format( "Hi {}", z ) )
    push( v, m )
    # We have BASIC style for loops
    for idx = 0 to len( v ) - 1
        # Variables have block scope
        let y = idx + 1
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
```

## Types and Variables

A variable can be of the following types:

* Scalars:
  * Integer (signed, 32-bit)
  * Float (32-bit)
  * String (UTF-8 encoded)
  * Boolean (true or false)
  * Nil
* Collections:
  * Vector (an ordered collection of values of any type)
  * Map (an associative array, or dictionary, where the keys are String, the
    values any type)

To save memory, internally a `String` distinguishes between a pointer to a
string-literal in the program source, and a heap-allocated string which has
been created during program execution (e.g. with the `format` function).

The language is dynamically typed, so a variable remembers at run time both
its type and its value. Scalars are passed to functions by value, and have
Copy semantics. Collections are passed by reference, and have
reference-counting semantics.

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

### open(filename, mode)

Open a file. Returns an integer file handle, or Nil if there was an error.

### close(handle)

Closes a previously opened file.

### read(handle, length)

Returns bytes from a file as a Vec.

### write(handle, vec)

Writes bytes to a file.

### seek(filename, whence, offset)

Seeks to a byte offset in the file.

### opendir(directory)

Open a directory

### readdir(handle)

Read a directory entry

### closedir(handle)

Closes a directory.

### stat(filename)

Get stats for a file as a Map.

### lasterror()

Get the most recent error code from the OS.

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
