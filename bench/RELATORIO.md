# Relatório de benchmark do knudge

> Gerado pela bancada [`bench/`](README.md). Números são **medianas de latência** salvo
> indicação; reproduza com `make bench`. Snapshot bruto em [`ULTIMO.md`](ULTIMO.md) e
> [`ULTIMO.json`](ULTIMO.json).

## Ambiente

| Item | Valor |
|---|---|
| CPU | AMD Ryzen 5 5500U (12 threads) |
| RAM/disco | — / **btrfs** (Fedora Workstation 44, kernel 7.2.7) |
| Toolchain | rustc 1.98.1 (edição 2024, perfil release com `lto="fat"`, `codegen-units=1`, `overflow-checks=true`) |
| `kd` | v0.3.3 (`target/release/kd`) |
| Corpus | sintético determinístico: ~70% conhecimento, ~20% itens de trabalho, ~10% épicos, com âncoras/tags/arestas |
| Config | a que o `kd init` grava (provider `http`, mode `lazy`, sem servidor de embeddings) |

**Corpus efetivo:** `N=200` → 235 notas (em disco); `N=1000` → 1167 notas.

> Limitações: máquina de desenvolvimento (não isolada; há variância entre execuções), corpus
> sintético e sem servidor de embeddings. Serve para **ordenar** custos e achar gargalos, não
> para cravar números absolutos de produção.

## TL;DR

1. **`maintenance doctor`, `doctor --audit` e `compact` são O(N²)**: 167 ms → **2,46 s** de
   `N=235` para `N=1167`. É o pior gargalo do sistema, disparado por `propose_merges`
   (`write/dedup/merges.rs`), que roda BM25 sobre o índice inteiro **para cada nota**.
2. **Todo comando não-`maintenance` paga uma varredura O(N) escondida** no auto-drain ocioso
   (`commands/idle.rs`). Em `N=1167`, desligá-lo corta `prime`/`self version` de ~40–48 ms para
   ~13 ms. Afeta até comandos que não têm relação com embeddings.
3. **Todo comando reconstrói `Index` + `Graph` lendo todas as notas do disco** (`Index::from_store`
   / `Graph::build`), em vez de usar o `.idx/` derivado. `Index::build` custa ~8 µs/nota e
   `Note::parse` ~6,6 µs/nota; a leitura+parse+stats domina o custo variável.
4. **`rewind` é a ação mais cara que não é quadrática**: 362 ms em `N=1167` (4,5× o custo em
   `N=235`) porque lê o **corpo de todas as notas** e calcula frescor no caminho default.
5. O **piso fixo** de uma invocação é ~4 ms (`--help`, só clap) a ~13 ms (clap + `logging::init` +
   `Session::open`). Processo + `git` (resolução de projeto) + subscriber de log explicam boa
   parte do custo das ações baratas.

## Progresso (A/B por tarefa)

