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
```
d16 36274      ;decimal
d16 -6         ;decimal (negative)
d32 0xF3576ACE ;hex
d08 0b10001001 ;binary
```
---

## Formatting

Comments are defined by prepending a comment with a semicolon.
```
;this is a comment
move.l D0, D5 ;another comment
```
---

## Labels

Labels are used to refer to specific offsets of the code. They are created by appending a colon to a name.
```
    lea value, A0    ;load the address of the label "value" into A0
    move.l  (A0), D0 ;load data into D0 from where A0 is pointing to
    rts

value:               ;this is a label
    d32 0x12345678
```
When used with branches, index and displacement-style addressing modes, labels are converted to an offset automatically.
```
    bne.b  skip                   ;branch to label "skip" if not equal
    move.b (values, PC, D1.w), D0 ;otherwise, load byte from label "values" + D1.w
    lea (values, PC), A0          ;load address of "values" into A0
skip:
    rts

values:
    d08 1, 2, 3, 4
```
As of right now, almost anything that isn't parsed as an opcode / value / etc is accepted as a label. This is likely to become more well-defined at some point.

### Sub labels

Sub labels are useful when you want to use commonly used names multiple times in different subroutines.
They are created by prefixing a label with a dot. Internally, they are stored as "label.sublabel" and can be
referenced as such from anywhere in the code.
```
do_something:
    ;...
exit:
    ;...

some_function:
    ;...
exit: ;error: label redefinition!
    nop

;-----

do_something:
    nop
.exit:
    nop

some_function:
    nop
.exit: ;ok
    nop
    beq.b do_something.exit ;referencing a sub label from another main label
```
---

## Defines

Defines are created by prepending a name with an exclamation mark, followed by an equals sign and a value.
```
!five = 5

move.l #!five, D0
```
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
Note: if `d08` or `d24` define an uneven amount of data in bytes, a padding zero byte will be appended.
```
;various ways of defining data
;7 bytes are defined here, so a padding zero byte will automatically be appended.
!five = 5

some_offset:
    ;...

d08 1, 0x02, 0b11, -4, !five, some_offset, 1
```