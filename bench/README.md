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
make bench          # micro + e2e, corpora 200 e 1000, 8 amostras → bench/ULTIMO.md
make bench-quick    # 1 corpus, 5 amostras

# direto, sem Makefile:
cargo build --release -p knudge-cli
cargo run --release --manifest-path bench/Cargo.toml -- all \
    --kd target/release/kd --sizes 200,1000 --samples 8 \
    --out bench/ULTIMO.md --json bench/ULTIMO.json

# só micromb / só e2e
cargo run --release --manifest-path bench/Cargo.toml -- micro --sizes 1000
cargo run --release --manifest-path bench/Cargo.toml -- e2e --sizes 1000 --samples 10

# A/B do auto-drain ocioso (KNUDGE_NO_IDLE=1) — ver RELATORIO.md
cargo run --release --manifest-path bench/Cargo.toml -- e2e --sizes 1000 --no-idle
```

Flags: `--sizes A,B`, `--samples N`, `--kd PATH`, `--out rel.md`, `--json out.json`,
`--no-idle`, `--quick`. Os comandos `kd` rodam com `HOME`/`XDG_CONFIG_HOME` dentro do projeto
temporário, `NO_COLOR=1` e `GIT_CONFIG_NOSYSTEM=1`. `KNUDGE_BENCH_KEEP=1` preserva o projeto.

## Saída

`bench/ULTIMO.md` (tabela Markdown) e `bench/ULTIMO.json` (contrato de máquina) contêm
`min/mediana/p95/máx/desvio` por operação. O relatório com a análise e os gargalos está em
[`RELATORIO.md`](RELATORIO.md); o A/B do auto-drain em [`e2e-noidle.md`](e2e-noidle.md).

## Estrutura

| Arquivo | Papel |
|---|---|
| `src/main.rs` | CLI da bancada (modos `micro`/`e2e`/`all`) |
| `src/harness.rs` | medição, percentis, tabela Markdown e JSON |
| `src/fixture.rs` | geração determinística de notas e lotes JSONL |
| `src/micro.rs` | micromb dos componentes puros e escala por `N` |
| `src/e2e.rs` | benchmark das ações ponta-a-ponta (spawn do `kd`) |
