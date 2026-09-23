# knudge

**knudge** é uma CLI Rust (`kd`) de **memória por projeto otimizada para LLM**: notas em
**Markdown como verdade**, índice derivado reconstruível e busca híbrida (BM25 + âncoras +
embeddings opcional via RRF) — sem servidor, sem banco, sem daemon. Feito para o agente
**buscar antes de gravar**, registrar decisões/fatos/erros e planejar tarefas com contexto
mínimo.

## Instalação

```bash
curl --proto '=https' \
     --tlsv1.2 \
     --show-error \
     --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh \
  | bash
```

> Requisito: `git` no projeto. Rust 1.97+ só para compilar do source; embeddings são opcionais.

Instala `kd` + `knudge-mcp` em `~/.local/bin` (release pré-compilado, checksum SHA-256).
Versão fixa: `... | VERSION=v0.2.0 bash`. Do source: `make install` (ou `./install.sh --from-source`).

## Quickstart

```bash
# Fundar a memória do projeto (ancorada na raiz do git)
kd init

# Buscar antes de gravar
kd ask "como o gateway limita requisições" --brief

# Gravar um fato (dedup automático: <0.75 cria, 0.75–0.92 merge, >=0.92 rejeita)
kd write --type fact "Rate limit é 100 rps por chave" --tag gateway --anchor src/gateway.rs

# Planejar e executar
kd task new "Sync offline-first" --scope epic
kd task new "Resolver conflito de merge" --scope task --parent <epic>
kd task list --ready --sort impact
kd task close <task> --outcome success --note "testes verdes"

# Retomar contexto entre sessões (handoff com orçamento de tokens)
kd rewind --budget 2000

# Mapa de conhecimento (clusters estruturais + semânticos)
kd knowledge map --axis container --semantic

# Manutenção (só propõe)
kd maintenance doctor --audit
kd maintenance learn

# Versionar notas/ + eventos/
kd sync
```

Protocolo completo (o "help da IA"): `kd prime` (ou `kd`). Busca sem embeddings funciona: o
`ask` usa BM25 + âncoras.

## O que esta ferramenta faz?

**O knudge dá memória durável ao agente, por projeto.** A verdade são **arquivos Markdown**
(`.knudge/notas/`); todo o resto (índice BM25, embeddings, grafo) é **derivado** e
reconstruível. Não há servidor, banco nem daemon — só o binário `kd`.

O ciclo é **buscar → gravar → executar → commitar**:

- **`kd ask`** — busca híbrida (filtros → BM25 → âncoras → RRF), `--id` para corpos, `--around`
  para expandir o grafo, `--json` para máquinas.
- **`kd write`** — create idempotente com dedup (0.75/0.92); `--update` versiona; `--link` cria
  aresta explícita.
- **`kd task`** — hierarquia `plan ⊃ epic ⊃ issue ⊃ task`, com rollup de progresso por épico e
  fechamento por evidência.
- **`kd rewind`** — handoff ponto-no-tempo dentro de um orçamento de tokens, retomável 1:1.
- **`kd knowledge`** — mapa de clusters (estrutural e semântico).
- **`kd maintenance`** — `doctor`/`learn`/`compact`/`prune` **propõem**; nunca mudam o corpus
  sozinhos.

O `id` de cada nota é derivado do conteúdo (`<tipo>_<base36(8)>`); reclassificar o `type` não o
reescreve. O contrato de bytes (TOON) é congelado por testes.

### Casos de uso

| Cenário | Como usar | Benefício |
|---|---|---|
| **Lembrar uma decisão** | `kd write --type decision "..."` | Conhecimento durável, id estável e arestas |
| **Achar o que já se sabe** | `kd ask "..." --brief` | Busca híbrida com score e `why`; `--json` para agentes |
| **Planejar trabalho** | `kd task new ... --scope epic/task` | WBS com progresso por épico e bloqueios |
| **Retomar contexto** | `kd rewind --budget 2000` | Handoff dentro do orçamento de tokens |
| **Evitar duplicata** | `kd ask "<rascunho>"` antes do `write` | Dedup lexical 0.75/0.92 |
| **Auditar a base** | `kd maintenance doctor --audit` | Integridade + arestas sugeridas |
| **Revisão por IA** | `knudge-mcp` | Hints-ponteiro antes de gravar/editar |

## Embeddings (opcional)

O `ask` funciona **sem embeddings** (BM25 + âncoras + RRF); a busca semântica é um canal
derivado que melhora perguntas em linguagem natural. Sem provedor, degrada para lexical com
`warnings[]`.

Modelo recomendado: `ibm-granite/granite-embedding-97m-multilingual-r2` (384d, Apache-2.0,
multilíngue com PT), servido por `llama.cpp` com **`--pooling mean`** e **`-ub 2048`**:

```bash
llama serve -m models/granite-97m-r2-Q8_0.gguf --embeddings --pooling mean -b 2048 -ub 2048 --port 8084

kd config set embeddings.provider http
kd config set embeddings.model ibm-granite/granite-embedding-97m-multilingual-r2
kd config set embeddings.dimensions 384
kd config set embeddings.endpoint http://127.0.0.1:8084/v1/embeddings

kd maintenance index --drain
kd ask "como o servidor não vê o conteúdo das notas"
```

