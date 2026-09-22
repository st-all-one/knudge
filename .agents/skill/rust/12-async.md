# 12 — Async/Await

Async em Rust é **zero-cost**: não há runtime embutido. Você escolhe
seu runtime (tokio, async-std, smol).

## Future Trait

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

// Poll::Ready(valor) — terminou
// Poll::Pending — ainda não, me chame de novo quando estiver pronto
```

## async/await Sintaxe

```rust
use tokio::time::{sleep, Duration};

async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() {
    let data = fetch_data("https://example.com").await.unwrap();
    println!("{data}");
}
```

### async main

```rust
// tokio
#[tokio::main]
async fn main() {
    println!("tokio runtime");
}

// Desugar de #[tokio::main]:
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("tokio runtime");
    });
}
```

## Tokio Runtime

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### Runtime manual

```rust
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // async code
    });

    // Multi-threaded, work-stealing scheduler
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
}
```

### Tokio Tasks (green threads)

```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        "hello from task"
    });
    let result = handle.await.unwrap();
    println!("{result}");

    // Múltiplas tasks
    let mut handles = vec![];
    for i in 0..10 {
        handles.push(tokio::spawn(async move {
            work(i).await;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
}
```

## Concorrência Async

### join! (esperar múltiplos futures)

```rust
use tokio::join;

async fn concurrent() {
    // Roda ambos simultaneamente (mesma task)
    let (r1, r2) = join!(fetch("a.com"), fetch("b.com"));
    println!("{r1} {r2}");
}
```

### select! (primeiro a completar)

```rust
use tokio::select;
use tokio::time::{sleep, Duration};

async fn race() {
    select! {
        result = fetch("fast.com") => println!("fast: {result}"),
        _ = sleep(Duration::from_secs(1)) => println!("timeout"),
    }
}
```

### spawn (paralelismo real)

```rust
use tokio::spawn;

async fn parallel() {
    // spawn cria tasks separadas (paralelismo real)
    let t1 = spawn(fetch("a.com"));
    let t2 = spawn(fetch("b.com"));
    let (r1, r2) = (t1.await.unwrap(), t2.await.unwrap());
}
```

## Tokio Channels

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);  // bounded

    tokio::spawn(async move {
        tx.send(42).await.unwrap();
    });

    while let Some(msg) = rx.recv().await {
        println!("{msg}");
    }

    // oneshot — um valor só
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move { tx.send("done").unwrap(); });
    let msg = rx.await.unwrap();

    // broadcast — 1 para muitos
    let (tx, _rx1) = tokio::sync::broadcast::channel(16);
    let mut rx2 = tx.subscribe();
    tx.send("hello").unwrap();
}
```

## Streams

```rust
use tokio_stream::StreamExt;
use futures::stream;
use tokio::time::{interval, Duration};

// Interval stream
let mut stream = tokio_stream::wrappers::IntervalStream::new(
    interval(Duration::from_secs(1))
);

while let Some(_) = stream.next().await {
    println!("tick");
}

// Criando streams
let s = stream::iter(vec![1, 2, 3]);
let doubled = s.map(|x| x * 2);

// Stream adaptors
let s = stream::iter(0..10);
let vec: Vec<_> = s
    .filter(|x| futures::future::ready(*x % 2 == 0))
    .collect()
    .await;
```

## Pin<&mut T> e Unpin

```rust
use std::pin::Pin;

// Futures podem ser auto-referenciais (Poll::Pending salva pointers
// internos). Pin garante que o valor não é movido depois de pollado.

// A maioria dos tipos é Unpin (pode ser movido)
// self-referencial structs são !Unpin

// Pin<Box<T>> para heap pinning
let pinned = Box::pin(async { 42 });

// Pin<&mut T> para stack pinning
async fn needs_pinned(x: &mut i32) {
    let pinned = std::pin::pin!(async { *x += 1 });
    pinned.await;
}
```

## Cancellation Safety

```rust
use tokio::select;

// Tokio tasks podem ser canceladas (drop do JoinHandle)
// Futures devem ser cancellation-safe:
// - E/S geralmente é safe (ler/socket podem ser retomados)
// - MutexLock: NÃO é cancellation-safe — pode travar

// select! dropa futures não escolhidos
async fn cancel_safe_example() {
    select! {
        _ = tokio::spawn(long_task()) => {},
        _ = sleep(Duration::from_secs(1)) => {},
    }
}
```

## Async Patterns

### Timeout

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout() {
    let result = timeout(Duration::from_secs(5), fetch("example.com")).await;
    match result {
        Ok(Ok(data)) => println!("{data}"),
        Ok(Err(e)) => eprintln!("error: {e}"),
        Err(_) => eprintln!("timeout"),
    }
}
```

### Retry

```rust
async fn retry<T, F, Fut>(f: F, max_retries: u32) -> Result<T, Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    let mut last_err = None;
    for _ in 0..max_retries {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => last_err = Some(e),
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(last_err.unwrap())
}
```

## Comparing: Threads vs Async

| Threads | Async |
|---------|-------|
| 1:1 OS thread | M:N green thread |
| Stack per thread (MB) | Stack per task (KB) |
| Preemptive | Cooperative (await points) |
| Ideal para CPU-bound | Ideal para I/O-bound |
| Mais overhead | Mais throughput |
| std library | Runtime necessário |

## Boas Práticas

1. **Tokio é o runtime padrão** da indústria
2. **Prefira async para I/O-bound** (web, db, filesystem)
3. **Use `spawn_blocking`** para CPU-bound em runtime tokio
4. **Evite `block_on`** dentro de async code (deadlock)
5. **Cancellation safety** — não segure locks através de `.await`
6. **Channels bounded** para backpressure
7. **`select!`** para timeouts, races, e graceful shutdown
8. **`join!` para paralelizar** futures que devem rodar juntos
9. **Streams** para dados que chegam ao longo do tempo
10. **Teste cancellation safety** — tasks podem ser canceladas a
    qualquer momento
