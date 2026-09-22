# 11 — Concorrência (Threads, Message Passing, Shared State)

Rust garante **data-race freedom** em compile-time via sistema de
ownership + `Send`/`Sync` traits.

## Threads

```rust
use std::thread;
use std::time::Duration;

// Spawn
let handle = thread::spawn(|| {
    for i in 1..10 {
        println!("{i} from thread");
        thread::sleep(Duration::from_millis(1));
    }
});

// join — espera thread terminar
handle.join().unwrap();

// move é necessário para capturar valores
let v = vec![1, 2, 3];
let handle = thread::spawn(move || {
    println!("{v:?}");
});
handle.join().unwrap();
```

### Builder Pattern

```rust
use std::thread;

let builder = thread::Builder::new()
    .name("worker".into())
    .stack_size(1024 * 1024);

let handle = builder.spawn(|| {
    println!("named thread");
}).unwrap();
```

### Thread Pool (rayon)

```rust
use rayon::prelude::*;

fn sum_of_squares(input: &[i32]) -> i32 {
    input.par_iter()       // parallel iterator
         .map(|&i| i * i)
         .sum()
}

// Parallel sort
let mut v = vec![...];
v.par_sort_unstable();
```

## Message Passing (Channels)

mpsc = Multiple Producer, Single Consumer:

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    tx.send(42).unwrap();
});

let received = rx.recv().unwrap();     // blocking
// let received = rx.try_recv();        // non-blocking

// Múltiplos producers (clone tx)
let tx1 = tx.clone();
thread::spawn(move || { tx1.send(1).unwrap(); });
thread::spawn(move || { tx.send(2).unwrap(); });

// Iteração sobre receiver
for received in rx {
    println!("{received}");
}
```

### Async Channels (tokio / crossbeam)

```rust
// tokio::sync::mpsc
use tokio::sync::mpsc;
let (tx, mut rx) = mpsc::channel(32);  // bounded

// crossbeam::channel — multi-consumer também
use crossbeam::channel;
let (tx, rx) = channel::unbounded();
```

## Shared State (Mutex + Arc)

```rust
use std::sync::{Mutex, Arc};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("{}", *counter.lock().unwrap()); // 10
```

### Mutex Internals

```rust
{
    let mut data = mutex.lock().unwrap();
    *data += 1;
    // MutexGuard droppado aqui → unlock automático
}
// try_lock() para non-blocking
if let Some(mut guard) = mutex.try_lock().ok() {
    *guard += 1;
}
```

### RwLock (multiple readers / single writer)

```rust
use std::sync::RwLock;

let lock = RwLock::new(5);

// Múltiplos readers simultâneos
let r1 = lock.read().unwrap();
let r2 = lock.read().unwrap();

// Writer (exclusivo)
let mut w = lock.write().unwrap();
*w += 2;
```

### Condvar (condition variable)

```rust
use std::sync::{Mutex, Condvar};
let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair2 = Arc::clone(&pair);

thread::spawn(move || {
    let (lock, cvar) = &*pair2;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_one();  // acorda thread esperando
});

let (lock, cvar) = &*pair;
let mut started = lock.lock().unwrap();
while !*started {
    started = cvar.wait(started).unwrap();  // libera lock e espera
}
```

## Send e Sync (Marker Traits)

```rust
// Send: pode ser transferido entre threads
// Sync: pode ser compartilhado entre threads (&T é Send)

// A maioria dos tipos são Send + Sync automaticamente
// Tipos raw pointer: !Send + !Sync
// Rc<T>: !Send + !Sync (contagem non-atomic)
// Arc<T>: Send + Sync
// RefCell<T>: Send + !Sync
// MutexGuard<'_, T>: !Send (deve ser droppado na mesma thread)

// Implementação manual (unsafe)
unsafe impl Send for MyType {}
unsafe impl Sync for MyType {}
```

## Barrier

```rust
use std::sync::{Arc, Barrier};
use std::thread;

let barrier = Arc::new(Barrier::new(3));  // espera 3 threads

let mut handles = vec![];
for i in 0..3 {
    let barrier = Arc::clone(&barrier);
    handles.push(thread::spawn(move || {
        println!("thread {i} working");
        barrier.wait();       // todas esperam aqui
        println!("thread {i} past barrier");
    }));
}
```

## Atomic Types

```rust
use std::sync::atomic::{AtomicU64, Ordering};

let counter = AtomicU64::new(0);
counter.fetch_add(1, Ordering::SeqCst);

// Ordering:
// Relaxed — sem garantias de ordenação (mais rápido)
// Release — writes visíveis para Acquire
// Acquire — vê writes de Release
// AcqRel — Acquire + Release
// SeqCst — sequencialmente consistente (default, mais lento)
```

## Boas Práticas

1. **Prefira message passing** sobre shared state — mais seguro e
   testável
2. **Arc<Mutex<T>>** é o padrão para shared state — mas evite lock
   contention
3. **Lock scopes devem ser pequenos** — mutex guard vive até fim do
   escopo
4. **Evite `unwrap()` em locks** — use `lock().expect("poisoned")`
5. **Thread pools (rayon)** para tarefas paralelas genéricas
6. **canal bounded (tokio)** para backpressure
7. **Atomic types** são mais leves que Mutex para tipos simples
8. **`Send`/`Sync`** são implementados automaticamente — raramente
   precisa implementar manualmente
9. **Mutex poisoning** — se uma thread panica segurando o lock, o
   mutex fica corrompido. Use `.lock().unwrap_or_else(|e| e.into_inner())`
10. **RwLock** para read-heavy workloads
