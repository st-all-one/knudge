# 4 — Structs, Enums e Pattern Matching

## Structs

```rust
// Struct nomeada (C-like)
struct User {
    active: bool,
    username: String,
    sign_in_count: u64,
}

// Tuple struct (newtype pattern)
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

// Unit struct (zero-size)
struct AlwaysEqual;
```

### Instanciação e shorthand

```rust
let user = User {
    active: true,
    username: String::from("alice"),
    sign_in_count: 1,
};

// Field init shorthand (variável com mesmo nome do campo)
let username = String::from("bob");
let user = User {
    username,  // equivalente a username: username
    ..Default::default()  // update syntax
};

// Destructuring
let User { username, .. } = user;
```

### #[derive] — traits comuns

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct Config {
    host: String,
    port: u16,
}
```

### Métodos

```rust
impl User {
    // Associated function (construtor)
    fn new(name: &str) -> Self {
        Self {
            username: name.to_string(),
            active: true,
            sign_in_count: 0,
        }
    }

    // Method (&self)
    fn display(&self) {
        println!("{}", self.username);
    }

    // Mutable method
    fn increment(&mut self) {
        self.sign_in_count += 1;
    }

    // Ownership method (raro)
    fn into_string(self) -> String {
        self.username
    }
}

// Múltiplos impl blocks
impl User {
    fn is_active(&self) -> bool { self.active }
}
```

## Enums

```rust
// Enum com dados variados
enum Message {
    Quit,
    Move { x: i32, y: i32 },  // struct-like
    Write(String),              // tuple-like
    ChangeColor(i32, i32, i32), // tuple-like
}

// Métodos em enums
impl Message {
    fn call(&self) {
        match self {
            Message::Write(text) => println!("{text}"),
            Message::Quit => println!("bye"),
            _ => (),
        }
    }
}
```

### Option<T> (o null seguro de Rust)

```rust
enum Option<T> {
    Some(T),
    None,
}
// Já no prelude — não precisa `Option::` prefix
```

### Result<T, E> (o error handling)

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### C-like Enums

```rust
enum Color {
    Red = 0xff0000,
    Green = 0x00ff00,
    Blue = 0x0000ff,
}
let color = Color::Red as i32; // cast explícito
```

## Pattern Matching

### Match Exaustivo

```rust
match value {
    1 => "um",
    2..=5 => "dois a cinco",
    6 | 7 | 8 => "seis, sete ou oito",
    _ => "outro",
}
```

### Destructuring em match

```rust
// Tupla
match pair {
    (0, y) => println!("x=0, y={y}"),
    (x, 0) => println!("x={x}, y=0"),
    _ => (),
}

// Enum
match msg {
    Message::Quit => (),
    Message::Move { x, y } => println!("{x},{y}"),
    Message::Write(s) => println!("{s}"),
    Message::ChangeColor(r, g, b) => println!("{r},{g},{b}"),
}

// Struct
match user {
    User { username, active: true, .. } => println!("{username} ativo"),
    _ => (),
}

// Slice
match arr {
    [first, rest @ ..] => println!("{first}, rest: {rest:?}"),
    [] => println!("vazio"),
}

// Ponteiro/referência
match &val {
    &val => println!("referência para {val}"),
}
```

### Combinators (Option/Result)

```rust
// map — transforma o valor interno
let len = name
    .as_deref()
    .map(|s| s.len())
    .unwrap_or(0);

// and_then — flatMap
let zip = name
    .and_then(|n| n.parse::<i32>().ok());

// Combinators encadeados
let value = opt
    .ok_or("missing")
    .and_then(|v| v.parse::<i32>())
    .map_err(|e| format!("error: {e}"))
    .unwrap_or(-1);
```

### Extraindo com if let / while let

```rust
// if let — match de um braço só
if let Some(user) = maybe_user {
    println!("tem user");
}

// while let — processa fila até None
while let Some(item) = queue.pop() {
    process(item);
}

// let-else — desestrutura ou sai
let Some(val) = optional else {
    return Err("missing value");
};
```

## Boas Práticas

1. **Preferir métodos a funções avulsas** — mais idiomático, autocomplete
2. **`Self` é alias para o tipo do impl** — use, não repita o nome
3. **Destructuring em `let` e parâmetros** é mais conciso que acesso
   posicional
4. **Evitar muitos campos em structs** — considere builder pattern
5. **Newtype pattern** (`struct Email(String)`) para segurança de tipo
6. **Usar `Default` derive** para structs com defaults sensatos
7. **Evitar `match` com `_` quando variantes podem ser adicionadas**
   — prefira match exaustivo, depois lide com novas variantes
8. **`if let` é preferível a `match` para Single-variant checks**