| Tarefa | Ação | Antes | Depois | Ganho |
|---|---|---:|---:|---:|
| E15-T02 (O1) | `rewind` (N=1167) | 362 ms | 295 ms | −18 % |
| E15-T02 (O1) | `rewind --files` (N=1167) | 164 ms | 101 ms | −39 % |
| E15-T02 (O1) | `ask --anchor` (N=1167) | 129 ms | 96 ms | −26 % |
| E15-T03 (O1.5) | `self version` (N=1167) | 42,9 ms | 3,5 ms | −92 % |
| E15-T03 (O1.5) | `prime` (N=1167) | 43,6 ms | 2,5 ms | −94 % |
| E15-T04 (O3) | `doctor` (N=1167) | 4,87 s | 4,22 s | −13 % |
| E15-T04 (O3) | `compact` (N=1167) | 2,44 s | 2,20 s | −10 % |
| E15-T05 (O6.1/O6.2) | consistência (`sort_unstable` + capacidade) | — | — | neutro no e2e |
| E15-T06 (O2) | índice invertido + hoisting BM25 | — | — | neutro no e2e (corpus denso) |
| E15-T07 (O2.3) | glob com DP de uma linha | 223 ns/call | 138 ns/call | micro (e2e no ruído) |
| E15-T08 (O4) | `normalize` / `body_hash` / `note_id` | 1,98 / 2,92 / 1,17 µs | 0,22 / 0,47 / 0,18 µs | −89 % / −84 % / −84 % |
| E15-T09 (O5) | `rewind` (N=1167) | 319 ms | 134 ms | **−58 %** |
| E15-T09 (O5) | `rewind --json` (N=1167) | 309 ms | 113 ms | **−63 %** |
| E15-T20 (O8) | `task graph` (N=1167) | 115 ms | 77 ms | −33 % |
| E15-T20 (O8) | `task list --ready` (N=1167) | 99 ms | 76 ms | −23 % |
| E15-T20 (O8) | `task::impacts` (todos os ids) | 152 µs × N | 150 µs | O(N)→O(1) por id |
| E15-T10 (O6) | `content_terms` (micro) | 2,13 µs | 1,91 µs | −10 % |
| E15-T11 (O1.6) | carga do `.idx/` por `mtime` | — | — | **não habilitada** (parse 13,8 ms > rebuild 8,5 ms) |
| E15-T11 | correção da bancada (`timed`/`--universe`/`--key`/`forget`) | mediava falhas rápidas | mede de verdade | — |
| E15-T12 (O7) | leitura paralela do corpus (N=1167) | 21,2 ms | 7,4 ms | **−65 %** |
| E15-T12 (O7) | `ask` (N=1167, `--no-idle`) | 71,7 ms | 43,0 ms | **−40 %** |
| E15-T12 (O7) | `task list --universe` (N=1167, `--no-idle`) | 50,3 ms | 24,4 ms | **−51 %** |
| E15-T12 (O7) | `task graph` (N=1167, `--no-idle`) | 49,3 ms | 34,2 ms | −31 % |
| E15-T12 (O7) | deps O7 (`memchr`…`mimalloc`) + binário `.idx/` | — | — | **rejeitadas por medição** |
| E15-T21 (O9) | `toon::parse` (micro, `entry`) | 1,66 µs | 1,52 µs | −8 % |
| E15-T21 (O9) | `Note::parse` (micro, `entry`) | 3,54 µs | 3,35 µs | −5 % |

Baseline pré-reforma preservado em [`ULTIMO-v0.3.3.md`](ULTIMO-v0.3.3.md); a bancada passou a
medir `kd doctor`/`kd drain` (a reforma CLI de E15 T13/T14 renomeou os verbos). Os recortes de
T03/T04 estão em [`e2e-t03.md`](e2e-t03.md) e [`e2e-t04.md`](e2e-t04.md) (micro: [`micro-t04.md`](micro-t04.md));
o recorte de T05 está em [`t05.md`](t05.md), o de T06 em [`t06.md`](t06.md) e o de T07 em
[`t07.md`](t07.md); a micromb de T08 em [`micro-t08.md`](micro-t08.md) e o e2e em [`t08.md`](t08.md);
o recorte de T09 em [`t09.md`](t09.md); o de T20 em [`t20.md`](t20.md) e a micromb em
[`micro-t20.md`](micro-t20.md); o de T10 em [`t10.md`](t10.md) e a micromb em
[`micro-t10.md`](micro-t10.md); o baseline corrigido e o recorte de T11 em [`t11.md`](t11.md) e
[`t11-cached.md`](t11-cached.md); o recorte de T12 em [`t12.md`](t12.md) (depois), [`t12-antes.md`](t12-antes.md)
(antes) e [`t12-full.md`](t12-full.md) (com idle).
>
> **T12 e a leitura paralela.** `Corpus::load_notes` paraleliza a leitura+parse do corpus com
> `std::thread::scope` (zero-dep) e remonta na ordem de `list_ids` (bytes idênticos). As
> dependências da Onda 7 (`memchr`, `smallvec`, `rustc-hash`, `globset`, `rayon`, `mimalloc`) e o
> formato binário do `.idx/` foram rejeitados por medição — o `mimalloc` regride (workload
> I/O-bound) e o binário economiza < 20 % porque as notas ainda são lidas para o grafo. Ver a
> seção T12 do épico.
>
> **T11 e o formato JSONL.** A validação de frescor (`mtime`) e o caminho de carga
> (`Index::load_if_fresh`/`Corpus::load_fresh`) ficam prontos e testados, mas **não** habilitados:
> decodificar o `retrieval.jsonl` custa **13,8 ms** (N=1167) contra **8,5 ms** do rebuild, então
> ligar regride `ask` (96→116 ms), `rewind` (110→111 ms) e `task graph` (100→90 ms com ruído) —
> medido em [`t11-cached.md`](t11-cached.md). O ganho real depende do formato binário de T12.
>
> **Correção da bancada (T11).** O runner media comandos **sem checar o exit**; `task list` (sem
> filtro), `task list --sort impact`/`--full-content`, `config get`/`set` e o aquecimento de
> `forget` falhavam e viravam "ganhos" falsos. Agora `timed` exige exit 0, os `task list` usam
> `--universe`, `config` usa `--key/--value` e `forget` é idempotente. Baseline corrigido em
> [`t11.md`](t11.md): `task list --universe` 90 ms, `--sort impact` 92 ms, `--full-content`
> 123 ms em N=1167 (os números de T20 para estes três eram falhas).
> **O3 e a densidade do corpus.** A peneira de postings só pula documentos **sem overlap**; no
> corpus sintético (vocabulário de 24 palavras + prefixo comum por tipo) quase todo par
> compartilha termos, então o ganho em `doctor`/`compact` é modesto. A micromb isola o efeito:
> `write::propose_merges` cai de **1,46 s** (denso, N=1167) para **1,6 ms** (esparso, cada nota com
> termos próprios) — a diferença assintótica que o `compact` real de um corpus diverso vê.
>
> **O2 e o custo do índice invertido.** `Postings::build` custa **4,26 ms** (N=1167) e só se paga
> quando a consulta é seletiva. Como o corpus sintético é denso, `score_with` cai no fallback por
> `df` (varredura) e o e2e fica neutro; o índice invertido fica pronto para vocabulário real e é
> travado pelo proptest `sieve_positions_match_scan`.

