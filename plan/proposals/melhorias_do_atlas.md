# Melhorias inspiradas no atlas — proposta revisada

> **Status:** revisão em profundidade (2026-11-17) **implementada em 0.6.6**. Entraram os itens §3
> (D226), §1 (D227), §6 (D228) e §4 (D229, só medição); §5/§7 ficaram diferidos e §2 rejeitado
> (D152 permanece). Nenhum item rejeitado ressurgiu por "implementação disfarçada".
>
> **Origem:** análise do `github.com/bloodfeast/atlas` v0.7.0 (Apache-2.0, Rust, MCP stdio + hooks
> do Claude Code). O atlas resolve problemas que o knudge **não** tem (roteamento de output de
> ferramenta, índice de código por símbolo) e um que o knudge resolve melhor (memória de projeto).
> Esta revisão mantém só o que sobrevive ao confronto com a **versão atual** do projeto.
>
> **Base:** v0.6.6 (`Cargo.toml`), `D01–D229`, `R01–R45`, superfície v3 (D209),
> [`ARCHITECTURE.md`](../../wiki/specs/ARCHITECTURE.md),
> [`16_cli_surface.md`](../implementation/16_cli_surface.md),
> [`17_matriz_aceitacao.md`](../implementation/17_matriz_aceitacao.md),
> [`reranking_ann.md`](reranking_ann.md) §1, [`24_plano_mestre_0.5.0.md`](../implementation/24_plano_mestre_0.5.0.md)
> §3. Políticas que decidem: D06/D95 (`body_hash`), D14/D15/D84 (sem retrocompatibilidade; `.idx/`
> derivado), D20–D27 (tmp+rename, nota antes do evento, double-buffer), D39/D71/R20 (pipe,
> envelope, stdout=dados / stderr=logs), D46/D19 (doctor advisory, `--fix` só no reversível), D48 (`revision`
> não é CAS), D57/D88 (`prime` byte-idêntico, `context_id` 1:1), D86 (`content_hash` de âncora),
> D92 (≤300 linhas, sem `unwrap`), D101/R16/R43 (sem inferência in-process; sem ganho medido →
> reverte), **D152 (observabilidade de busca foi explicitamente removida)**, D161 (corpo visível no
> `ask`), D207/D208 (proveniência, TMS), D209/214/218–223 (superfície v3, fachada, shards por
> agente), **R45 (invariantes de retenção/reescrita: default identidade, leitura nunca escreve,
> supersessão vence, transformação não deleta a fonte)**.

---

## 0. Veredito executivo

| # | Item | Veredito | Onde vive | Razão curta |
|---|---|---|---|---|
| 3 | Deny-list fixa de segredos em âncoras/hints | **ADOTAR** | `core::store::paths` + 3 chamadas | R22 cobre log, não cobre caminho; função pura, risco alto evitado |
| 1 | Refs `id@body_hash` + verificação no `rewind` | **ADOTAR reformulado** | `handoff` + `doctor --refs` (opt-in) | Fecha o único furo real do handoff; precisa de flag para não quebrar D88 |
| 6 | Guarda de concorrência na `update` | **ADOTAR na forma mínima** | `write --expect-body-hash` | `ErrorKind::Conflict`/exit 4 **já existem**; um flag, não dois |
| 4 | Custo por resposta | **ADOTAR só a medição** | `bench/custo.md` (fora do runtime) | A tese merece número; o ledger e o verbo novo violam R45 e a superfície v3 |
| 5 | Colapso de quase-duplicatas no `rewind` | **DIFERIR condicionado** | `handoff` (se aprovado) | Depende de índice quente → não determinístico; falta medir tokens liberados |
| 7 | Piso/orçamento de hints MCP | **REJEITAR por ora** | — | Depende de §2 (morto) e de bancada com folga (não existe) |
| 2 | `.knudge/misses.jsonl` durável | **REJEITAR** | necessidade real vai para `bench/` | Reabre D152, cria segunda verdade fora de `notas/`, faz `ask` escrever (R45) |

Os três primeiros entram. O quarto entra **como medição, não como feature**. Os dois seguintes
ficam atrás de evidência que ainda não existe. O §2 não entra: ele é a reedição de uma decisão que
o projeto já tomou contra.

### 0.1 O filtro filosófico

A tese do knudge ([`00_panorama.md`](../00_panorama.md) §1) é externalizar **conhecimento** para
otimizar o **contexto injetado** — não observar o próprio tool. Quatro consequências decidem quase
todos os itens acima:

