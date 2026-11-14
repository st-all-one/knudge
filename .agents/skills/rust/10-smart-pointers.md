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

### Box: reuso de alocação e allocators (1.100+)

```rust
// Box::take extrai o valor e devolve o slot não inicializado
let (value, slot) = Box::take(Box::new(5));
assert_eq!(value, 5);
let b = Box::write(slot, 6);        // reusa a MESMA alocação
assert_eq!(*b, 6);

// Box::map transforma o valor, reusando a alocação quando possível
let b = Box::map(Box::new(7), |i| i + 7);
assert_eq!(*b, 14);

// Box com allocator explícito (não é o #[global_allocator])
use std::alloc::System;
let b: Box<u32, System> = Box::new_in(5, System);
let (ptr, alloc) = Box::into_raw_with_allocator(b);
let b = unsafe { Box::from_raw_in(ptr, alloc) };

// Variante NonNull
let b: Box<u32, System> = Box::new_in(9, System);
let (nn, alloc) = Box::into_non_null_with_allocator(b);
let b = unsafe { Box::from_non_null_in(nn, alloc) };
let a: &System = Box::allocator(&b);
```

`Box::take`, `Box::map` e `Box::allocator` são **funções associativas**
(`Box::map(b, f)`, não `b.map(f)`) para não colidir com métodos do `T`
interno.

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

// Rc::map (1.100): transforma o valor reusando a alocação
let r = Rc::map(Rc::new(7), |x| x + 1);
assert_eq!(*r, 8);
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

// Arc::map (1.100): transforma o valor reusando a alocação
let a = Arc::map(Arc::new(7), |x| x * 2);
assert_eq!(*a, 14);
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

### UnsafeCell: acesso unchecked (1.100)

`UnsafeCell` é a primitiva por trás de `Cell`, `RefCell` e `Mutex`. A
1.100 estabilizou três operações `const` sem nenhuma checagem:

```rust
use std::cell::UnsafeCell;

let uc = UnsafeCell::new(5);

let old = unsafe { uc.replace(10) };          // devolve o valor antigo
assert_eq!(old, 5);

unsafe {
    *uc.as_mut_unchecked() += 1;              // &mut T direto
    assert_eq!(*uc.as_ref_unchecked(), 11);   // &T direto
}
assert_eq!(uc.into_inner(), 11);
```

**Safety**: `as_ref_unchecked`/`as_mut_unchecked` não checam nada (nem
em debug). É UB manter outra referência viva, mutar por fora enquanto a
referência existe, ou correr com outro acesso. Use só quando o aliasing
já for garantido por uma camada superior (ex.: `Mutex`, `RefCell`).

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

## ManuallyDrop — Controle Manual de Drop (Rust 1.98+)

`ManuallyDrop<T>` suprime o `Drop` automático, deixando o controle para
você. Antes da 1.96 havia um bug: mover um `ManuallyDrop<Box<_>>` cujo
`Box` já havia sido dropado era **UB**. A 1.96 corrigiu o comportamento, e
a **1.98 documenta a garantia estável** de que esse código continua sendo
seguro (não-UB) no futuro (ver RFC 3336).

```rust
use std::mem::ManuallyDrop;

let mut x = ManuallyDrop::new(Box::new(1));
unsafe { ManuallyDrop::drop(&mut x) }; // Box dropado, x ainda válido
let x = x; // OK desde 1.96 — não é mais UB (garantido na 1.98)
```

> **Regra**: `ManuallyDrop::drop` deve ser chamado no máximo uma vez, e o
> valor não deve ser usado depois. A garantia da 1.98 cobre apenas a
> movimentação do wrapper após o drop, não o reuso do conteúdo.

## Box::into_non_null / Box::from_non_null (1.99)

```rust
use std::ptr::NonNull;

// Box<T> → NonNull<T> (sem alocação extra)
let boxed = Box::new(42);
let ptr: NonNull<i32> = Box::into_non_null(boxed);

// NonNull<T> → Box<T>
let boxed = unsafe { Box::from_non_null(ptr) };
```

## Allocators customizados (`Allocator`) — 1.100+

