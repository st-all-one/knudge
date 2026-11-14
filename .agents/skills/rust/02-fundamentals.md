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
| `!` | 0 bytes | never type — inabitado (nenhum valor) |

```rust
let decimal = 98_222;       // underscore separador
let hex = 0xff;             // base 16
let octal = 0o77;           // base 8
let binary = 0b1111_0000;   // base 2
let byte = b'A';            // u8 literal (ASCII)

// bool::toggle (1.100) — in-place, equivalente a `b = !b`
let mut running = false;
running.toggle();
assert!(running);
```

### Floating-Point: Métodos Algébricos (Rust 1.98+)

Os tipos `f32` e `f64` ganharam métodos **"algébricos"** para as cinco
operações básicas. Eles permitem que o compilador aplique otimizações
usando as propriedades algébricas dos números reais (associatividade,
distributividade, etc.), mesmo que essas propriedades **não** valham para
representações de ponto flutuante IEEE 754. É análogo ao `-ffast-math`
de outras linguagens.

```rust
let a = 1.0_f64;
let b = 2.0_f64;
let c = 3.0_f64;
let d = 4.0_f64;

// Normal: avaliação estritamente left-associative ((a+b)+c)+d
let normal = a + b + c + d;

// Algébrico: o compilador PODE reordenar, ex.: (a+b) + (c+d),
// habilitando paralelismo/loop-vectorization
let alg = a.algebraic_add(b).algebraic_add(c).algebraic_add(d);
```

| Método | Operação |
|--------|----------|
| `algebraic_add` | adição (`+`) |
| `algebraic_sub` | subtração (`-`) |
| `algebraic_mul` | multiplicação (`*`) |
| `algebraic_div` | divisão (`/`) |
| `algebraic_rem` | resto (`%`) |

**Características importantes:**

- **Não-determinísticos**: o compilador escolhe as otimizações, então o
  resultado pode variar entre builds/plataformas.
- **Nunca causam UB** — são seguros; apenas relaxam garantias de
  precisão/ordem.
- **Não são** um replacement direto dos operadores quando a reprodutibilidade
  bit-a-bit importa (ex.: finanças, hashing de floats). Use-os em
  hot loops numéricos (SIMD, ML, gráficos) onde a reordenação compensa.

```rust
// Exemplo: redução vetorizada de um slice
let sum = data.iter().copied()
    .reduce(f64::algebraic_add)
    .unwrap_or(0.0);
```

> **Quando NÃO usar**: se você depende de igualdade exata, comparações
> determinísticas ou resultados reprodutíveis entre máquinas, mantenha
> os operadores normais (`+`, `-`, `*`, `/`, `%`).

### Formatação de Inteiros em Buffer (Rust 1.98+)

Todos os inteiros primitivos têm `format_into`, que escreve a forma
decimal em um `NumBuffer<Self>` — um buffer opaco grande o suficiente para
qualquer valor do tipo — e retorna um `&str` emprestado do buffer. Evita
`write!` dinâmico e tem performance comparável ao crate `itoa`.

```rust
use core::fmt::NumBuffer;

let mut buf = NumBuffer::<i32>::new();
let s: &str = 42_i32.format_into(&mut buf);
assert_eq!(s, "42");

// Útil em serialização / logging de alto throughput
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

### Never type (`!`)

`!` é o tipo canônico **inabitado**: nenhum valor existe, então qualquer
código depois de uma expressão `!` é inalcançável. Desde a 1.100 ele pode
ser **nomeado em qualquer posição** (antes, só no retorno de funções).

```rust
fn panic_forever() -> ! {
    loop { println!("forever"); }     // loop infinito tem tipo !
}

// `return`, `break`, `continue`, `panic!` e `process::exit` são `!`
fn pick(opt: Option<u32>) -> u32 {
    match opt {
        Some(x) => x,
        None => return 0,             // `!` coage para u32
    }
}

// Nomeável em argumentos e tipos genéricos (1.100)
fn absurd<T>(x: !) -> T { x }         // nunca chamável, mas type-checka
fn infallible<T>(res: Result<T, !>) -> T {
    let Ok(v) = res;                  // Err é inabitado: match exaustivo
    v
}
```

`!` também marca operações infalíveis: `Result<T, !>` dispensa tratar
`Err`, e `Infallible` agora é `type Infallible = !`.

> **Rust 1.100+**: o *fallback* de `!` mudou de `()` para `!` em todas as
> editions. Código que dependia de `!` virar `()` (ex.: `f()?;` com `f`
> genérico) pode precisar de anotação de tipo. Ver `06-error-handling.md`.

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