> O `-ub` (µbatch físico) do `llama.cpp` é **512** por default e o `/v1/embeddings` rejeita a nota
> inteira acima disso — o `drain` então falha com `indexed=0`. Suba com `-ub 2048` (≥ o maior corpo).

Assíncrono e lazy: notas novas ficam `pending` e, com `embeddings.mode=lazy` (default), o CLI
drena **um lote** ao fim de cada comando (auto-drain ocioso) — o `kd maintenance index --drain`
esvazia o resto. `mode=manual` só drena sob `--drain` explícito. Para desligar o canal:
`kd config set recall.semantic false`. Veredito da bancada e A/B em
[`plan/04_embeddings.md`](plan/04_embeddings.md).

Para drenar também quando você **não usa** o `kd` (worker contínuo), instale o timer de usuário
— ele garante o servidor local e drena a fila periodicamente:

```bash
scripts/knudge-idle.sh install --project /caminho/do/projeto   # systemd --user; --every 1h
scripts/knudge-idle.sh status
scripts/knudge-idle.sh uninstall
```

Ou pelo próprio binário — ele **pergunta antes** de agir e embute o worker (sem download):

```bash
kd maintenance watch-service --install      # pré-flight + timer + cadastra este projeto
kd maintenance watch-service --subscribe    # cadastra outro projeto (multi-projeto)
kd maintenance watch-service --unsubscribe  # descadastra (mantém o sistema instalado)
kd maintenance watch-service --status       # saúde: timer, servidor, fila por projeto (default)
kd maintenance watch-service --uninstall    # remove o sistema
```

## MCP (agentes de IA)

`knudge-mcp` serve os gatilhos de memória por **JSON-RPC 2.0 sobre stdio** — sem servidor de
rede. Configure no cliente com `kd self setup <claude|cursor|codex|pi>`.

Tools: `knudge_pre_write` (quase-duplicados antes de gravar), `knudge_pre_edit` (working set
antes de editar), `knudge_session_end` (fim de sessão) e `knudge_status`. Os hints são
**ponteiros** (`id + statement + score`) — o conteúdo fica no `kd`, nunca no contexto do modelo.

## Destaques

- **Busca híbrida com pesos** — BM25 + âncoras + vetor fundidos por RRF (peso por canal, D124).
- **Contrato de bytes congelado** — TOON + IDs por hash; mudança exige golden/proptest (D95).
- **Determinístico** — testes de core usam fakes (relógio/RNG/FS), reprodutíveis byte a byte.
- **Local e leve** — binário estático, sem `tokio`/`reqwest`/servidor; MCP é stdio.
- **Degradação graciosa** — canal opcional fora do ar vira `warnings[]` (e `strict` promove a erro).
- **Distribuição sem Docker** — binários otimizados para Linux (x86_64/ARM64), macOS (Apple
  Silicon/Intel) e Windows (x86_64/ARM64).

## Superfície

```
kd              # = kd prime (protocolo estático, byte-idêntico)
kd init         # funda .knudge/ + bloco no AGENTS.md
kd rewind       # estado/handoff ponto-no-tempo
kd ask          # toda pesquisa (recall + get + expand)
kd write        # toda escrita (create + update + arestas)
kd task         # plan / epic / issue / task
kd knowledge    # mapa de conhecimento (clusters)
kd maintenance  # doctor, compact, eval, index, learn, prune
kd config       # .knudge/config.toml
kd forget       # soft-delete / restore
kd sync         # commit de notas/ + eventos/
kd self         # setup, completions, upgrade, version
```

Contrato congelado: [`plan/implementation/16_cli_surface.md`](plan/implementation/16_cli_surface.md).

## Desenvolvimento

```bash
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas
make dist      # release otimizado + pacote da plataforma atual em dist/
```

Contribuir: [`AGENTS.md`](AGENTS.md) · Arquitetura: [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Documentação

| Doc | Conteúdo |
|---|---|
| [`SKILL.md`](SKILL.md) | Guia de uso ativo para agentes de IA. |
| [`llms.txt`](llms.txt) | Índice para modelos de linguagem. |
| [`plan/implementation/16_cli_surface.md`](plan/implementation/16_cli_surface.md) | Referência completa da CLI. |
| [`plan/implementation/17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md) | Matriz por verbo (pipe/`--json`/exit/estado). |
| [`TOON.md`](TOON.md) | Contrato de bytes. |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | Arquitetura interna. |
| [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md) | Decisões D01–D130. |
| [`plan/04_embeddings.md`](plan/04_embeddings.md) | Embeddings, modelos e A/B. |
| [`DIVERGENCES.md`](DIVERGENCES.md) | Bordas + testes que as travam. |
| [`AGENTS.md`](AGENTS.md) | Contribuir no código do knudge. |
| [`CHANGELOG.md`](CHANGELOG.md) | Histórico de versões. |

## Licença

[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE) — uso livre.
