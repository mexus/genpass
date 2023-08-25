# genpass

A super simple password generator.

## Usage

## Examples

To generate a password without any special symbols, but allow any of `!@#$%-` to
appear though:

```
$ genpass --no-special --allow '!@#$%-'
pJd!q$IdrQ49P2g9n2CGlzP0
```

Use `-v/--verbose` flags to see symbols groups:

```
$ genpass --no-special --allow '!@#$%-' -v
DEBUG genpass: Add symbols ABCDEFGHIJKLMNOPQRSTUVWXYZ
DEBUG genpass: Add symbols abcdefghijklmnopqrstuvwxyz
DEBUG genpass: Add symbols 0123456789
DEBUG genpass: Add symbols !#$%-@
DEBUG genpass: Symbols to use: #$%-0123456789@ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz
FnDnoO1exy6HTdQquLQWw6d0
```

To put the generated password to the clipboard:

```
$ genpass --no-special --allow '!@#$%-' -c
$ xclip -selection clipboard -o
ZCiE10G@ZtP5N#s6s8YVyMvM
```

## Explanation

When run without arguments, some groups of symbols are enabled by default. You
can explicitly disable any of them by providing a `--no-...` argument, e.g.
`--no-latin`. Please run with `--help` for the full list of arguments.

Apart from the built-in symbol groups you can also provide your own sets via the
`-a/--allow` argument, followed by a sequence of characters. You can repeat the
argument multiple times for multiple groups. Empty sets are not allowed!

The application also provides the ability to remove some symbols from the list
by utilizing the `-d/--deny` argument, followed by a sequence of characters. You
can repeat the argument multiple times for multiple groups. Empty sets are not
allowed!

The final list of symbols to generate a password with is built in the following
order:

- Non-disabled built-in groups of symbols are added to the list.
- All the additionally allowed symbols with `-a/--allow` argument(s) are added
  to the list.
- Denied with `-d/--deny` argument symbols are removed from the list.

If the resulting symbols list is empty, the program will refuse to generate a
password and terminate with an error code.

Length of the generated password can be adjusted by simply adding the desired
positive amount of symbols to the command line, no flags needed: `genpass 12`
will generated a password of 12 unicode scalar values.

## Randomness

The application doesn't do anything special about fetching the random values, it
completely relies on the random generation (`getrandom`) of your operation
system, see more at
[`OsRng`](https://docs.rs/rand/latest/rand/rngs/struct.OsRng.html).

## Clipboard

While it might be not 100% safe, you might want to put the generated password
directly into the clipboard.

Here's the `-c/--copy` flag for you.

On Linux it will cause the process to fork and wait in the background until somebody else changes the contents of the clipboard. *say hello to X11*

On other platforms the generated password is simply put to the clipboard.

The flag also disables output of the password to stdout.
