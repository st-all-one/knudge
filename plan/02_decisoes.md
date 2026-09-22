# Registro de decisões — knudge

> Pontos que precisam ser decididos **antes** de virar código, para evitar retrabalho e bugs de borda.
> Cada decisão: opções → recomendação → o que previne.
>
> Legenda: 🔴 **bloqueante** (decidir antes de escrever código) · 🟠 **estruturante** (decidir antes de implementar a feature) · 🟡 **adiável** (pode esperar, mas registrar).
>
> Referências: `00_panorama.md` (visão) e `01_gaps-plan-rs.md` (achados da investigação).

---

## A. Identidade e IDs

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D01** 🔴 | Geração de ID | (a) aleatório base36; (b) endereçado por conteúdo `hash(type+statement)`; (c) híbrido | **(b) com escape** — hash curto do conteúdo torna o `write` idempotente sob retry do LLM; se a chave mudar, cria novo id + `superseded_by` | Duplicatas por re-tentativa; colisões; writes não idempotentes |
| **D02** 🔴 | Prefixo do ID acompanha o `type`? | (a) prefixo fixo no id; (b) id sem prefixo, tipo só no frontmatter | **(a)** — prefixo declarativo é "filtro de graça" (já decidido); **definir** que reclassificar `type` **não** reescreve o id (o id é histórico) | IDs inválidos após reclassificação; renomeação em massa |
| **D03** 🟠 | Tamanho/formato do sufixo | base36(8) fixo vs variável | **Fixo**; colisão → re-rolar (se aleatório) ou detectar no write | IDs ambíguos; parsing inconsistente |

