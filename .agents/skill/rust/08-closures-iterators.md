# 8 — Closures e Iterators (Programação Funcional)

## Closures

Closures são funções anônimas que capturam o ambiente:

```rust
// Sintaxe |args| body
let add_one = |x: i32| -> i32 { x + 1 };
let add_one = |x| x + 1;                // type inference

// Exige anotação de tipo se não houver contexto
let identity = |x: &str| -> &str { x };
```

### Captura do Ambiente

```rust
let x = 5;

// Captura por referência imutável (Fn)
let print_x = || println!("{x}");

// Captura por referência mutável (FnMut)
let mut move_x = || { x += 1; };

// Captura por valor (FnOnce) — move x para dentro da closure
let consume_x = || { drop(x); };
```

### Closure Traits (Fn, FnMut, FnOnce)

| Trait | Captura | Chama | Usado para |
|-------|---------|-------|------------|
| `FnOnce` | por valor | 1x | Consumir capturas |
| `FnMut` | &mut ref | várias | Mutar capturas |
| `Fn` | & ref | várias | Só ler |

```rust
fn call_fn<F: Fn()>(f: F) { f(); f(); }
fn call_fn_mut<F: FnMut()>(mut f: F) { f(); f(); }
fn call_fn_once<F: FnOnce()>(f: F) { f(); }

// Toda closure implementa FnOnce
// Toda closure que não move, implementa FnMut
// Toda closure que não move + não muta, implementa Fn
```

### move Keyword

Força a closure a tomar ownership das capturas:

```rust
let x = vec![1, 2, 3];

// Sem move — borrow, precisa viver mais que closure
let equal = |z| z == x;

// Com move — ownership transferido
thread::spawn(move || {
    println!("{x:?}");
}).join().unwrap();
```

### Retornando Closures

```rust
fn returns_closure() -> impl Fn(i32) -> i32 {
    |x| x + 1
    // move é necessário se captura ambiente
}

// Retornando closures com captura
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}
```

### Closures em Structs

```rust
struct Cacher<T, R>
where
    T: Fn(u32) -> R,
{
    calculation: T,
    value: Option<R>,
}
```

## Iterators

O trait `Iterator`:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // muitas default methods...
}
```

### Creating Iterators

```rust
let v = vec![1, 2, 3];

v.iter()         // &T (borrow)
v.iter_mut()     // &mut T (mutable borrow)
v.into_iter()    // T (consumes v)

// De ranges
(0..10).step_by(2)

// De Option
let iter = Some(42).into_iter();

// Custom iterator
struct Counter { count: u32 }
impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count < 6 { Some(self.count) } else { None }
    }
}
```

### Consuming Adaptors (consomem o iterator)

```rust
let sum: u32 = v.iter().sum();
let count = v.iter().count();
let max = v.iter().max();
let min = v.iter().min();
let last = v.iter().last();
let nth = v.iter().nth(2);
let collected: Vec<_> = v.iter().collect();
let reduced = v.iter().fold(0, |acc, x| acc + x);
let all_gt = v.iter().all(|x| x > &0);
let any_gt = v.iter().any(|x| x > &5);
let position = v.iter().position(|x| x == &3);
```

### Iterator Adaptors (retornam novos iterators) — lazy!

```rust
v.iter()
    .map(|x| x * 2)             // transforma cada elemento
    .filter(|x| x > &5)         // mantém só os que satisfazem
    .take(3)                     // pega N elementos
    .skip(2)                     // pula N elementos
    .step_by(2)                  // pula de 2 em 2
    .flatten()                   // achata nested iterators
    .flat_map(|x| 0..*x)        // map + flatten
    .enumerate()                 // (index, element)
    .zip(other_iter)             // combina com outro iterator
    .chain(other_iter)           // concatena iterators
    .rev()                       // reverse
    .cycle()                     // loop infinito
    .inspect(|x| println!("{x}")) // debug
    .peekable()                  // peek sem consumir
    .collect::<Vec<_>>();        // materializa
```

### Partition

```rust
let (even, odd): (Vec<_>, Vec<_>) = (0..10).partition(|x| x % 2 == 0);
```

### Unzip

```rust
let pairs = vec![(1, "a"), (2, "b")];
let (nums, strs): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
```

### FilterMap

```rust
// filter + map em uma passada só
let vals = ["1", "two", "3", "four"];
let nums: Vec<i32> = vals.iter()
    .filter_map(|s| s.parse().ok())
    .collect();
```

### Combinando Closures e Iterators

```rust
struct Shoe { size: u32, style: String }

fn shoes_in_size(shoes: Vec<Shoe>, shoe_size: u32) -> Vec<Shoe> {
    shoes.into_iter()
        .filter(|s| s.size == shoe_size)
        .collect()
}

// Uso: `for` loop vs pipeline (são equivalentes)
```

### IntoIterator

```rust
// for x in v equivale a v.into_iter()
// Arrays (Edition 2021+)
let arr = [1, 2, 3];
for x in arr {}  // OK — Edition 2021

// Implementando IntoIterator pra tipo customizado
struct Grid(Vec<Vec<i32>>);
impl IntoIterator for Grid {
    type Item = i32;
    type IntoIter = std::vec::IntoIter<i32>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().flatten().collect::<Vec<_>>().into_iter()
    }
}
```

## Zero-Cost Abstractions

Iterators e closures **não adicionam overhead** em relação a loops
manuais. O compilador otimiza o pipeline em um único loop.

```rust
// Este código:
let sum: i32 = (0..1000)
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .sum();

// Compila para o mesmo assembly que:
let mut sum = 0;
for x in 0..1000 {
    if x % 2 == 0 {
        sum += x * 2;
    }
}
```

## Boas Práticas

1. **Preferir pipelines de iterators** a loops manuais — mais
   expressivo e compila igual
2. **Usar `filter_map`** para filter + map em uma passada
3. **`collect::<Vec<_>>()`** só quando precisar materializar
4. **Lazy evaluation** — iterators não executam até serem consumidos
5. **`enumerate()`** substitui `for i in 0..v.len() {}`
6. **Closures pequenas podem ser `Fn`** — prefira à mais genérica
7. **`move`** é necessário para threads, async, ou retornar closure
8. **Evitar `for` em favor de `map`/`filter`/`fold`** em cadeias curtas
9. **Usar `any()`/`all()`** em vez de loops com flag bool
10. **`flat_map()`** para nested loops
