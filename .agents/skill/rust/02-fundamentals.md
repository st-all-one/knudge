# 2 — Fundamentos da Linguagem

## Variáveis e Mutabilidade

```rust
// Imutável por default — safety e legibilidade
let x = 5;          // const, imutável

// Mutabilidade explícita
let mut y = 10;
y += 5;

// Shadowing — re-declara com mesmo nome
let name = "hello";
let name = name.len();  // agora é usize

// Shadowing permite mudar tipo
let value = "123";
let value: i32 = value.parse().unwrap();
```

### Constantes vs Statics

```rust
// Const — compilado inline, sem endereço fixo
const MAX_POINTS: u32 = 100_000;

// Static — endereço fixo, 'static lifetime
static APP_NAME: &str = "MyApp";
static mut COUNTER: u32 = 0;  // unsafe para acessar
```

## Tipos Escalares

| Tipo | Tamanho | Valores |
|------|---------|---------|
| `i8`..`i128` | 8-128 bits | com sinal |
| `u8`..`u128` | 8-128 bits | sem sinal |
| `isize`/`usize` | arquitetura (64 bits) | com/sem sinal |
| `f32`/`f64` | 32/64 bits | IEEE 754 |
| `char` | 32 bits | Unicode scalar (4 bytes!) |
| `bool` | 8 bits | `true`/`false` |
| `()` | 0 bytes | unit type |

```rust
let decimal = 98_222;       // underscore separador
let hex = 0xff;             // base 16
let octal = 0o77;           // base 8
let binary = 0b1111_0000;   // base 2
let byte = b'A';            // u8 literal (ASCII)
```

## Tipos Compostos

```rust
// Tupla — tipos heterogêneos, tamanho fixo
let tup: (i32, f64, u8) = (500, 6.4, 1);
let (x, y, z) = tup;       // destructuring
let first = tup.0;          // index access

// Array — tipos homogêneos, tamanho fixo (stack)
let arr: [i32; 3] = [1, 2, 3];
let first = arr[0];

// Slice — view sobre uma sequência
let slice: &[i32] = &arr[1..3];
```

## Funções

```rust
// `fn` — parâmetros tipados, retorno explícito ou implícito
fn add(x: i32, y: i32) -> i32 {
    x + y  // expressão final (sem `;`) = retorno implícito
}

fn greet(name: &str) {
    println!("Hello {name}!");
}
```

### Diverging Functions (never type)

```rust
fn panic_forever() -> ! {
    loop {
        println!("forever");
    }
    // ! significa "nunca retorna"
}
```

## Controle de Fluxo

### if/else (expressão)

```rust
let condition = true;
let number = if condition { 5 } else { 6 };
// Ambos braços precisam mesmo tipo!
```

### loop (expressão)

```rust
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;  // retorna valor
    }
};

// Labels
'outer: loop {
    loop {
        break 'outer;  // break do outer loop
    }
};
```

### for (range + iterator)

```rust
for i in 0..10 {}           // 0..9
for i in 0..=10 {}          // 0..10 inclusive
for (idx, val) in arr.iter().enumerate() {}
```

### while

```rust
while n < 100 {
    n *= 2;
}
```

### match (expressão exaustiva)

```rust
match value {
    1 => println!("one"),
    2 | 3 => println!("two or three"),
    4..=10 => println!("range"),
    _ => println!("catch-all"),
}
```

## Expressões vs Statements

**Tudo em Rust é expressão ou statement**:

```rust
fn example() -> i32 {
    let y = {          // bloco é expressão
        let x = 3;
        x + 1          // valor da expressão (sem ;)
    };                 // ; faz let y = ... ser statement
    y                  // retorno implícito
}
```

## Pattern Matching Avançado

```rust
// Destructuring
let (a, b, ..) = (1, 2, 3, 4);
let Point { x, y } = point;
let Some(val) = option else { return };

// @ bindings
match num {
    e @ 0..=5 => println!("pequeno: {e}"),
    e @ 6..=10 => println!("médio: {e}"),
    _ => (),
}

// Guards
match pair {
    (x, y) if x == y => println!("iguais"),
    (x, y) if x + y == 0 => println!("opostos"),
    _ => (),
}
```

## if let / while let / let-else

```rust
// if let — match de um braço só
if let Some(val) = optional {
    println!("tem valor: {val}");
}

// while let — enquanto padrão casar
while let Some(val) = iter.next() {
    println!("{val}");
}

// let-else (Rust 1.65+) — desestrutura ou early return
let Some(val) = optional else {
    return;
};
```

## Doc Comments

```rust
/// Documentação para este item.
/// Suporta Markdown, code blocks, links.
///
/// ```
/// let x = add(1, 2);
/// assert_eq!(x, 3);
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

//! Documentação interna para o módulo/crate.
```

## Boas Práticas

1. **Preferir `let` imutável** — mutabilidade explícita é mais fácil de
   auditar
2. **Usar `const` para constantes**, `static` raramente necessário
3. **Evitar `isize`/`usize`** em APIs públicas (portabilidade)
4. **Preferir `for` sobre `while`** com índices — mais idiomático
5. **match exaustivo** — o compilador garante que todos os casos são
   tratados
6. **Nomear com snake_case** para funções/variáveis, UpperCamelCase
   para tipos, SCREAMING_SNAKE para constantes
7. **Usar `_` prefixo** para variáveis não usadas: `_unused`