1. **Conhecimento ≠ metaconhecimento do tool.** Misses, ledgers e contagem de hints são sobre o
   knudge, não sobre o projeto. O lugar deles é `bench/` e docs, nunca `.knudge/`. O agente não
   consegue `kd ask` um miss — ele não é nota, não tem id, não tem schema, não entra no grafo.
2. **Índice é derivado; leitura não escreve** (R45 #1 e #7). Nenhum `ask`/`rewind` pode criar
   arquivo durável só por ser executado. O que não é reconstruível de `notas/` + `eventos/` é uma
   segunda verdade — e o projeto só admite uma.
3. **Bytes são contrato.** `render_item` hoje emite `id|statement|tier` (D39/D57/D106); qualquer
   sufixo novo muda `context_id` (derivado do texto, D88) e quebra goldens. Opt-in ou `Dxx` — nunca
   mudança silenciosa.
4. **Barato construir ≠ barato manter.** Item "P1 baixo custo" ainda custa golden, doc,
   `prime`, `MODULE.md`, `llms.txt`, `CHANGELOG` e atenção por anos. O orçamento de superfície da
   v3 (14 verbos, D209) é parte do produto, não burocracia.

### 0.2 Premissas do rascunho que a versão atual corrigiu

| O que o rascunho assumia | Realidade verificada | Efeito |
|---|---|---|
| Base pré-0.5, até D136/R43 | v0.6.5, até D225; R45 rege retenção/reescrita | Novo gate nos itens 2, 4, 5 |
| "misses precisa de decisão explícita" | **D152 já decidiu removê-los** (`.idx/recall_stats.jsonl`, `--explain-miss`) | §2 passa a reversão de decisão, não novidade |
| "novo `ErrorKind` + exit code" para guarda | `ErrorKind::Conflict`, exit **4**, já existe e diz "revisão concorrente" | §6 perde custo e risco de contrato |
| "D136 fixa agente único" | D136 fixa **posse**; D218–D223 acrescentam **identidade de escritor** (shards, `agent_role`) | §8 precisa de justificativa nova |
| "item 1 não muda contrato" | `render_item` → `id|statement|tier`; trocar texto troca `context_id` | §1 vira opt-in para preservar D88 |
| `ask` devolve "corpos crus" no contra-factual | D161 já mostra 1 corpo cheio + 4 truncados a 280 chars | §4 precisa de definição explícita de baseline |
| "`install.sh` com SHA-256 basta" | `kd self upgrade` é real (E18/T05); `drain service --install` valida SHA-256 (D184) | §8 mantém a rejeição, troca a evidência |
| bancada saturada "do atlas" | Confirmado: `bench/qualidade.md` 100 % de nDCG@5; `reranking_ann.md` §1 já pede famílias difíceis | §2 perde a desculpa; §7 perde o medidor |

---

## 1. Refs `id@body_hash` + verificação de citações — **ADOTAR reformulado**

**Problema real (e mais estreito do que o rascunho dizia).** `rewind --resume` devolve bytes
idênticos por desenho (D88) — isso é correto para reprodutibilidade, mas o texto cita notas só por
`id` (`render_item` = `id|statement|tier`; D162 anexa corpo a `foundational`/`decision`). O agente
que retoma o handoff não consegue saber, **pelo handoff**, se a nota citada mudou de corpo. O grafo
já sabe — `replaces`/`superseded_by` (D46), `content_hash` de âncora com verify-on-hit (D86), TMS
(D208) — mas esse conhecimento vive no índice, não no snapshot. A lacuna é de *transporte* da
prova de frescor, não de ausência total de mecanismo.

**Mecanismo reimplementado dentro das R01–R45.** `body_hash` já é SHA-256 truncado a hex8 (D06/D95)
e nunca deve virar outro formato. A consequência de D01+D06 é elegante e merece ser explorada:
**mesmo `id` com `body_hash` diferente = corpo editado; `id` novo = `statement` mudou** (a chave do
id inclui `normalize(statement)`), e nesse caso o grafo já carrega `replaces`/`superseded_by`. Os
estados de verificação, portanto, **não são vocabulário novo**:

| Resultado | Condição | Já existe como |
|---|---|---|
| `changed id@antigo -> id@atual` | mesmo `id`, `body_hash` divergente | `doctor --fix` recompara hash (D19) |
| `superseded id@hash por <novo_id>` | `id` ausente, `superseded_by`/`replaces` aponta | integridade D46 |
| `retired id@hash (motivo)` | `status=forgotten`/`retired_at` | ciclo de vida D48/D84 |
| `dangling id@hash` | sem ref no índice nem tombstone | `dangling` do `audit` (D46) |

**Desenho corrigido (a mudança importante em relação ao rascunho):**

1. **`kd rewind --with-refs` — opt-in.** Default **desligado**: stdout, `context_id` e goldens de
   D88/D106/D162 ficam byte-idênticos. Ligado, cada citação vira `id@hex8` (e
   `anchor_path@content_hash` quando a linha vem de âncora, reaproveitando `.idx/anchors.jsonl`).
   Como o `context_id` deriva do texto, um handoff `--with-refs` ganha id novo — correto e honesto,
   sem tocar nos antigos.
2. **`kd doctor --refs <context_id>` — flag no verbo existente**, não subcomando. Varre as refs do
   contexto, compara com índice/âncoras e imprime **stderr** (R20) nos quatro estados acima. É
   extensão natural dos 13 checks de `doctor` (advisory, D46); `--fix` continua corrigindo só
   derivado (D19/D84), nunca conhecimento. Contexto sem refs → "nenhuma ref verificável", não erro.
3. **`kd rewind --resume <id> --check-refs`** verifica e reporta em stderr; o stdout **continua
   sendo o texto gravado, byte a byte** (D88 intacto mesmo no modo de checagem). Exit 0 salvo
   `behavior.strict` (D94) promover o aviso.

**Contrato:** `não` no default; `sim, opt-in` com `--with-refs` (exige `Dxx` + golden da variante,
não do default). O rascunho marcava "não" sem qualificar — estava errado.

**Custo.** Baixo: `body_hash` e `anchors.jsonl` já existem; a varredura é O(refs) sobre o índice em
memória; nenhum arquivo novo. Testes: golden de `--resume` puro idêntico; `parse_ref(render_ref(id,
hash)) == (id, hash)`; cenários `changed`/`superseded`/`retired`; `--check-refs` com stdout igual.

---

## 2. Registro durável de *misses* — **REJEITAR**

Esta seção existe para dizer "não" com precisão, porque é o item mais sedutor do rascunho.

**O que o rascunho propõe.** `.knudge/misses.jsonl` (fora de `.idx/`) com `{at, query, filters,
hits, top_score, why_empty}`, `kd maintenance misses`, check no `doctor --fix`, e as consultas
virando famílias da bancada. O próprio rascunho admite: "precisa de decisão explícita".

**Por que não entra.**

1. **D152 já fechou a questão ao contrário.** "`kd ask` sem hits → `[no_results]`... **A demanda
   de observabilidade de busca é removida** (sem `.idx/recall_stats.jsonl`, sem `--explain-miss`).
   Se um dia a observabilidade voltar, ela é uma decisão nova." Voltar exige mais do que um `Dxx`:
   exige a evidência que D152 não tinha e que o rascunho também não traz (nenhum número de
   lacuna real descoberta, nenhum miss que tenha virado nota).
2. **Quebra a regra de ouro.** `.knudge/misses.jsonl` não é `notas/` nem `eventos/`, mas é durável e
   não reconstruível — uma segunda verdade sem id, sem schema, sem âncoras, sem dedup, sem
   `doctor`. Onde o projeto guarda auditoria é `eventos/`; onde guarda derivado é `.idx/`. Um
   terceiro lugar durável é anomalia arquitetural, não "conhecimento de segunda ordem".
3. **Viola R45 #1 e #7.** "default identidade, inclusive sem criar derivados" e "leitura nunca
   escreve nota". `ask` com miss viraria `ask` com efeito colateral. A degradação graciosa (R33)
   e o `strict` (D94) falam de **canal falho**, não de ausência de conhecimento — o rascunho
   reconhece isso e ainda assim tenta usar o mesmo buraco.
4. **A necessidade legítima tem casa melhor.** O rascunho diz que o objetivo é "sair do nDCG@5
   saturado". Isso é **pré-requisito 3 de `reranking_ann.md` §1**, que já manda adicionar famílias
   difíceis (distratores confundíveis, consultas longas/ambíguas, paráfrases distantes) à bancada.
   Consultas reais podem ser **destiladas em famílias rotuladas** em `bench/` — sem que o runtime
   escreva coisa alguma. O ganho pretendido é obtido, o custo arquitetural desaparece.
5. **Risco R22 fora do lugar.** Query livre pode conter segredo. A redação existe no *layer de
   log*; um arquivo que não é log nem nota escapa dela. Mais uma superfície de vazamento.

**Se um dia voltar (critério de mudança de veredito):** (a) evidência de que misses reais,
repetidos, viraram conhecimento perdido — não só "a bancada está saturada"; (b) reversão explícita
de D152 com essa evidência; (c) home em `.idx/` (derivado, purgável), opt-in por config, com
redação e sem novo verbo. Até lá, `hits: []` no `--json` e `[no_results]` no pipe já são o contrato
suficiente.

---

## 3. Deny-list fixa na varredura de âncoras e nos hints — **ADOTAR**

**Problema.** R22 redige segredos no *log* (allowlist). Âncora, hint e `anchors.jsonl` não são log:
`--anchor .env`, `--anchor secrets/prod.yaml`, ou a edição de um arquivo de credenciais podem pôr
caminho sensível em nota, hint ou índice. D91 ("segredos só no global") é a mesma intenção, mas do
lado da config; falta a aplicação do lado do **caminho**.

**Mecanismo.** `is_denied(path)` pura em `knudge-core::store::paths` (ou módulo irmão de
`ignored_dirs`), lista fixa que a config **não pode reduzir**: `.env*`, `*.pem`, `*.key`, `*.p12`,
`*.jks`, `*.keystore`, `id_*`, `credentials*`, `.netrc`, `.pgpass`, `secrets.*`, `*.tfstate*`.
Pontos de uso: (a) `write --anchor` com glob que casa a lista → `invalid_input` com a lista na
mensagem (**não** `AnchorDenied`: D212 exige conjunto fechado + sugestão; e R31 já tem o código);
(b) `decay::walk_paths_ignoring` pula o caminho sem tocar em mtime; (c) `pre_edit` não emite hint
com âncora casada; (d) a remoção de âncora continua só desvinculando, nunca apagando nota (já é o
comportamento). Documentar em `ARCHITECTURE.md` §11 como extensão de R22/D91, com a frase que
importa: "a lista não é configurável para menos".

**Ajustes de precisão ao rascunho.** Casar por **basename**, não por substring (`id_*` não pode
pegue `identity/`); checar globs **resolvidos**, não só literais (`**/*.pem` também é negado);
`Exit code` 2, sem `ErrorKind` novo.

**Custo/teste.** Função pura + três chamadas + fixture adversário (`.env`, `id_ed25519`,
`server.pem`): nenhum dos nomes pode aparecer em `ask`, `pre_edit`, `anchors.jsonl` ou nota. É o
único item sem cláusula de reversão por medição — proteção não precisa de A/B.

---

## 4. Custo por resposta — **ADOTAR a medição, REJEITAR o ledger e o verbo**

**O que é legítimo.** "recall em pipe e corpo sob demanda economizam contexto" é tese, não número.
Medir é coerente com R43 e com a bancada que observa (E13-T09). O que **não** é legítimo é pagar
runtime por isso.

**O que o rascunho propõe e por que muda.** `.idx/token_log.jsonl` escrito por `ask`/`rewind` +
`kd self token-report [--since] [--tsv]`. Dois problemas: (1) R45 #1/#7 — default identidade e
leitura sem escrita; um log de leitura é exatamente o que a política proíbe; (2) superfície v3
congelada (D209) e `self` = distribuição/setup (D69), não observabilidade. Além disso, gravar por
chamada torna o número dependente do histórico da máquina, quando ele é uma **propriedade estática
de (corpus, consulta, flags)** — perfeitamente computável offline.

**Desenho corrigido.** Em `bench/` (nova `bench/custo.md`, mesmo espírito de `qualidade.md`):
para cada família rotulada, `bytes_out` = bytes exatos do stdout do comando (já determinístico), e
`bytes_counterfactual` = soma dos corpos completos de todas as notas devolvidas + seus statements
— o que um leitor faria com `cat`. **Definir o baseline contra D161**, não contra um `ask` ideal:
o default já emite 1 corpo cheio + 4 truncados a `recall.preview_chars` (280) + linhas pipe, então a
razão honesta é `counterfactual / bytes_out`, com um A/B secundário `--brief` vs default. Golden do
formato TSV e determinismo mesmo corpus+consulta. Nenhum arquivo em `.knudge/`, nenhuma chave de
config, nenhum verbo, nenhum byte de envelope alterado.

**Critério de reversão:** se a razão vier perto de 1 em corpus real, a tese econômica está errada e
deve ser reescrita na documentação — não o tool.

---

## 5. Colapso de quase-duplicatas no orçamento do `rewind` — **DIFERIR condicionado**

**O que há de bom.** A seleção auditável ("n inteiros, m colapsados, k tokens liberados") é melhor
que heurística silenciosa, e o objetivo — não gastar 4000 tokens repetindo a mesma afirmação — é
legítimo num corpus de anos. O rascunho acerta ao exigir que nenhum id citado desapareça e ao
preservar o link.

**Por que não entra agora.**

1. **Determinismo (D92/byte contract).** O rascunho propõe consultar `duplicate` (D158,
   `ask --suggest`) ou `same_as` (D207). O primeiro depende do índice semântico **quente**; com
   embeddings lazy (D80), notas recém-criadas ficam "dark" até o dreno. O mesmo `rewind` poderia
   colapsar ou não conforme o estado do worker → dois agentes, manifests
   diferentes. Isso fere D88 *na prática* (o `context_id` mudaria, o que é honesto mas caro) e R45
   #1. A rota de degradação (R33) "cai para `same_as`/âncora igual" é justamente a parte que volta a
   ser determinística — então use **só** o explícito (`same_as`/`replaces`), nunca o sugerido.
2. **`replaces` já resolve o caso mais comum.** Nota substituída tem `superseded_by` e não deveria
   estar no manifest; colapsá-la é redundância. Sobra `same_as`, que é equivalência real — e aí o
   ganho é especulativo: no contrato pipe (~15–30 tokens/hit), três duplicatas custam pouco diante de
   4000 tokens de orçamento.
3. **Sem medição, R43 revertia.** O próprio rascunho exige "tokens liberados" como delta; não há
   número. E R45 #3/#4 exigem que supersessão vença evidência e que acesso ≠ verdade — colapsar
   por similaridade é usar "acesso/aparência" como verdade.

**Condição para entrar:** prova, em corpus real, de que duplicatas `same_as` explícitas ocupam
orçamento que uma nota distinta não consegue usar; implementação **só** sobre arestas explícitas;
golden com 3 duplicatas + 1 distinta mostrando que a distinta entra. Sem isso, fica de fora.

---

## 6. Guarda de concorrência na `update` — **ADOTAR na forma mínima**

**Correção factual.** O rascunho pede "novo `ErrorKind`, exit code próprio". Isso já existe:
`ErrorKind::Conflict`, `code = "conflict"`, `exit_code = 4`, descrito no próprio código como
"lock, dedup, **revisão concorrente**". Não há custo de taxonomia nem de mapa de exit — só um flag.

**Problema real, porém menor do que o rascunho sugere.** `revision` não é CAS por decisão (D48); o
lock advisory por arquivo-alvo com stale de 30 s (D23–D25) protege a janela curta. O risco é o
retry de agente que reaplica `update` sobre corpo já mudado. Mitigações existentes: id por conteúdo
(D01) torna reescrita idempotente; o protocolo de duas fases (D26) exige `recall` antes de `write`.
Ainda assim, um guard explícito é barato e honesto para quem faz retry — e a chegada de shards por
escritor (D220) torna identidade de agente uma coisa real, ainda que **sem posse** (D136).

**Desenho.** `kd write --update <ID> --expect-body-hash <hex8>`: sob o lock existente, logo antes
da escrita, hash diferente → `Error::conflict` com esperado/atual na mensagem (exit 4, `retryable`
falso — `retryable()` só é true para `Timeout`, R31). Sem flag, comportamento idêntico. Em
`--batch`, por linha. Documentar como "protocolo de retry de agente", **não** como CAS.

**O que cai:** `--expect-revision`. Expor `revision` para uso CAS-like reintroduziria exatamente a
confusão que D48 removeu; `body_hash` já carrega a versão do conteúdo e é o contrato (D06). Um
flag, não dois. Teste: A lê, B atualiza, A falha com hash antigo e a nota de B fica intacta; proptest
de sequência de updates sem perder a última versão.

---

## 7. Piso e orçamento de tokens nos hints MCP — **REJEITAR por ora**

`HintEngine` já é pointer-only, cap 3, dedup por sessão e reativo por gatilho (D68) — desenho enxuto
e correto. O rascunho propõe `mcp.hints_floor` + `mcp.hints_budget_tokens` com default identidade e
calibração por replay. Três obstáculos:

1. **A dependência declarada dele mesmo morreu:** "só depois de §2 existir, porque é o §2 que dá o
   corpus de consultas reais". §2 foi rejeitado. Não há corpus novo.
2. **O medidor não tem folga:** a bancada está em 100 % de nDCG@5 (`reranking_ann.md` §1). Um piso
   não pode ser calibrado onde nenhuma família cai — "nenhuma família cai mais de 1 ponto" é
   tautologia em teto.
3. **Custo de superfície sem evidência:** duas chaves de config para um problema que ninguém mediu.
   R43 e "simplicidade/localidade" (AGENTS §0) pedem o contrário.

Reavaliar somente depois que famílias difíceis existirem **e** replay de `session-log` (D220) mostrar
que hints fracos realmente ocupam vagas. Até lá, default identidade é a própria decisão.

---

## 8. O que **não** transplantar, e por quê

| Mecanismo do atlas | Por que não |
|---|---|
| Embedder próprio in-process (BERT int8 + GEMM AVX2/VNNI/NEON) | D101/R16/R43: o knudge ganha qualidade e PT com provedor HTTP local (D216). Manter kernels e tokenizer por anos não paga. |
| Store SQLite como verdade | Contra a tese: nota = verdade, índice derivado. Trocaria `git diff` por `VACUUM INTO`. |
| Code index (SCIP + scanner, `symbol`) | Fora de escopo; o knudge deliberadamente não é ferramenta de código. Se um dia for, LSP/SCIP como adaptador opcional, nunca scanner próprio. |
| Roteamento de output de ferramenta (handle `h:ID` + digest) | Quem vê output de ferramenta é o harness (Claude Code, pi), não o `kd`. Duplicar criaria segundo lugar onde o output vive. |
| Lanes, `Kind:`, caps por tipo de agente, ledger de worktrees | **Justificativa atualizada:** D136 remove **posse/autoridade**; D218–D223 dão **identidade** (shards, `agent_role`) como proveniência, não como poder. Identidade ≠ autoridade: o projeto separa as duas, e é essa separação — não "existir só um agente" — que barra lanes e caps. |
| Cartão de checkpoint e injeção automática por prompt | MCP do knudge é reativo por gatilho (D68); injeção por prompt é do cliente. |
| Auto-update assinado no binário | Supply chain desnecessária: `kd self upgrade` já verifica checksum (D184/E18-T05) e `drain service --install` valida SHA-256 do script remoto; assinatura não acrescenta o suficiente. |

Observação de engenharia, não de feature: o atlas é um monólito de ~172k LOC com 15 dias de vida e
autor único. Nada aqui copia código — copia **mecanismo**, reimplementado dentro das R01–R45. O
fato de um mecanismo ser pequeno no atlas não o torna barato de possuir aqui.

---

## 9. Ordem de execução e gates revisados

1. **§3 (deny-list)** — independente, menor custo, maior risco evitado. Entra primeiro.
2. **§1 (refs opt-in)** — fecha o furo de transporte de frescor do handoff; só toca `handoff` e
   stderr; `--with-refs` mantém o default byte-idêntico.
3. **§6 (guarda mínima)** — um flag, `Conflict` existente, sem novo `ErrorKind`.
4. **§4 (custo na bancada)** — fora do runtime; dá número para defender ou corrigir a tese.
5. **§5 e §7** — só com as evidências descritas em cada seção; hoje **fora** do escopo.
6. **§2** — **fora**; a necessidade real vai para `bench/` (famílias difíceis, `reranking_ann.md` §1).

Regras de fecho, no espírito de R43 e R45:

- Cada item que entra traz golden/property de **não-regressão** do contrato que encosta; os itens que
  tocam leitura (`rewind`, `ask`) passam pelo checklist R45 inteiro — default identidade, leitura
  não escreve, supersessão vence, fonte nunca deletada, sweep com relatório.
- Toda mudança de bytes/saída entra em `17_matriz_aceitacao.md` + `prime`/`MODULE.md`/`CHANGELOG`;
  todo estado novo, em `ARCHITECTURE.md`; toda decisão nova, em `03_decisoes-fechadas.md`.
- Se o número não move (custo, tokens liberados, hints emitidos × nDCG), **reverte** e registra
  `Dxx` negativa. O valor desta revisão é a lista do que **não** vale — agora com os motivos
  amarrados a decisões que o projeto já tomou.
