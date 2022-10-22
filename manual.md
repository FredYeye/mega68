# mega68

A Motorola 68000 assembler, in the early stages of development. This document is also in the early stages!

---

## Usage

```
mega68 [in_file] [out_file]

-----

[in_file]  | path to file to assemble. If none is specified, "code.asm" will be used.

[out_file] | path to where to create assembled file. If none is specified, the in_file name will be used, adding or replacing an existing file extension with ".bin".
```

---

## Number Literals

Literals can be in decimal, hexadecimal and binary. Hex values use the `0x` prefix and binary uses `0b`. Negative values are also accepted (for decimal only).

---

## Formatting

Comments are defined by prepending a comment with a semicolon `;`.

---

## Labels

Labels are a convenient way to refer to specific offsets of the code. They are created by appending a colon `:` to a name.
As of right now, almost anything that isn't parsed as an opcode / value / etc is accepted as a label. This is likely to change at some point.

### Sub labels

Sub labels are useful when you want to use commonly used names multiple times in different subroutines.
They are created by prefixing a label with a dot `.`. Internally, they are stored as "label.sublabel" and can be
referenced as such from anywhere in the code.

---

## Defines

Defines are created by prepending a name with an exclamation mark `!`, followed by an equals sign and a value: "!test = 0x1234".

---

## Data

The following commands are used for defining binary data:
```
d08
d16
d24
d32
d64
```
where the number represents the bit count per value. Values are defined by adding a comma separated list of values
after any of the above commands. The values will be stored in big-endian format.
Note: if `d08` or `d24` define an uneven amount of data in bytes, a padding zero byte will be added.
