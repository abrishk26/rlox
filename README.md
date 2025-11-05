# 🦀 Rlox — A Tree-Walk Interpreter in Rust

This is a Rust implementation of the **Lox programming language** from *Crafting Interpreters* by Robert Nystrom.

It follows the **tree-walk interpreter** model and supports the features up to classes (without inheritance).  
The project demonstrates how an interpreter can be built from scratch, including scanning, parsing, and evaluation.

---

## ⚙️ Installation

There’s no published binary or package for Rlox — the interpreter can only be run by **cloning the repository** and using Cargo directly.

```bash
# Clone the repository
git clone https://github.com/abrishk26/rlox.git
cd rlox

# Run a Lox file
cargo run -- examples/basics.lox
```

> ⚠️ Make sure to include `--` before the file path so Cargo passes it to your program and not to itself.

---

## 🔧 Built-in Functions

Rlox provides the following built-in functions:

- `print(arg1, arg2, ...)` — prints all arguments **concatenated with spaces**, without a newline.  
- `println(arg1, arg2, ...)` — prints all arguments **concatenated with spaces**, with a newline.  
- `input([prompt])` — reads a line from the user. Optionally displays `prompt` if provided.

Example:

```lox
print("Enter your name: ");
var name = input();
println("Hello, " + name + "!");

var age = input("Enter your age: ");
println("You are " + age + " years old.");
```

---

## 🧩 Example Snippets

Here are simple Lox programs you can use to test the interpreter.  
None of these examples use inheritance — they’re all self-contained demonstrations.

### 1. Basic Arithmetic
```lox
print(2 + 3 * 4);     // 14
println((10 - 4) / 3);  // 2
print(5 > 2);         // true
println("lox" + "lang"); // "loxlang"
```

---

### 2. Variables and Scope
```lox
var greeting = "Hello";
var name = "World";
println(greeting + ", " + name);

{
  var greeting = "Hi";
  println(greeting); // "Hi"
}

println(greeting); // "Hello"
```

---

### 3. Functions
```lox
fun add(a, b) {
  return a + b;
}

println(add(2, 3)); // 5
```

---

### 4. While Loop
```lox
var i = 0;
while (i < 3) {
  println(i);
  i = i + 1;
}
```

---

### 5. Closures
```lox
fun makeCounter() {
  var count = 0;
  fun inc() {
    count = count + 1;
    println(count);
  }
  return inc;
}

var counter = makeCounter();
counter(); // 1
counter(); // 2
```

---

### 6. Classes and Methods
```lox
class Person {
  init(name, age) {
    this.name = name;
    this.age = age;
  }

  sayHi() {
    println("Hi, I'm " + this.name + " and I'm " + this.age + " years old.");
  }
}

var p = Person("John", 40);
p.sayHi();
```

---

### 7. Property Access
```lox
class Box {
  init(value) {
    this.value = value;
  }

  show() {
    println("Value: " + this.value);
  }
}

var b = Box("Rust");
b.show();       // "Value: Rust"
b.value = "Lox";
b.show();       // "Value: Lox"
```

---

### 8. Error Examples
```lox
// Undefined variable
println(notDefined); // runtime error

// Calling a non-function
var x = 123;
x(); // should error: Can only call functions and classes.
```

---
