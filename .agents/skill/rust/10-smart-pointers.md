# 10 — Smart Pointers

## Box<T> — Heap Allocation Simples

```rust
// Aloca na heap
let b = Box::new(5);
assert_eq!(*b, 5);  // deref

// Tipos recursivos (precisam de tamanho conhecido)
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

// Trait objects
let trait_obj: Box<dyn Iterator<Item = i32>> = Box::new(0..10);
```

## Deref Trait — Deref Coercion

```rust
use std::ops::Deref;

struct MyBox<T>(T);
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

// Deref coercion: &MyBox<String> → &String → &str
fn hello(name: &str) { println!("{name}"); }
let m = MyBox(String::from("Rust"));
hello(&m);  // automaticamente converte
```

Deref coercion acontece com:
- `&T` → `&U` quando `T: Deref<Target=U>`
- `&mut T` → `&mut U` quando `T: DerefMut<Target=U>`
- `&mut T` → `&U` quando `T: Deref<Target=U>`

## Drop Trait — Destructor

```rust
struct CustomSmartPointer { data: String }

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping with data: {}", self.data);
    }
}

// Liberação manual (std::mem::drop)
let c = CustomSmartPointer { data: "stuff".into() };
drop(c);  // drop() da std, não Drop::drop
```

## Rc<T> — Reference Counting (non-threaded)

```rust
use std::rc::Rc;

let a = Rc::new(5);
let b = Rc::clone(&a);  // incrementa contagem
let c = Rc::clone(&a);  // incrementa contagem
assert_eq!(Rc::strong_count(&a), 3);

// Precisa de Rc para múltiplos owners
// Não é Send nem Sync (só single-threaded)

// Cons list com Rc
enum List<T> {
    Cons(T, Rc<List<T>>),
    Nil,
}
use List::{Cons, Nil};
let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
let b = Cons(3, Rc::clone(&a));
let c = Cons(4, Rc::clone(&a));
```

## Arc<T> — Atomic Reference Counting (threaded)

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);

let mut handles = vec![];
for _ in 0..3 {
    let data = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        println!("{data:?}");
    }));
}
```

## RefCell<T> — Interior Mutability

Permite mutabilidade através de referência imutável — verificado em
**runtime** (não compile-time):

```rust
use std::cell::RefCell;

let data = RefCell::new(5);

// borrow() — empresta imutável (runtime)
let borrowed = data.borrow();
// drop(borrowed);

// borrow_mut() — empresta mutável (runtime check)
*data.borrow_mut() += 1;

// Panic se violar regras de borrowing em runtime:
let a = data.borrow();       // OK
let b = data.borrow_mut();   // PANIC: already borrowed
```

### RefCell + Rc = Multiple Owners Mutável

```rust
use std::rc::Rc;
use std::cell::RefCell;

let value = Rc::new(RefCell::new(5));

let a = Rc::clone(&value);
let b = Rc::clone(&value);

*a.borrow_mut() += 10;     // muta através de Rc

println!("{a:?}");  // 15
println!("{b:?}");  // 15
```

### Use Cases para RefCell

- Mock objects em testes
- Quando o compilador não entende que a mutação é segura
- Caches, contadores, estados internos

## Weak<T> — Prevenindo Reference Cycles

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

let leaf = Rc::new(Node {
    value: 3,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![]),
});

let branch = Rc::new(Node {
    value: 5,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![Rc::clone(&leaf)]),
});

*leaf.parent.borrow_mut() = Rc::downgrade(&branch);

// Weak::upgrade → Option<Rc<T>>
println!("parent: {:?}", leaf.parent.borrow().upgrade());
```

| Rc methods | Weak methods |
|------------|--------------|
| `Rc::clone(&rc)` | `Rc::downgrade(&rc)` |
| `Rc::strong_count(&rc)` | `Weak::upgrade()` → `Option<Rc<T>>` |
| `Rc::weak_count(&rc)` | |

## Cell<T> — Interior Mutability para Copy Types

```rust
use std::cell::Cell;

let cell = Cell::new(5);
cell.set(10);
let val = cell.get();  // 10 (copia o valor)
// Sem runtime borrow check (Copy types são baratos de copiar)
```

## OnceCell / OnceLock — Inicialização Tardia

```rust
use std::cell::OnceCell;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

fn get_config() -> &'static Config {
    GLOBAL_CONFIG.get_or_init(|| {
        Config::load()
    })
}
```

## Resumo

| Smart Pointer | Multi-Owner | Mutável | Thread-Safe | Overhead |
|---------------|-------------|---------|-------------|----------|
| `Box<T>` | Não | Sim (se mut) | Sim | Mínimo |
| `Rc<T>` | Sim | Não | Não | Count (non-atomic) |
| `Arc<T>` | Sim | Não | Sim | Count (atomic) |
| `RefCell<T>` | Não | Sim (runtime) | Não | Runtime check |
| `Cell<T>` | Não | Sim (Copy) | Não | Nenhum |
| `Mutex<T>` | Não | Sim | Sim | Lock |
| `RwLock<T>` | Não | Sim | Sim | Lock |
| `OnceCell<T>` | 1x | Sim (1x) | Não (Cell) / Sim (Lock) | Option |

## Boas Práticas

1. **Box é o default** para heap allocation — use quando precisar
2. **Rc/Arc** só quando múltiplos owners — caso contrário, mova
3. **RefCell é code smell** em APIs públicas — prefira mut ownership
4. **Arc<Mutex<T>>** é o padrão para estado compartilhado entre
   threads
5. **Weak<T>** previne reference cycles — essencial em grafos/árvores
6. **Cell para Copy types** — mais leve que RefCell
7. **OnceCell para lazy statics** — melhor que `lazy_static!` ou
   `once_cell` (agora na std)