## Velocidade do sistema (ponta-a-ponta)

Medianas, corpus de 235 notas (`N=200`) e 1167 notas (`N=1000`):

| Ação | N=235 | N=1167 | ×N |
|---|---:|---:|---:|
| `--help` (piso clap) | 3,8 ms | 3,9 ms | 1,0 |
| `self version` | 27,0 ms | 47,6 ms | 1,8 |
| `prime` | 28,5 ms | 40,2 ms | 1,4 |
| `config get` | 3,1 ms | 3,7 ms | 1,2 |
| `task list` | 16,8 ms | 17,8 ms | 1,1 |
| `task list --full-content` | 17,6 ms | 22,0 ms | 1,3 |
| `task list --sort impact` | 17,4 ms | 19,7 ms | 1,1 |
| `ask --id <nota>` | 36,3 ms | 58,8 ms | 1,6 |
| `task show --id` | 47,7 ms | 82,5 ms | 1,7 |
| `task graph` | 59,5 ms | 122,4 ms | 2,1 |
| `ask` (query comum) | 43,9 ms | 112,8 ms | 2,6 |
| `knowledge rank --universe` | 59,8 ms | 112,7 ms | 1,9 |
| `knowledge tags` | 54,7 ms | 88,9 ms | 1,6 |
| `write (nova)` | 38,5 ms | 100,8 ms | 2,6 |
| `forget (soft)` | 20,7 ms | 54,9 ms | 2,7 |
| `sync` | 39,1 ms | 69,7 ms | 1,8 |
| `rewind --files` | 79,6 ms | 164,5 ms | 2,1 |
| `knowledge map --universe` | 85,8 ms | 191,2 ms | 2,2 |
| `task list --ready` | 64,3 ms | 104,0 ms | 1,6 |
| `maintenance prune --universe` | 52,8 ms | 178,1 ms | 3,4 |
| `maintenance learn --universe` | 51,1 ms | 88,1 ms | 1,7 |
| **`rewind`** | **80,7 ms** | **362,1 ms** | **4,5** |
| **`maintenance doctor`** | **167,2 ms** | **2 464 ms** | **14,7** |
| **`maintenance doctor --audit`** | **157,8 ms** | **2 475 ms** | **15,7** |
| **`maintenance compact --universe`** | **108,0 ms** | **2 490 ms** | **23,1** |

Leituras: as ações baratas são **dominadas pelo piso fixo** (por isso `×N` pequeno). As ações de
manutenção (`doctor`/`compact`) explodem por causa do algoritmo quadrático. `rewind` é a ação
"normal" mais custosa.

## Gargalos (ranqueados)

### 1. Dedup pairwise O(N²) — `doctor`, `doctor --audit`, `compact`  ← crítico

