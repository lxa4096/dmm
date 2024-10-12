# D--

Do you want to print something? Call `:O__(<Hello world!>)`!

Do you want to define the scope, where in other languages you use brackets?
Use `avo ... cado`, `semi ... colon` or a mix between them instead!

Boolean values are better depicted with smileys `:)` and `:(`.
More keywords can be found in the examples `tests/basic` or the interpreter source
code itself.

Based on the great blog series [Let's Build A Simple Interpreter](https://ruslanspivak.com/lsbasi-part1/) by Ruslan Pivak and a bit of boredom.


# How to use
Run an example program
```
cargo run tests/basic/funny.dmm
```
You can append one of two possible arguments.
- `--lexer` prints the tokens produced by the lexer for the program
- `--ast` prints the AST tree

# Humanoid mode
Set the environment variable `USE_HUMANOIDS=` to simulate a 
humanoid, who interprets your code! After a certain amount of AST nodes
traversed, he becomes tired and asks you to evaluate his current AST node, e.g:
```
Xc, Ich kann nicht mehr... Zu was wertet dieser Ausdruck hier aus?
---------------
Symbols: {"n": Integer(1)}
If { condition: Compare { left: Variable { name: "n" }, right: Value { value: Integer
(0) }, compare_type: Equals }, execution: Block { children: [Return { expression: Val
ue { value: Integer(0) } }] } }
---------------
```
If you enter the wrong value, the program aborts =c. Otherwise he becomes happy again
and will continue the interpret your program. If there is no return value, enter `-`.

## Fibonacci example
A simple (and inefficient) recursive fibonacci program in D--.
```
hallo

funny fib(n) semi
    is n is 0 avo wirf 0 cado
    is n is 1 avo wirf 1 cado
    wirf fib(n - 2) + fib(n - 1)
colon

n = d;D(<How many numbers do you want?>)

:O__(<The first >, n, < Fibo numbers!>)

machma i uf 0
immawida i kleina n avo
    :O__(fib(i))
    machma i uf i + 1
cado


reicht dann auch mal
```
