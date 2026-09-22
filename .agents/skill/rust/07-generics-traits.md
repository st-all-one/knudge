# 7 — Generics, Traits e Tipos Avançados

## Generics

### Em funções

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest { largest = item; }
    }
    largest
}
```

### Em structs

```rust
struct Point<T, U> {
    x: T,
    y: U,
}

// Métodos genéricos
impl<T: Clone, U: Clone> Point<T, U> {
    fn mixup<V, W>(self, other: Point<V, W>) -> Point<T, W> {
        Point { x: self.x, y: other.y }
    }
}
```

### Em enums

```rust
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

### Monomorfização

Generics são **zero-cost**: o compilador gera código específico para
cada tipo concreto (monomorfização). Não há virtual dispatch.

## Traits

### Definição e implementação

```rust
trait Summary {
    fn summarize(&self) -> String;
    fn author(&self) -> String {
        String::from("unknown")  // default implementation
    }
    fn summarize_with_author(&self) -> String {
        format!("by {}: {}", self.author(), self.summarize())
    }
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}, by {}", self.headline, self.author)
    }
}
```

### Trait Bounds

```rust
// Sintaxe básica
fn notify<T: Summary>(item: &T) {}

// Múltiplos bounds
fn notify<T: Summary + Display>(item: &T) {}

// where clause (mais legível com muitos bounds)
fn some_function<T, U>(t: &T, u: &U) -> i32
where
    T: Display + Clone,
    U: Clone + Debug,
{ 42 }

// impl Trait syntax (argumento)
fn print_summary(item: &impl Summary) {}
// equivalente a generic com trait bound

// impl Trait syntax (retorno)
fn returns_summarizable() -> impl Summary {
    Tweet { ... }
}
```

### Blanket Implementations

```rust
impl<T: Display> ToString for T {
    // Tudo que implementa Display ganha to_string() automaticamente
}
```

### Supertraits

```rust
trait OutlinePrint: Display {
    fn outline_print(&self) {
        let output = self.to_string();
        println!("{output}");
    }
}
// OutlinePrint só pode ser implementado para tipos que implementam Display
```

### Fully Qualified Syntax

```rust
trait Animal { fn name(&self) -> String; }
trait Pet { fn name(&self) -> String; }

struct Dog;
impl Animal for Dog { fn name(&self) -> String { "Dog".into() } }
impl Pet for Dog { fn name(&self) -> String { "Rex".into() } }

fn main() {
    let dog = Dog;
    println!("{}", <Dog as Animal>::name(&dog));
    println!("{}", <Dog as Pet>::name(&dog));
}
```

### Marker Traits

```rust
// Traits sem métodos — funcionam como marcas no type system
trait Send: unsafe impl {}    // seguro para enviar entre threads
trait Sync: unsafe impl {}    // seguro para compartilhar entre threads
trait Copy: Clone {}          // bitwise copy é seguro
trait Sized {}                // tamanho conhecido em compile-time
```

## Associated Types

```rust
trait Iterator {
    type Item;                               // associated type
    fn next(&mut self) -> Option<Self::Item>;
}

impl Iterator for Counter {
    type Item = u32;                         // definido na impl
    fn next(&mut self) -> Option<Self::Item> { todo!() }
}
```

### Associated Constants

```rust
trait ConstValue {
    const DEFAULT: Self;
    fn get(&self) -> u32;
}

impl ConstValue for u32 {
    const DEFAULT: u32 = 0;
    fn get(&self) -> u32 { *self }
}
```

## Advanced Traits

### Default Generic Parameters (Operator Overloading)

```rust
use std::ops::Add;

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

// Com parâmetro genérico default
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

// Milimetros + Metros
struct Millimetres(u32);
struct Metres(u32);
impl Add<Metres> for Millimetres {
    type Output = Millimetres;
    fn add(self, other: Metres) -> Millimetres {
        Millimetres(self.0 + (other.0 * 1000))
    }
}
```

### Newtype Pattern

```rust
// Wrapper para implementar trait externa em tipo externo (orphan rule)
struct Wrapper(Vec<String>);

impl Display for Wrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

// Type safety
struct Email(String);
struct Phone(String);
fn send_email(to: Email) {}

send_email(Email("alice@example.com".into()));
// send_email(Phone("123".into())); // ERROR!
```

## Type Aliases

```rust
type Kilometers = i32;           // sinônimo
type Thunk = Box<dyn Fn() + Send + 'static>;
type Result<T> = std::result::Result<T, std::io::Error>;

// Uso em bibliotecas
pub type IoResult<T> = Result<T, io::Error>;
```

## Never Type (!)

```rust
// ! = never type — "nunca retorna"
fn forever() -> ! {
    loop { println!("forever"); }
}

// Útil em match com arms de tipos diferentes
let guess: u32 = match guess.trim().parse() {
    Ok(num) => num,
    Err(_) => continue,  // continue é !
};

// panic! e loop infinito também são !
```

## Dynamically Sized Types (DSTs)

```rust
// DSTs: tamanho não conhecido em compile-time
// str, [T], dyn Trait

// Não podem ser armazenados diretamente na stack:
// let s: str = "hello";      // ERROR: não Sized
// let arr: [u32] = [1, 2];   // ERROR

// Sempre atrás de um ponteiro:
let s: &str = "hello";              // fat pointer (ptr + len)
let arr: &[i32] = &[1, 2, 3];      // fat pointer (ptr + len)
let obj: &dyn Display = &some_val; // fat pointer (ptr + vtable)

// ?Sized — trait bound que permite DST ou Sized
fn generic<T: ?Sized>(t: &T) {}
```

## PhantomData

```rust
use std::marker::PhantomData;

// Marca que um tipo possui parâmetro genérico mesmo sem campo
struct Iter<'a, T: 'a> {
    ptr: *const T,
    _marker: PhantomData<&'a T>, // ownership-like semantics
}

// Unidades de medida via phantom types
struct Meter<Unit>(f64, PhantomData<Unit>);
struct Cm;
struct Km;

let dist = Meter::<Km>(5.0, PhantomData);
```

## Coherence (Orphan Rule)

- Você pode implementar **seu trait** em **qualquer tipo**
- Você pode implementar **qualquer trait** em **seu tipo**
- Você **não pode** implementar trait externo em tipo externo

```rust
// Seu trait, tipo externo → OK
trait MyTrait {}
impl MyTrait for Vec<String> {}

// Trait externo, seu tipo → OK
impl Display for MyStruct {}

// Trait externo, tipo externo → ERROR
// impl Display for Vec<String> {} // ERROR!
```

## Boas Práticas

1. **Preferir `impl Trait`** em argumentos para funções simples
2. **Usar `where` clauses** quando múltiplos bounds ficam ilegíveis
3. **Newtype pattern** para type safety e implementação de traits
   externas
4. **Evitar trait objects** (`dyn Trait`) onde generics bastam
5. **Use marker traits** para adicionar garantias no type system
6. **Associated types** para traits com uma única output type
7. **Default generic params** para operadores e extensibilidade
8. **PhantomData** para parâmetros genéricos que só existem no
   type level
