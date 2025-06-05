# Xod 🧮

A tiny REPL for bitwise arithmetic and expression evaluation.

Xod is a minimal scripting language and interactive REPL designed for experimenting with bitwise logic and integer math. Think of it as a scratchpad for systems engineers, embedded developers, and anyone who needs to test bitwise operations in a focused, rule-consistent environment.

## ✨ Features

    •	Evaluate bitwise and arithmetic expressions interactively.
    •	Integer-only logic with no floating point or negative values.
    •	Support for basic list manipulation and iteration.
    •	Familiar control flow with if, while, and for blocks.
    •	Hex, octal, binary, and decimal formatting.
    •	Simple built-in commands like help(), clear(), and history().

## 🧠 Language Overview

### Xod supports:

    •	Operators: &, |, ^, ~, <<, >>, +, -, *, /, %, **
    •	Booleans: ==, !=, <, <=, >, >=
    •	List Methods: append, prepend, front, back, index
    •	Builtin Commands: hex(), bin(), log(base, value), range(start, end), etc.
    •	Control Flow:

```c
for(x in range(1, 5)) { bin(x) }
if(3 & 1) { hex(3) }
while(x < 8) { x = x + 1 }
```

> 🛑 Floating point values and negative integers are not supported.

### ❗️Operator Precedence

To avoid ambiguity in bitwise expressions, parentheses are required to define precedence. For example:

```c
(a & b) | c   // ✅ Clear
a & b | c     // ❌ Error: ambiguous expression
```

## 🧪 Usage

Launch the REPL:

```bash
xod
```

Then type any expression:

```c
>> 3 & 6
2
>> hex(255)
0xff
>> for(x in range(0, 4)) { bin(x) }
0b0
0b1
0b10
0b11
```

## 🧰 Commands

    •	`help()` – Show help message
    •	`history()` – Show input history
    •	`clear()` – Clear the screen
    •	`quit()` – Exit the REPL

## 📦 Installation

Coming soon! For now, clone the repo and run it directly:

```bash
cargo install xod
```

## 🛠 Contributing

Contributions, feedback, and bitwise rants are welcome!
Feel free to open an issue or PR. If you’re interested in improving the parser, extending list functionality, or adding file I/O support—let’s chat.
