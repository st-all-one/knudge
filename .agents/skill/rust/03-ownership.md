# 3 — Ownership, Borrowing e Lifetimes

## Ownership Rules

1. Cada valor tem **exatamente um dono** (owner)
2. O dono é a variável que o possui
3. Quando o dono sai de escopo, o valor é **dropado** (drop)

```rust
{                              // s não é válido aqui
    let s = String::from("olá"); // s é válido a partir daqui
    // usa s
}                              // escopo termina → s dropado
```

### Move vs Copy

```rust
// Copy — tipos que ficam na stack
let x = 5;         // i32: Copy
let y = x;         // ainda posso usar x

// Move — tipos com heap allocation
let s1 = String::from("hello");
let s2 = s1;       // s1 é MOVED para s2
// println!("{s1}"); // ERROR: s1 foi movido!

// Clone — explícito, custoso
let s2 = s1.clone(); // deep copy

// Drop — tipos que não podem ser copiados
// Copy NÃO pode ser implementado com Drop
```

### Stack vs Heap

```rust
// Stack: tamanho conhecido em compile-time
// i32 (4 bytes), bool (1 byte), [i32; 3] (12 bytes)
// (i32, f64) (12 bytes)

// Heap: tamanho dinâmico, alocado via allocator global
// String, Vec<T>, Box<T>, HashMap<K,V>

fn main() {
    let stack_i32: i32 = 42;               // Stack
    let heap_string: String = "hello".into(); // Stack (ptr, len, cap) → Heap (data)
    let heap_box: Box<i32> = Box::new(42);  // Stack (ptr) → Heap (i32)
}
```

| Operação | Stack | Heap |
|----------|-------|------|
| Alocação | `add rsp, N` (1 instrução) | `malloc` (syscall ou bump alloc) |
| Liberação | `sub rsp, N` (ao sair do escopo) | `free` ou RAII |
| Fragmentação | Zero | Pode fragmentar |

## Borrowing (Referências)

```rust
// & — referencia sem tomar ownership (empresta)
fn calculate_length(s: &String) -> usize { s.len() }
// s é emprestado, não dropado ao fim

// &mut — referência mutável
fn change(s: &mut String) {
    s.push_str(" world");
}
```

### Regras de Borrowing

1. **Qualquer número de referências imutáveis (`&T`)** simultâneas
2. **Exatamente UMA referência mutável (`&mut T`)** por vez
3. Referências **nunca podem viver mais que o valor original**

```rust
let mut s = String::from("hello");

let r1 = &s;      // OK
let r2 = &s;      // OK
// let r3 = &mut s; // ERROR: já tem imutáveis

println!("{r1} {r2}");

let r3 = &mut s;  // OK — imutáveis não usados mais
```

### Dangling References

O compilador **garante** que não existam dangling references:

```rust
fn dangle() -> &String {
    let s = String::from("hello");
    &s
} // s dropado, mas referência retornada → ERROR!
```

## Slices

View contígua sobre uma coleção (sem ownership):

```rust
let s = String::from("hello world");
let hello = &s[0..5];    // &str
let world = &s[6..];     // até o fim

let arr = [1, 2, 3, 4, 5];
let slice = &arr[1..3];  // &[i32]
```

## Lifetimes

### Lifetime Elision (regras automáticas)

1. Cada parâmetro referência ganha seu próprio lifetime
2. Se há exatamente 1 lifetime de input, ele é atribuído a todos
   outputs
3. Se há `&self` ou `&mut self`, seu lifetime é atribuído a todos
   outputs

```rust
// O compilador infere:
fn first_word(s: &str) -> &str
// Equivalente a:
fn first_word<'a>(s: &'a str) -> &'a str
```

### Lifetime Annotations (quando necessário)

```rust
// Função com lifetimes explícitos
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Struct com referência
struct Excerpt<'a> {
    part: &'a str,
}

// Múltiplos lifetimes
fn complex<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x
}
```

### `'static` Lifetime

```rust
// Referência que vive toda a execução do programa
let s: &'static str = "Hello";  // string literals

// Trait bound: T não contém referências não-'static
fn foo<T: 'static>(t: T) {}

// Leak um Box para 'static
let leaked: &'static mut [u8] = Box::leak(Box::new([0u8; 1024]));
```

### Lifetime Bounds

```rust
// T: 'a — T deve viver ao menos 'a
struct Wrapper<'a, T: 'a> {
    value: &'a T,
}

// 'a: 'b — lifetime 'a é maior/igual a 'b
fn longer_lifetime<'a, 'b: 'a>(x: &'a str, _y: &'b str) -> &'a str {
    x
}
```

### Lifetime Coercion

Lifetimes maiores são automaticamente encurtados quando necessário:

```rust
fn takes_static(_s: &'static str) {}
fn takes_any<'a>(_s: &'a str) {}

let s: &'static str = "hello";
takes_any(s);  // OK: 'static é encurtado para 'a
```

## RAII (Resource Acquisition Is Initialization)

Recursos são adquiridos na inicialização e liberados no destructor:

```rust
struct MyResource;
impl Drop for MyResource {
    fn drop(&mut self) {
        println!("limpeza automática");
    }
}

{
    let _res = MyResource;
} // drop() chamado automaticamente

// Liberação manual antecipada
std::mem::drop(my_resource);
```

## Boas Práticas

1. **Preferir referências (`&T`) a ownership** quando possível — evita
   alocações
2. **Usar `&str` em vez de `&String`** para argumentos (coerção DST)
3. **Usar `&[T]` em vez de `&Vec<T>`** para argumentos
4. **structs raramente precisam de lifetimes** — a maioria deve owned
5. **Evitar `Clone` em hot paths** — custoso
6. **`Copy` só para tipos pequenos** (< 128 bytes, stack-only)
7. **Lifetime elision funciona 95% das vezes** — não anote sem
   necessidade
