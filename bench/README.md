# Bancada de benchmark do knudge

Bancada **isolada do workspace** e **sem dependências externas** (zero crates além do
`knudge-core`). Mede, com `std::time::Instant` e `black_box`, dois níveis:

- **micromb** (`micro`): cada componente puro do `knudge-core` (schema, TOON, JSONL, retrieval,
  grafo, lifecycle, embeddings, orçamento, config) — escala com o corpus `N`.
- **ponta-a-ponta** (`e2e`): cada ação do binário `kd` real sobre um projeto temporário com
  corpus sintético determinístico (conhecimento + épicos + tarefas), medindo a latência de
  parede (processo + resolução de projeto + I/O + domínio).

> `criterion` é **não-objetivo** do projeto (E13-T09, R43): a bancada é observação, não gate, e
> não entra em `make check`/release. É uma ferramenta de diagnostic.

## Uso

```sh
make bench          # micro + e2e + qualidade, corpora 200 e 1000, 8 amostras → bench/ULTIMO.md
make bench-quality  # só a qualidade de retrieval (E16/T01) → bench/qualidade.md
make bench-sweep    # varre rrf_k × pesos da fusão (E16/T09/D179) → stdout
make bench-quick    # 1 corpus, 5 amostras

# direto, sem Makefile:
cargo build --release -p knudge-cli
cargo run --release --manifest-path bench/Cargo.toml -- all \
    --kd target/release/kd --sizes 200,1000 --samples 8 \
    --out bench/ULTIMO.md --json bench/ULTIMO.json

# só micromb / só e2e / só qualidade
cargo run --release --manifest-path bench/Cargo.toml -- micro --sizes 1000
cargo run --release --manifest-path bench/Cargo.toml -- e2e --sizes 1000 --samples 10
cargo run --release --manifest-path bench/Cargo.toml -- quality
cargo run --release --manifest-path bench/Cargo.toml -- sweep

# A/B do auto-drain ocioso (KNUDGE_NO_IDLE=1) — ver RELATORIO.md
cargo run --release --manifest-path bench/Cargo.toml -- e2e --sizes 1000 --no-idle
```

Flags: `--sizes A,B`, `--samples N`, `--kd PATH`, `--out rel.md`, `--json out.json`,
`--quality-out bench/qualidade.md`, `--quality-json bench/qualidade.json`, `--no-idle`,
`--quick`. Os comandos `kd` rodam com `HOME`/`XDG_CONFIG_HOME` dentro do projeto temporário,
`NO_COLOR=1` e `GIT_CONFIG_NOSYSTEM=1`. `KNUDGE_BENCH_KEEP=1` preserva o projeto.

## Saída

`bench/ULTIMO.md` (tabela Markdown) e `bench/ULTIMO.json` (contrato de máquina) contêm
`min/mediana/p95/máx/desvio` por operação. O relatório com a análise e os gargalos está em
[`RELATORIO.md`](RELATORIO.md); o A/B do auto-drain em [`e2e-noidle.md`](e2e-noidle.md).

A **qualidade** de retrieval (E16/T01) sai em `bench/qualidade.md`/`bench/qualidade.json`:
Recall@k/MRR/nDCG@k sobre um corpus PT-BR sintético e rotulado (tópicos × consultas). É a régua
para D172/D173/D179 e para a fusão — o baseline fica versionado no repositório. O recorte da
calibração da fusão (E16/T09/D179) fica em [`t09_fusao.md`](t09_fusao.md) e o do stemming
(E16/T11/D206) em [`t11_stemming.md`](t11_stemming.md).

## Estrutura

| Arquivo | Papel |
|---|---|
| `src/main.rs` | CLI da bancada (modos `micro`/`e2e`/`quality`/`sweep`/`all`) |
| `src/harness.rs` | medição, percentis, tabela Markdown e JSON |
| `src/fixture.rs` | geração determinística de notas e lotes JSONL |
| `src/micro.rs` | micromb dos componentes puros e escala por `N` |
| `src/e2e.rs` | benchmark das ações ponta-a-ponta (spawn do `kd`) |
| `src/quality.rs` | avaliação de qualidade de retrieval (Recall@k/MRR/nDCG@k) |
