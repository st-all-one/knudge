# 5 — Collections (Vec, String, HashMap)

## Vec<T>

```rust
// Criação
let mut v: Vec<i32> = Vec::new();
let v = vec![1, 2, 3];
let v = (0..10).collect::<Vec<_>>();

// Escrita
v.push(1);
v.extend([2, 3, 4]);
v.insert(0, -1);          // custo O(n)

// Leitura segura
if let Some(x) = v.get(100) {
    println!("{x}");
}
let x = &v[0];            // panic se vazio

// Iteração
for x in &v {}             // &i32
for x in &mut v {}         // &mut i32
for x in v {}              // i32 (consome vec)

// Remoção
v.pop();                   // Option<T>
v.remove(0);               // O(n)
v.swap_remove(0);          // O(1) — troca com último

// Ordenação
v.sort();
v.sort_by(|a, b| b.cmp(a));
v.sort_unstable();         // mais rápido que sort()
v.reverse();

// Propriedades
v.len();                   // quantos elementos
v.is_empty();
v.capacity();              // capacidade alocada
v.shrink_to_fit();         // libera excesso
v.resize(10, Default::default());
v.truncate(5);
v.clear();

// Vec de enums para tipos heterogêneos
enum Value { I32(i32), F64(f64), Text(String) }
let items = vec![Value::I32(1), Value::F64(2.0)];
```

## String

```rust
// Criação
let s = String::new();
let s = "hello".to_string();
let s = String::from("world");
let s = format!("hello {name}");  // concatenação sem alocação extra

// Append
s.push('!');               // char
s.push_str(" world");       // &str

// Concatenação (move self)
let s = s1 + &s2;          // s1 moved, s2 borrowed
let s = format!("{s1}{s2}{s3}"); // sem mover — mais idiomático

// Indexação NÃO é possível diretamente (UTF-8)
// s[0] — ERROR! Use:
let slice = &s[0..4];      // panic se não em boundary UTF-8
for c in s.chars() {}       // Char (Unicode scalar)
for b in s.bytes() {}       // u8 (raw bytes)

// Transformação
let upper = s.to_uppercase();
let lower = s.to_lowercase();
let replaced = s.replace("old", "new");
let trimmed = s.trim();

// Split
for word in s.split_whitespace() {}
for line in s.lines() {}
for part in s.split(',') {}

// Conversões
let bytes: Vec<u8> = s.into_bytes();
let s = String::from_utf8(bytes).expect("invalid UTF-8");
let s: &str = &string;     // borrow, não custa
```

### &str — string slice

```rust
let s: &'static str = "Hello";  // string literal — 'static lifetime
let slice: &str = &full_string[start..end];  // borrow
```

## HashMap<K, V>

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Yellow"), 50);

// Entry API — insert condicional
scores.entry(String::from("Blue")).or_insert(50); // só insere se não existir
scores.entry(String::from("Red")).or_insert(75);
scores.entry(String::from("Blue")).and_modify(|e| *e += 1).or_insert(1);

// Leitura
scores.get("Blue");              // Option<&i32>
scores.get("Blue").copied();     // Option<i32>
scores.get("Blue").copied().unwrap_or(0);

// Iteração
for (key, value) in &scores {}
for (key, value) in &mut scores {}

// Ownership
// Tipos com Copy são copiados, outros são moved
// Referências viram borrows (precisa lifetime)
scores.insert(key, value);  // key e value moved
```

### HashSet

```rust
use std::collections::HashSet;

let mut set = HashSet::new();
set.insert(1);
set.insert(2);

// Operações de conjunto
let union: Vec<_> = set1.union(&set2).collect();
let diff: Vec<_> = set1.difference(&set2).collect();
let inter: Vec<_> = set1.intersection(&set2).collect();
```

## Outras Collections

| Collection | Descrição |
|------------|-----------|
| `VecDeque<T>` | Fila dupla (push/pop em ambos lados) |
| `LinkedList<T>` | Lista duplamente ligada (raro) |
| `BinaryHeap<T>` | Max-heap |
| `BTreeMap<K, V>` | Map ordenado por chave |
| `BTreeSet<T>` | Set ordenado |
| `HashMap<K, V>` | Map hash (default) |
| `HashSet<T>` | Set hash |

## Boas Práticas

1. **Preferir `Vec`** — melhor performance de cache, mais simples
2. **`Vec::with_capacity(n)`** se tamanho aproximado é conhecido
3. **`sort_unstable()`** é mais rápido que `sort()` — use a menos que
   precise de estabilidade
4. **`swap_remove`** é O(1), `remove` é O(n)
5. **Evitar `+` para strings** — `format!()` é mais legível e eficiente
6. **Entry API** para inserção condicional em HashMap
7. **Preferir `&str`** para argumentos de função (mais flexível)
8. **`collect::<Vec<_>>()`** de iterators é idiomático
9. **Evitar `HashMap` com chaves grandes** — hashing custa caro
10. **`BTreeMap`** se precisa de ordenação ou range queries