`propose_merges` (`crates/knudge-core/src/write/dedup/merges.rs`) faz, para **cada** nota, um
`index.score(...)` (O(N)) e ainda um `index.docs.iter().find` (O(N)) por hit:

```rust
for doc in &index.docs {                       // N
    for hit in index.score(&doc.statement, ...).take(MAX_CANDIDATES) {  // N
        let other = index.docs.iter().find(...);                        // N
        ...
    }
}
```

Evidência empírica (mesmo código, corpus ~5× maior, tempo ~15–23×):

| | N=235 | N=1167 | fator notas | fator tempo |
|---|---:|---:|---:|---:|
| `doctor` | 167 ms | 2 464 ms | 4,97× | **14,7×** |
| `doctor --audit` | 158 ms | 2 475 ms | 4,97× | **15,7×** |
| `compact` | 108 ms | 2 490 ms | 4,97× | **23,1×** |

`learn` **não** sofre disso: ele limita a comparação pairwise a `MAX_DOCS = 64` e ficou em 88 ms.
Candidatos de correção: comparar só dentro de clusters/âncoras, usar índice invertido para pegar
pares que compartilham termos raros, amostrar/bloquear em corpora grande ou reaproveitar o teto
de `learn`.

### 2. Auto-drain ocioso O(N) em **todo** comando não-`maintenance`  ← alto impacto

`commands/idle.rs::maybe_drain` roda **depois** de todo comando (exceto `maintenance`) e, no
default (`embeddings.provider = "http"`, `mode = "lazy"`), chama `drain` → `pending_queue`, que
faz `store.list_ids()` + `store.read()` + `body_hash` de **todas** as notas. O resultado só é
usado para, eventualmente, embedar ≤ `embeddings.batch` (32) notas.

A/B em `N=1167` (medianas; `KNUDGE_NO_IDLE=1` desliga o drain):

| Ação | default | `--no-idle` | Δ |
|---|---:|---:|---:|
| `self version` | 47,6 ms | 13,0 ms | **−34,6 ms** |
| `prime` | 40,2 ms | 12,8 ms | **−27,4 ms** |
| `write (nova)` | 100,8 ms | 69,0 ms | −31,8 ms |
| `ask` (comum) | 112,8 ms | 93,3 ms | −19,5 ms |
| `rewind` | 362,1 ms | 340,7 ms | −21,4 ms |

Detalhes que agravam:
- `maybe_drain` chama `Session::open()` **antes** de checar `KNUDGE_NO_IDLE` (resolução de projeto
  via subprocessos `git` + leitura de config), então o escape hatch ainda paga ~5 ms.
- `prime` é declarado "estático e cacheável" (`16_cli_surface.md`), mas paga a varredura do corpus.
- Com servidor de embeddings no ar, o drain ainda **embedaria** até 32 notas a cada comando
  (rede) — pior que a varredura.
- O comentário "nunca atrasa a resposta" vale para TTY, mas consumidores que esperam EOF do
  processo (pipe/`Command::output()`, o caso de agentes) **esperam** o drain terminar.

### 3. Reconstrução de `Index` + `Graph` a cada invocação  ← alto impacto, transversal

`Session::index()` faz `Index::from_store` (lê e parseia **todas** as notas) e `Session::graph()`
faz `Graph::build`. O `.idx/retrieval.jsonl` existe e é derivado/reconstruível, mas não é usado
com um teste barato de validade. Custos por componente (micromb, `N=1167`):

| Componente | Mediana | Por nota |
|---|---:|---:|
| `Note::parse` (render completo) | 6,59 µs | — |
| `Index::build` (em memória) | 9,71 ms | ~8,3 µs |
| `Graph::from_notes` | 2,73 ms | ~2,3 µs |
| `Index::serialize + parse` | 21,8 ms | ~18,7 µs |
| `retrieval::recall` (limit 5) | 2,16 ms | ~1,85 µs |
| `Index::score` (BM25) | 1,53 ms | ~1,3 µs |
| `lifecycle::structural_clusters` | 7,05 ms | ~6,0 µs |

Para 1167 notas, só ler+parsear é ~7,7 ms (fora I/O e recomputo de `df`/`avg_len`). Comandos que
não precisam do corpus inteiro (ex.: `task list`, `config`) mostram `×N ≈ 1` porque **não**
constroem o índice; os demais pagam isso sempre.

