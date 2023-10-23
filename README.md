# Rin

Rin is a simple transpiler made in Rust, it transforms code from an unspecified language to MARIE.js assembly code. This tool grants users the ability to integrate various programming constructs such as functions, loops, conditions, and pointers with specific memory addresses.

## Features
- **Pointer Management:** Directly assign values to memory locations with the pointer notation.
- **Control Structures:** Incorporate `loop`, `while`, and `if` conditions seamlessly.
- **Direct Memory Addressing:** Specify the exact memory address where data should be stored.
- **Intuitive Syntax:** The design ensures clarity and straightforwardness, making the code easy to write and read.

### Main Constructs:
- `var`: Declare a new variable.
- `*ptr`: Pointer notation for direct memory addressing.
- `loop`: Construct for creating a loop.
- `if`: Conditional construct for decision-making.

## Getting Started

### 1. Installation:

To dive into Rin, you must first install it. Follow these simple steps:

```bash
git clone https://github.com/zam-cv/rin
cd rin
cargo install --path .
```

### 2. Start Coding:

Once installed, write your code in Rin's syntax, and then use the transpiler to convert it into MARIE.js assembly code.

### 3. Syntax Guidelines:

A quick glance at the Rin syntax:

```rust
var *ptr -> 200;
var current = 0;
var result = 0;
var sum = 0;
var i = 1;
var limit = 10;

loop {
  current = input();
  *ptr = current;
  sum = sum + current;

  if (i == limit) {
    result = sum / limit;
    print(result);

    sum = 0;
    i = 0;
  }

  i = i + 1;
  ptr = ptr + 1;
}
```

### 4. Compilation:

Once your code is ready, run Rin to convert your high-level code to assembly code compatible with MARIE.js.