## B. Schema e serialização (contrato de bytes)

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D04** 🔴 | Ordem canônica das chaves | (a) alfabética; (b) ordem de declaração do schema; (c) livre | **(b)** — ordem fixa no schema; round-trip preserva; nota nova insere na ordem canônica | Diffs enormes; hash inconsistente; round-trip instável |
| **D05** 🔴 | Opcional ausente vs `null` | (a) omitir; (b) `null` | **(a)** — chave ausente, nunca vazia/nula | Divergência de bytes; falsos positivos de hash |
| **D06** 🔴 | Normalização do `body_hash` | (a) bruto; (b) lowercase + colapso de whitespace; (c) + NFC; (d) incluir `statement` | **Definir por escrito** e congelar: normalizar Unicode NFC + trim + colapso de whitespace; **incluir `statement`** no hash | Dedup falho; hash que não detecta edição de título; divergência Unicode |
| **D07** 🔴 | Formato de timestamp | ISO-8601 UTC; com/sem ms; `Z` vs offset | **UTC com milissegundos e `Z`** (espelha `toISOString`) | Parsing/comparação divergente; ordenação errada |
| **D08** 🔴 | Unidade de `statement ≤ 120` | escalares Unicode / unidades UTF-16 / grafemas | **Escalares Unicode** (mais intuitivo para o LLM), documentado | Validação inconsistente; emoji/CJK passando ou falhando |
| **D09** 🟠 | Números no frontmatter | inteiro vs float; `1` vs `1.0` | Normalizar inteiros na decodificação; nunca emitir `.0` | Diff espúrio; comparação de igualdade |
| **D10** 🟠 | Escapes/Unicode no TOON | `ensure_ascii` vs raw UTF-8 | Raw UTF-8; escapar só `"` `\` e controles | Bytes divergentes; emoji quebrado |
| **D11** 🟠 | Lista vazia | arquivo zero bytes vs `"\n"` | **Zero bytes** | Diffs espúrios; leitura de linha vazia |
| **D12** 🟠 | Append e newline final | normalizar vs append cru | **Normalizar** (garante arquivo bem-formado); documentar como contrato | Linha colada/malformada |
| **D13** 🟠 | Ordem de campos em nota nova | ordem canônica vs ordem de escrita | **Canônica** (D04) | Inconsistência entre notas |

## C. Versionamento e migração

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D14** 🔴 | Aliases de campo (legado→canônico) | (a) não suportar; (b) tabela de aliases no read; (c) migração em massa | **(b)** — tabela versionada (`k→type`, `t→statement`, `c→confidence`, `cls→classification`, `ev→evidence`, …); canônico vence; `doctor --fix` normaliza | Quebra do corpus após os renames recém-feitos |
| **D15** 🔴 | Política de `schema_version` | migração on-read vs rebuild em massa | **On-read com defaults + aliases**; rebuild só quando o índice muda de formato | Downtime; corrupção em migração |
| **D16** 🔴 | Chave desconhecida no **read** | rejeitar vs tolerar (warning) | **Tolerar com warning**; `write` continua estrito | Uma nota de versão futura envenenar o `recall` |
| **D17** 🔴 | Tipo desconhecido no read | rejeitar vs tolerar | **Tolerar** (fica fora do índice ativo, recuperável por `get`) | Forward-compat; nota órfã derrubando a base |
| **D18** 🔴 | Nota/linha malformada | erro fatal vs skip-on-read | **Skip com warning**; `--strict` para CI | Uma nota ruim impedir toda leitura |
| **D19** 🟠 | Escopo do `doctor --fix` | só reportar vs corrigir | **Corrigir** o reversível (aliases, hash, âncoras quebradas, locks stale, duplicatas) | Degeneração silenciosa do corpus |

## D. Escrita, atomicidade e crash

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D20** 🔴 | Escrita atômica | tmp+rename vs write direto | **tmp+rename no mesmo diretório** | Corrupção se o processo morrer no meio |
| **D21** 🔴 | Ordem de commit multi-arquivo | nota→eventos→container vs eventos primeiro | **Nota primeiro, evento depois** (evento é auditoria, não verdade); container derivado | Estado inconsistente após crash; evento apontando para nota inexistente |
| **D22** 🟠 | `fsync` | por write vs batch | **Batch** (page cache no write, fsync no rebuild/sync) | Gargalo de I/O; perda de até N eventos em crash (aceitável) |

## E. Concorrência

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D23** 🔴 | Lock advisory | (a) nenhum; (b) por arquivo; (c) por base | **(b) por arquivo-alvo** — CLI e MCP são dois processos; custo baixo | Lost update; corrupção quando CLI+MCP rodam juntos |
| **D24** 🔴 | Reclaim de lock stale | unlink direto vs rename sidecar + inode/mtime | **Rename sidecar + comparação de inode/mtime**; nunca apagar lock alheio | Dois escritores "vencendo" o reclaim |
| **D25** 🟠 | Ordem de aquisição de locks | ad hoc vs documentada | **Documentar** (ex.: externo=container, interno=nota) | Deadlock ABBA |
| **D26** 🔴 | Dedup | on-write vs on-read vs ambos | **On-write** para notas + **on-read** para `events.jsonl`/containers | Duplicatas sob merge de git; writes concorrentes |
| **D27** 🟠 | Rebuild vs leitores | in-place vs double-buffer | **Double-buffer** (`.idx.new/` + rename) | Leitor ver índice pela metade |
| **D28** 🟠 | Container | lista materializada vs deltas/eventos | **Deltas/eventos** (idempotente sob merge) | Lost update em adição concorrente de membro |

## F. Git, diretórios e persistência

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D29** 🔴 | Local do `.knudge/` em worktrees | por worktree vs worktree principal | **Worktree principal** (via `git rev-parse --git-common-dir`); submódulo não conta | Conhecimento fragmentado ou perdido entre worktrees |
| **D30** 🔴 | Exclusão do git | `.gitignore` vs `.git/info/exclude` | **`.git/info/exclude`** (por-clone, não versionado); excluir `.idx/`, `cache/`, `*.lock`; **verificar semântica por worktree** | Poluir o projeto para todos; rastro no repo |
| **D31** 🟠 | `merge=union` | quais arquivos | **`events.jsonl`** (append-only); containers via deltas; `.idx/` gitignored | Conflito de merge em log/containers |
| **D32** 🟠 | `sync` | o que commitar, mensagem, worktree | Commit de `notas/`+`eventos/`; mensagem gerada do evento; guard de worktree | Commit no worktree errado; mensagem inconsistente |
| **D33** 🟠 | Fonte do `learn()` | git diff vs eventos; working tree vs repo | **Eventos + `anchors`** (determinístico); não depender do diff global | Sugestões redundantes/ambíguas com múltiplas sessões |
| **D34** 🔴 | Semântica de `persist_in_project` | físico vs versionado | `true` = `.knudge/` versionado; `false` = criado mas excluído (local-only); **documentar** que os arquivos continuam em disco | Confusão "excluído" = "não criado" |

## G. Retrieval e ranking

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D35** 🔴 | Motor lexical | TF-IDF vs BM25 | **BM25** (`k1=1.5, b=0.75`) — melhor para notas curtas/técnicas | Ranking pior que o necessário |
| **D36** 🔴 | Tokenização | ASCII (`\w` JS) vs Unicode | **ASCII explícita** (replicar `\w`); documentar acentos | Divergência de tokenização; `café`→`caf` |
| **D37** 🟠 | IDF por campo/tipo | global vs por campo | **Peso por campo** (título/`statement` domina) + IDF por tipo | Termos de `decision` poluindo ranking de `fact` |
| **D38** 🟠 | Boost por confirmação | sim/não; fórmula | `score * (1 + 0.1 * (success + partial*0.5))` | Conhecimento validado não subir no ranking |
| **D39** 🟠 | `why surfaced` no `recall` | (a) não; (b) 3ª coluna | **(b)** — `id\|statement\|score\|why` (file_match/stars/recent) | LLM não sabe por que o hit apareceu; puxa errado |
| **D40** 🔴 | Orçamento de tokens no `prime` | sim/não; unidade | **Sim**; `ceil(len/4)`; budget default 4000; prioridade tipo→classe→score→ts | Contexto estourando — contradiz a tese do sistema |
| **D41** 🟠 | Auto-context-scope / auto-flip | manual vs automático | **Automático** (`git status -uall` + active work; flip se >100 notas/>5 containers) | LLM tendo que formular a query que o sistema já sabe |
| **D42** 🟡 | Embeddings | quando ligar | **Após `learn()` e dedup semântico**; sempre derivado/opcional | Peso morto; dependência prematura |

## H. Ciclo de vida e saúde

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D43** 🔴 | Decay de âncoras | sim/não; parâmetros | **Validar no rebuild**; demover após grace se fração válida < threshold | Notas órfãs acumulando; `prime(files)` não achando |
| **D44** 🟠 | Shelf life por classificação | 2 vs 3 classes | Adicionar **`observational`** (vida curta) além de foundational/tactical | Conhecimento tático/efêmero nunca expirando |
| **D45** 🔴 | Supersessão com ciclos | ignorar vs SCC | **Detectar ciclos** (Tarjan/DFS); membros de ciclo não demovem | Demover conhecimento válido; loop infinito no `expand` |
| **D46** 🟠 | Integridade do grafo | não checar vs checar | `referential-integrity` + bidirecionalidade `sup↔superseded_by` + dangling | Arestas apontando para nada; inconsistência |
| **D47** 🟠 | `compact` | proposta vs imposição; estratégias | **Propõe** (concat/keep_latest/merge_outcomes); agente aceita | Fusão destrutiva sem revisão |
| **D48** 🟠 | Outcome/confirmação | escalar+contador vs `outcomes[]` | **`outcomes[]`** com status/duration/agent/notes; confirmação **derivada** | Perda de histórico; contador sem evidência |

## I. Grafo e arestas

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D49** 🔴 | Arestas explícitas vs extraídas | (a) só regex; (b) só explícitas; (c) híbrido | **(c) explícitas primárias + regex como sugestão** — o `expand` confia no explícito; `audit` sugere | Falso positivo poluindo o grafo para sempre |
| **D50** 🟠 | Onde a extração roda | write vs rebuild | **No write** (custo do escritor), gravando como sugestão revisável | Custo no leitor; sugestões não auditáveis |
| **D51** 🟠 | Vocabulário de arestas | fechado vs aberto | **Fechado** (`ref, dep, contra, sup, ext, repl, rej, res`) | Vocabulário inflado; retrieval degradado |

## J. Tarefas e planos

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D52** 🔴 | Semântica de container/plan | lista materializada vs view derivada | **View derivada** (id + eventos de filiação); backref delimitado por marcador | Lost update; perda da lista |
| **D53** 🟠 | Ciclo de vida do plan | mínimo vs completo | Especificar **submit/adopt/reorder/release/outcome/review**, profundidade máxima, `blocks` 1-based, self-reference | Regras implícitas; plano inconsistente |
| **D54** 🟠 | `checks`/validators | prosa vs catálogo | **Catálogo executável** + resolução por `AGENTS.md` + `anchors`; severidade | Critério interpretável; fechamento por declaração |
| **D55** 🟠 | Fechamento | declaração vs evidência | **Evidência** (roda validators, infere `outcome`) | Task "fechada" sem verificação |
| **D56** 🟡 | Scheduling (`ready`) | `exp` vs `not_before` | `not_before` separado de `expires_at` (se surgir necessidade) | Conflar expiração com agendamento |

## K. Prime, protocolo e hooks

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D57** 🔴 | Onde vive o protocolo | no `prime` vs no `onboard` | **`onboard`** (uma vez, no `AGENTS.md`); `prime` só estado (~30 tokens) | Pagar 400 tokens de protocolo toda sessão |
| **D58** 🟠 | Session-close | checklist no `prime` vs hook | **Footer no `prime`** (curto) + hook opcional | LLM esquecer de externalizar |
| **D59** 🟠 | Hooks | existem? quais eventos | **Opcionais**: `pre-record`, `post-record`, `pre-prime`, `pre-prune`, `pre-compact`; stdin JSON; timeout + process-group kill; redaction | Automação de ciclo de vida; segredos no log |
| **D60** 🟠 | `onboard` | texto solto vs marcadores | **Marcadores idempotentes** (`<!-- knudge:start/end -->` + version marker) | Duplicação; onboard não idempotente |

## L. Config

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D61** 🔴 | Níveis de config | só global vs global+projeto | **Global (template) + projeto (efetivo, precedência)**; projeto clona o global na instanciação | Comportamento global não sobrescrevível por projeto |
| **D62** 🔴 | Clone vs merge na instanciação | cópia literal vs diff | **Cópia literal** (simples); merge fica como evolução futura | Semântica ambígua de "projeto parcial" |
| **D63** 🟠 | Reescrita do TOML | ordem/quoting estáveis? | Congelar writer (ordem canônica, quoting) | Diffs espúrios em `config set` |
| **D64** 🟠 | `config set/unset` | validar path vs livre | Validar contra o schema; podar ancestrais vazios; revalidar | Config inválida gravada |

## M. Arquitetura e distribuição

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D65** 🔴 | Separação núcleo/adaptador | monolito vs core+adapters | **`knudge-core` puro + `knudge-cli` + `knudge-mcp`**; ports `Clock/Rng/Git/Fs/Embedder` | Testes não determinísticos; MCP acoplado |
| **D66** 🔴 | Stack | Rust vs Python (MVP) | Decidir; o MVP do panorama é Python, mas os `plan-rs` apontam Rust para distribuição/embedding | Retrabalho de reescrita; distribuição travada |
| **D67** 🟠 | Nome do binário/crates | `kb` vs `knudge` | Unificar (`knudge`? `kb`?) | Inconsistência de docs e CLI |
| **D68** 🟡 | Embeddability | MCP / FFI / WASM | **MCP primeiro** (já é o caso de uso), FFI/WASM depois | Overengineering prematuro |
| **D69** 🟡 | Distribuição | source vs binário estático | Binário estático + `completions` + `setup` + `upgrade` | Atrito de adoção |
| **D70** 🟡 | Migração | manual vs `migrate-from-*` | Comando de migração de seeds/mulch/brainstorm | Conhecimento existente abandonado |

## N. Contrato de saída e erros

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D71** 🔴 | Saída para máquina | só pipe vs pipe+JSON | **Pipe para LLM + `--json`** (envelope `{success, command, error}`) para MCP/scripts | MCP sem saída estruturada |
| **D72** 🟠 | Catálogo de mensagens | livre vs congelado | **Congelado por teste** (o LLM lê erros) | Regressão silenciosa de mensagem |
| **D73** 🟠 | EPIPE | erro vs exit 0 | **Exit 0** em pipe fechado (ex.: saída truncada por `head`) | Stacktrace/exit errado em pipes |

## O. TOON

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D74** 🔴 | Parser/emitter TOON | biblioteca vs próprio | **Próprio**, com subconjunto documentado + corpus/golden | Sem referência; divergência de bytes |
| **D75** 🟠 | Fallback | TOON vs YAML/JSON | Definir fallback e detecção de versão | Incompatibilidade com frontmatter existente |

## P. Testes e qualidade

| # | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|
| **D76** 🔴 | Estratégia de teste | unit only vs golden+property+stress | **Golden/snapshot + property tests + stress de concorrência + crash-injection** | Bugs de borda não detectados |
| **D77** 🟠 | Documentação de bordas | informal vs `DIVERGENCES.md` | **`DIVERGENCES.md` do knudge** (Unicode, hash, ordem, lock, atomicidade, TOON) | Repetir erros já catalogados |
| **D78** 🟠 | Matriz de aceite | ad hoc vs por tool | **Matriz por tool**: formato pipe, JSON, erro, exit, estado do `.knudge/` | Feature "pronta" sem contrato verificado |

---

## Matriz de bloqueio — ordem mínima de decisão

```
D01,D04–D08,D14–D18  (identidade + contrato de bytes + migração)
        │
        ▼