### 4. `rewind` lê o corpo de todas as notas  ← alto impacto

`rewind` (default) é 362 ms em `N=1167` contra 164 ms de `rewind --files` e ~113 ms de `ask`. O
caminho monta `bodies` para todas as notas e calcula `fresh:` (shelf-life) além do ranking do
manifest e de `next_tasks` — que, no comparador de ordenação, chama `impact(graph, id)`
(travessia de grafo) dentro do `sort_by` (`handoff/next.rs`). O corpo só é necessário para os
itens efetivamente selecionados pelo orçamento.

### 5. Piso fixo de invocação (~4–13 ms)  ← médio

`--help` (só clap) ≈ 3,9 ms; uma ação sem trabalho (`self version` com `--no-idle`) ≈ 13 ms. A
diferença inclui `logging::init` (constrói o subscriber `tracing-subscriber`) e, para a maioria
dos comandos, `Session::open` (resolução de projeto com subprocessos `git`). Para ações baratas
chamadas em laço por agentes, esse piso domina.

## Custos por componente (micromb, resumo)

Fixos (independentes de `N`):

| Componente | Mediana |
|---|---:|
| `schema::body::normalize` (~200 B) | 3,27 µs |
| `schema::body::normalize` (~1,6 KB) | 23,9 µs |
| `schema::body::body_hash` | 4,98 µs |
| `schema::id::note_id` | 1,98 µs |
| `schema::hash::short_hash` | 769 ns |
| `toon::parse` / `toon::emit` | 3,53 µs / 1,38 µs |
| `jsonl::decode` / `encode` | 1,39 µs / 1,02 µs |
| `retrieval::rrf::fuse` (3×200) | 180 µs |
| `embeddings::lightweight::embed` (384d) | 49,3 µs |
| `embeddings::vector::cosine` (384d) | 1,33 µs |
| `config::Config::parse` | 5,18 µs |
| `handoff::budget::apply` (1000 linhas) | 6,57 µs |
| `lifecycle::confidence_score` | 7 ns |

Notas:
- `normalize` e `body_hash` são a parte cara da escrita: para um lote de 100, ~0,5 ms só em NFC.
- `rrf::fuse` é surpreendentemente caro por elemento (180 µs para 600 ids) e é chamado com canais
  do tamanho do corpus.
- `Index::serialize + parse` (~2× `Index::build`) é o custo de persistência do `.idx/`; está no
  caminho de `rebuild`, não no leitura-a-leitura atual.

## Recomendações priorizadas

1. **Tornar `doctor`/`compact` não-quadráticos** (P0). Bloquear candidatos por termos raros/
   âncoras antes do `dice`, ou limitar como `learn` (`MAX_DOCS`). Ganho potencial de ~2,5 s → dezenas de ms
   em corpora de ~1 k notas.
2. **Pular o auto-drain quando ele não pode render nada** (P0): `provider = none`; índice de
   embeddings ausente/vazio **e** nenhuma configuração de embeddings; ou um sentinela barato
   (mtime/`pending` conhecido). Mover a checagem de `KNUDGE_NO_IDLE` para **antes** do
   `Session::open`. Não rodar o drain em `prime`/`self`.
3. **Usar o `.idx/` derivado com invalidação barata** (P1) em vez de `Index::from_store`/
   `Graph::build` a cada comando: guardar um manifesto de `body_hash` + `mtime` da pasta e
   recarregar o índice serializado quando válido.
4. **`rewind`: ler corpos sob demanda** (P1) — só os itens selecionados — e pré-computar
   `impact` uma vez por id em `next_tasks` (fora do comparador de `sort_by`).
5. **Reduzir o piso fixo** (P2): inicializar o subscriber de log sob demanda (só quando houver
   log) e evitar `Session::open` redundante. Ganho de ~5–9 ms por invocação.
6. **Persistência do índice mais barata** (P2): não recomputar `df`/`avg_len` no load se o
   arquivo já os traz (hoje o comentário diz que são recomputados de propósito), ou serializar só
   `id`/`body_hash`/`meta` e reconstruir `tf` se necessário.

## Reprodução

```sh
make bench            # snapshot completo (bench/ULTIMO.md)
make bench-quick      # fumaca
cargo run --release --manifest-path bench/Cargo.toml -- e2e --sizes 1000 --no-idle --out bench/e2e-noidle.md
```
