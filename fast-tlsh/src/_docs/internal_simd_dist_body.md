SIMD-friendly TLSH Body Distance Calculation
=============================================

*   Author: Tsukasa OI
*   Date: 2024-02-16 (original description of the algorithm)
    *   Slightly modified (for this Rust crate): 2024-03-03
    *   Last mofidication: 2025-02-15

This document describes a SIMD-friendly algorithm for computing distance between
two TLSH bodies (consisting an array of dibits), the biggest contributor for
fuzzy hash comparison speedups in this crate.  Because this is a small
bit-slicing algorithm, it can be used even without wide SIMD vectors.


Objective to Resolve
---------------------

The distance between two TLSH bodies can be determined as below:

1.  Split two bodies as arrays of dibit (2-bit) values  
    (e.g. a TLSH body byte `0b00_01_10_11` → `[0b00, 0b01, 0b10, 0b11]`)
2.  For each dibit pair (at the same position; `X` and `Y`), compute
    the sub-distance:  
    `D := abs(X - Y); D := if D == 3 { 6 } else { D };`
3.  Take the sum.


Expression
-----------

### Sub-expression 1 (in 2-bits)

```text
A := ((X ^ Y) & (X ^ ((Y & 0b01) * 0b11)))
```

### Sub-expression 2 (in 2-bits)

```text
B := ((X ^ Y) & ((((X ^ Y) & (X ^ (0b10 - (X & 0b01)))) >> 1) * 0b11))
```

### Description

`A + B` (3-bit) equals `if D == 3 { 6 } else { D }` where `D == abs(X - Y)`.

This is discovered as a result of a machine-based search and is near optimal
under following conditions:

*   Virtual SIMD vectors of 2-bit integers (with no decomposition)
*   No cross-lane effects occur (to make SIMD handling easier)
*   Sum of each elements of two virtual SIMD vectors results in the
    correct value (but in 3-bits each).
*   `|`, `&`, `^`, `+`, `-`, `<<`, `>>` and `*` are allowed
    *   `<<` and `>>` only allows right hand side constant of `1`.
    *   `*` only allows right hand side constant of `0b11` (`3`).
        *   `x * 0` and `x * 1` are redundant
            and `x * 2` is equivalent to `x << 1`.
*   Do not count any variable sharing
    (TODO: redo the search accounting variable sharing).

Under those constraints, this is the optimal solution with the minimum number
of operations.

Considering the instruction delay, this is not always optimal.  But other
candidates with lower possible instruction delay tend to require more operations
than the latency we can save and `x * 3` can be easily replaced to either
`(x << 1) | x` or `(x << 1) + x`.

Also note that, `X ^ Y` (appears three times in total) can be shared and the
actual number of operations can be reduced by 2.
This expression (`X ^ Y`) is represented as `Z` in the next section.


Step by Step Evaluation
------------------------

```text
Z := X ^ Y     #  ****   00  01  10  11   01  00  11  10   10  11  00  01   11  10  01  00
```

The comment can be interpreted as follows:

1.  Possible values (denoting `00`, `01`, `10` and `11` respectively)  
    *   `*`: Possible
    *   `_`: Impossible
2.  Sixteen entries of the "truth" table  
    (depending on `X` and `Y`; see the rows `X` and `Y` for corresponding input)

```text
X              #  ****   00  00  00  00   01  01  01  01   10  10  10  10   11  11  11  11
Y              #  ****   00  01  10  11   00  01  10  11   00  01  10  11   00  01  10  11
Z := X ^ Y     #  ****   00  01  10  11   01  00  11  10   10  11  00  01   11  10  01  00

A := Y & 0b01  #  **__   00  01  00  01   00  01  00  01   00  01  00  01   00  01  00  01
A := A * 0b11  #  *__*   00  11  00  11   00  11  00  11   00  11  00  11   00  11  00  11
A := A ^ X     #  ****   00  11  00  11   01  10  01  10   10  01  10  01   11  00  11  00
A := A & Z     #  ****   00  01  00  11   01  00  01  10   10  01  00  01   11  00  01  00

B := X & 0b01  #  **__   00  00  00  00   01  01  01  01   00  00  00  00   01  01  01  01
B := 0b10 - B  #  _**_   10  10  10  10   01  01  01  01   10  10  10  10   01  01  01  01
B := B ^ X     #  *_*_   10  10  10  10   00  00  00  00   00  00  00  00   10  10  10  10
B := B & Z     #  *_*_   00  00  10  10   00  00  00  00   00  00  00  00   10  10  00  00
B := B >> 1    #  **__   00  00  01  01   00  00  00  00   00  00  00  00   01  01  00  00
B := B * 0b11  #  *__*   00  00  11  11   00  00  00  00   00  00  00  00   11  11  00  00
B := B & Z     #  *_**   00  00  10  11   00  00  00  00   00  00  00  00   11  10  00  00
```

Note that all steps do not involve any overflows and shift-outs of non-zero
bit values.  It makes sure that running this algorithm in an effectively SIMD
vector does not affect other lanes in the same vector even if the element width
is wider than 2-bits (this is usually true).

Then, the final 3-bit sum can be computed as follows:

```text
X      #   00  00  00  00   01  01  01  01   10  10  10  10   11  11  11  11
Y      #   00  01  10  11   00  01  10  11   00  01  10  11   00  01  10  11

A      #   00  01  00  11   01  00  01  10   10  01  00  01   11  00  01  00
B      #   00  00  10  11   00  00  00  00   00  00  00  00   11  10  00  00

A + B  #  000 001 010 110  001 000 001 010  010 001 000 001  110 010 001 000
   ==  #    0   1   2   6    1   0   1   2    2   1   0   1    6   2   1   0 (decimal)
```

Of course, `A + B` equals to the expected result.

Adding `A` and `B` after each horizontal addition (2-bits to 4-bits) helps to
merge the SIMD vector registers early (as a result, each 4-bit lane in `A + B`
contains a value in range of `0..=12`, which fits in a 4-bit lane).