D20–D28  (escrita, lock, dedup, rebuild)
        │
        ▼
D29–D34  (git, worktree, persistência)
        │
        ▼
D35–D42  (retrieval, ranking, prime)
        │
        ▼
D43–D51  (ciclo de vida, grafo)
        │
        ▼
D52–D56  (tarefas e planos)
        │
        ▼
D57–D64  (prime/hooks, config)
        │
        ▼
D65–D78  (arquitetura, distribuição, contrato, testes)
```

**Regra:** nada de D52 em diante deve ser implementado antes de D01–D18 estarem fechadas — são o alicerce de bytes e identidade que tudo o resto assume.

---

## As 10 decisões que mais previnem dor

1. **D14 aliases** — sem isso, os renames de schema quebram o corpus.
2. **D04–D08 contrato de bytes** — ordem, `null`, hash, timestamp, contagem.
3. **D16–D18 leitura tolerante** — uma nota ruim não derruba a base.
4. **D23–D26 lock + dedup** — CLI e MCP são dois processos.
5. **D35–D40 BM25 + boost + orçamento** — o núcleo do valor.
6. **D43/D45 decay + ciclos** — evita apodrecimento e demolição de conhecimento válido.
7. **D49 arestas explícitas** — corrige a fragilidade da regex.
8. **D65/D66 core+adapter e stack** — define o que é reescrever depois.
9. **D71 saída para máquina** — habilita o MCP proativo.
10. **D29/D30 worktree + exclude** — define onde o conhecimento vive de verdade.