A 1.100 estabiliza o núcleo de `allocator_api`: `Vec` e `Box` ganham um
parâmetro de tipo `A: Allocator`. `Global` delega ao alocador global
(`#[global_allocator]` ou o default do `std`); `System` usa
`malloc`/`HeapAlloc`.

```rust
use std::alloc::{AllocError, Allocator, Layout, System};
use std::ptr::NonNull;

/// Wrapper que delega ao `System` (ex.: para instrumentar alocações).
#[derive(Clone, Copy, Default)]
struct Counting;

unsafe impl Allocator for Counting {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        System.allocate(layout)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { System.deallocate(ptr, layout) }
    }
    // grow / grow_zeroed / shrink têm default (aloca, copia, libera)
}

let mut v: Vec<u8, Counting> = Vec::with_capacity_in(16, Counting);
v.extend(b"abc");

let b: Box<u32, Counting> = Box::new_in(9, Counting);
assert_eq!(*b, 9);

// Vec e Box expõem o alocador usado
let _: &Counting = v.allocator();
let _: &Counting = Box::allocator(&b);
```

| Método | Obrigatório | Contrato |
|---|---|---|
| `allocate(&self, Layout)` | sim | devolve `NonNull<[u8]>`; pode ser maior que o pedido |
| `deallocate(&self, NonNull<u8>, Layout)` | sim | `unsafe`; o layout deve *caber* no bloco |
| `allocate_zeroed` | default | aloca e zera |
| `grow` / `grow_zeroed` / `shrink` | default | copiam + realocam se preciso |

Pontos que importam:

- `Allocator` é `unsafe trait`: o bloco fica *currently allocated* até
  `deallocate`/`grow`/`shrink`, o alocador não pode tocar na memória do
  usuário nem fazer unwind. `&A`, `&mut A`, `Box<A>`, `Rc<A>` e `Arc<A>`
  também implementam `Allocator` e são *equivalentes* ao `A` interno.
- ZSTs podem ser alocadas (diferente de `GlobalAlloc`), então
  implementações sobre `malloc`/jemalloc precisam tratar `size == 0` e
  ponteiro nulo.
- `Allocator` **não** substitui `#[global_allocator]`: o global continua
  sendo um `GlobalAlloc`, definido uma vez por programa.
- Escopo atual: `Vec` e `Box`. `String`, `HashMap`, `VecDeque` e
  `Rc`/`Arc` como containers ainda não aceitam `A` — é o próximo passo
  do projeto "Allocators 1.0".

Para mover um `Vec` alocado por `A` entre camadas sem copiar:

```rust
let (ptr, len, cap, alloc) = v.into_parts_with_allocator();
let v = unsafe { Vec::from_parts_in(ptr, len, cap, alloc) };
```

## Cow<'_, T> — clone-on-write

Evita alocar quando o dado pode ser emprestado:

```rust
use std::borrow::Cow;

fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', "_"))
    } else {
        Cow::Borrowed(input)     // zero alloc
    }
}
```

### Comparação com Vec/slice (1.100)

`Vec<T>` e slices comparam com `Cow<'_, [U]>` sem alocar nem converter:

```rust
let owned = vec![1, 2, 3];
let cow: Cow<'_, [i32]> = Cow::Borrowed(&[1, 2, 3]);

assert_eq!(owned, cow);        // PartialEq<Cow<[U]>> for Vec<T, A>
assert_eq!(&owned[..], cow);   // ... e para &[T] / &mut [T]
```

> **Rust 1.100**: `dropping_references`, `dropping_copy_types` e
> `undropped_manually_drops` agora também disparam em
> `ptr::drop_in_place`, não só em `mem::drop`.

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

> `Box<T, A>` e `Vec<T, A>` armazenam o alocador (salvo quando `A` é um
> ZST, como `Global`/`System`), então o tamanho do container pode
> aumentar em relação ao default.

> **Rust 1.99**: `Pin::new_unchecked` teve seus safety invariants
> levemente alterados. `Box::leak` agora avisa sobre evitar unleaking.

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
8. **Allocator customizado** só com perfil que justifique — comece com
   `System`/`Global` e meça antes de implementar `allocate`/`deallocate`
