# Riscos da memória durável × knudge (análise crítica)

> **Status:** análise (não é decisão). Origem: avaliação externa dos riscos de ferramentas de
> memória durável para agentes de IA — **deriva do curador** (*curator drift*), **envenenamento
> de memória** (*persistent memory poisoning*), **drift de sumarização**, **diluição atencional**
> e **limites de plataforma** — e das mitigações que a avaliação sugeriu. Confrontada com o
> estado real do `knudge` (v0.5.0; E16–E19).
>
> **Caveat do mantenedor (vinculante):** a antiga `confidence` **declarada** foi **removida**
> (D142) por ser burlada — era marcada arbitrariamente (default 0.7) e nada a avaliava; o merge
> só mantinha o maior (no-op). Confiança só volta **como mecanismo derivado**, nunca como campo.
>
> **Regras invioláveis (AGENTS.md):** `make check` verde; `src/` ≤ 300 linhas; sem
> `unwrap/expect/panic/unsafe`; núcleo puro via portas; "propor, nunca agir em silêncio" (D47);
> índice derivado e reconstruível (D15/D27/D84); determinismo (D92); sem servidor/DB/LLM
> obrigatório (R16/R43).
>
> **Documentos irmãos:** [`../implementation/20_qualidade_busca_depreciacao.md`](../implementation/20_qualidade_busca_depreciacao.md)
> (E16), [`../implementation/23_modelo_conhecimento_rico.md`](../implementation/23_modelo_conhecimento_rico.md)
> (E19), [`melhorias_ai_memory.md`](melhorias_ai_memory.md) (D154–D159),
> [`../03_decisoes-fechadas.md`](../03_decisoes-fechadas.md) (D87/D142/D158/D176/D177),
> [`revisao_integrada.md`](revisao_integrada.md).

---

## 0. Veredito rápido

| Proposta do brainstorm | Situação no knudge | Veredito |
|---|---|---|
| `confidence` (0–1) **e** `source` no frontmatter TOON | campo `confidence` **removido** (D142); `actor`/`mode` removidos (D136); confiança **derivada** existe (D87) | ❌ **como campo**; ✅ **como mecanismo** (E19/T01 + E19/T09) |
| `kd maintenance audit --contradictions` | `contradicts` é aresta (`graph/extract.rs`); sugestão semântica em **D158**; efeito no ranking em **D177/E16-T07**; TMS em **E19/T10** | ◐ **coberto a jusante**; falta um **check read-only** do `doctor` (candidato) |
| `kd rewind --digest` (pointer summary) | `rewind` já reporta `dropped` (contagem), nunca o conteúdo | ✅ **válido e aditivo** (baixa prioridade; candidato) |
| `kd ask --explain-dropped` | **D152** removeu a observabilidade de busca de propósito | ❌ **conflita com D152** (só com demanda real) |
| `kd config set --profile codex` | `config set` já existe; limites do host são **externos e voláteis** | ❌ **complexidade desnecessária** (documentar, não codificar) |
| Mitigações já presentes (propor, `doctor`, retention, notes=truth, hints=pointer, dedup, budget, filtros) | **implementadas** | ✅ **confirmadas** |

---

## 1. Mitigações já presentes (confirmadas)

A avaliação pede "barreiras, auditorias e pontos de decisão humana". O `knudge` já os tem; a
tabela abaixo ancora cada um no código/decisão.

| Risco | Mitigação no `knudge` | Onde |
|---|---|---|
| Deriva do curador / reescrita silenciosa | `compact`/`learn`/`prune` **só propõem**; aplicação é `kd write --update`/`kd forget` | D47, D112 |
| Correção automática perigosa | `doctor --fix` só repara o **reversível** (layout, chave fora do schema, índice) e **nunca apaga nota** | E09, D163 |
| Acúmulo de obsoletos | `classification` (foundational/tactical/observational) + `retention.*` governam expiração | D44/D135, E19/T02 |
| Envenenamento: dano contido e auditável | a **nota é a verdade**, o índice é **derivado** e reconstruível; o veneno fica num `.md` versionado e removível | D15/D27/D84 |
| Envenenamento: injeção automática | MCP devolve **ponteiros** (`id + statement + score`); o corpo só entra sob pedido (`ask --id --full-content`) | E06/D47 |
| Envenenamento: diluição por quase-duplicata | dedup em duas fases (`<0.75` cria, `0.75–0.92` merge, `≥0.92` rejeita) | D26/D80 |
| Drift de sumarização | **não há sumarização**: notas são arquivos completos; `rewind` **seleciona/omite** dentro do `--budget`, não resume | D57/E08 |
| Diluição atencional | filtros duros (`--type/--status/--tag/--anchor/--scope`) antes do BM25; canal vetorial **intersectado**; `recall.default_limit`, `mcp.hints_cap`; `observation_mode` | D143/D144/D146/D151 |
| Limites de plataforma | `ask` sob demanda (barato) + `rewind --budget` explícito + hints como ponteiros | E15, D57 |

**Conclusão:** nenhum dos riscos centrais fica sem defesa; o que falta é **superfície de
auditoria** para contradições declaradas e **rastreabilidade** do que o orçamento descarta.

---

## 2. Propostas avaliadas em detalhe

### 2.1 `confidence` + `source` como campos → **rejeitado como campo; absorvido como mecanismo**

**Por que o campo é recusado.** D142 já provou que um campo `confidence` declarado é **teatro**:
nada o avaliava, o merge só mantinha o máximo e o default 0.7 era arbitrário. Reintroduzi-lo
(ou um `source: human|agent|external`) recria o mesmo problema: um selo de procedência que o
`ask` "penalizaria" sem qualquer sinal objetivo. `actor`/`mode` também foram removidos (D136: um
agente, sem posse) — `source` os reabriria pela porta dos fundos.

**O que é válido é o mecanismo**, e ele já está no plano:

- **Confiança derivada com incerteza principiada — E19/T01 (R1, D189).** Posterior
  `Beta(α+Σs, β+Σf)` sobre os `outcomes` existentes (parcial = 0.5); **média posterior** para
  `stars` e **limite inferior do intervalo de credibilidade** como confiança conservadora
  ("1 sucesso ≠ 20 sucessos"). Puro, determinístico, derivado, **sem byte novo**. Absorve
  E16/D174 (`drift` de âncoras + `feedback` dos `outcomes` negativos) — é a "avaliação real" que
  faltava ao campo antigo.
- **Proveniência com lastro — E19/T09 (R5, PROV-lite).** `entity/activity/agent` entram na onda
  de ontologia, junto de claims SPO, **com `schema_version`** e rebuild. É o único lugar em que
  "quem escreveu" vira dado — e mesmo lá como **derivado auditável**, não como selo de fé.
- **Efeito no ranking, não em flag de `ask`.** A confiança derivada já entra no `rank`
  (D87/D108) e passa a pesar a idade (E16/T06/D175) e a contradição (E16/T07/D177). "Filtrar
  nota de baixa confiança" = **rebaixar no ranking**, jamais esconder por um campo.

**Veredito:** a parte "confiança" do brainstorm **já é o E19/T01**; a parte "source" é o
**E19/T09**. Nada a adicionar — e **nada de campo novo**.

### 2.2 `kd maintenance audit --contradictions` → **coberto a jusante; falta o check do `doctor`**

O brainstorm acerta que hoje o `doctor` detecta **duplicata**, não **contradição**. Mas o
caminho já existe em três camadas:

- **Geração de candidatos:** D158 — par na banda cosseno `0.4–0.75`, sem aresta, vira sugestão
  `contradicts` em `.idx/suggestions.jsonl` (nunca vira aresta).
- **Efeito:** E16/T07 (D177) — `contradicts` rebaixa o lado perdedor no `recall`/`rank` e entra
  como `DemotionReason` no `prune` (que **só propõe**, D112).
- **Generalização:** E19/T10 (TMS/defeasible) — rastreia suposições e derruba dependentes.

**Lacuna real:** uma aresta `contradicts` **declarada** (extraída ou manual) **não é reportada**
por nenhum check de saúde. Um `CheckId::Contradictions` **read-only** no `doctor` (lista pares
`X ⊣ Y` com ambos visíveis, aponta o perdedor por confiança derivada) é barato, determinístico e
sem contrato novo — fecha o ciclo "propor → ranquear → **auditar**". Candidato a entrar como
extensão do aceite de **E16/T07** (não como verbo novo: evita violar D201/E19-T12).

### 2.3 `kd rewind --digest` (pointer summary) → **válido e aditivo**

`rewind` já conta `dropped`, mas não diz **o que** caiu. Um modo `--digest` que, ao estourar o
orçamento, lista **ponteiros** (`id + âncora/escopo`) das notas omitidas — "3 notas sobre rate
limit fora do orçamento; `kd ask --anchor src/gateway.rs`" — dá rastreabilidade **sem**
sumarização (preserva a vantagem do knudge) e é **aditivo** (flag nova; default byte-idêntico).

Custo baixo (o manifest já tem os itens e o `dropped`). Candidato de **baixa prioridade**: entra
depois de E16/T01–T09 ou no fecho, se o usuário quiser. Home sugerida: E16 (junto de T07) ou
E19/T11 (task/rewind).

### 2.4 `kd ask --explain-dropped` → **rejeitado (conflita com D152)**

D152 **removeu** a observabilidade de busca de propósito (`recall_stats` fora; busca vazia →
`[no_results]`). Os filtros de `ask` são **explícitos** (`--type/--status/...`): o operador já
sabe o que pediu. Um `--explain-dropped` reintroduz o tipo de instrumentação que D152 podou, com
custo no caminho quente e sem demanda. **Rejeitado**; reabrir só se surgir necessidade concreta
(então seria um `Dxx` que revisa D152).

### 2.5 `kd config set --profile codex` → **rejeitado (complexidade desnecessária)**

Limites de host (ex.: ~2.500 tokens no Codex) são **externos e voláteis**; codificar perfis por
nome de host engessa o `knudge` a um alvo que muda. O que serve é **documentar** valores
conservadores (`mcp.hints_cap`, `recall.default_limit`, `rewind.budget`) e deixar o usuário
setá-los com `kd config set` (já existe). Um `--profile` também adiciona superfície contra
D201/E19-T12 (≤10 verbos). **Rejeitado**; entra como nota em `docs/` se houver demanda.

---

## 3. Onde a confiança é mecanismo, não campo

| Pergunta do brainstorm | Resposta no knudge |
|---|---|
| "Um agente persiste hipótese como fato com a mesma confiança de uma conclusão" | `outcomes` são ensaios; o **Beta** (E19/T01) separa 1 sucesso de 20 e usa o **limite inferior** (conservador) |
| "Notas criadas por agentes sem confirmação humana" | **sem** campo de autor (D136); a confirmação é **derivada** (D87/D108: `outcomes`, âncoras, tarefas) e a **proveniência** só entra com lastro em E19/T09 |
| "Filtrar/penalizar baixa confiança no `ask`" | vira **rebaixamento no ranking** (D87 + E16/T06/T07), nunca flag de filtro |
| "Estender `outcomes` a notas de conhecimento" | já é o insumo do Beta; `--outcome` vale para **qualquer** nota (D103) |

---

## 4. Encaminhamento

| Item | Destino | Prioridade |
|---|---|---|
| `CheckId::Contradictions` (read-only) | extensão do aceite de **E16/T07** (D177) | média |
| `rewind --digest` (ponteiros) | candidato E16/T07 ou E19/T11 | baixa |
| Confiança Beta / proveniência | **E19/T01** (D189) e **E19/T09** (R5) — já planejados | — |
| `--explain-dropped` / `--profile` | **rejeitados** (D152 / complexidade) | — |

Nada aqui cria `Dxx` novo antes da implementação; os candidatos entram como **pontos em aberto**
de E16/E19 e viram decisão só quando priorizados.

---

## 5. Não-objetivos (recusados por premissa)

- Reintroduzir `confidence`/`source`/`actor`/`mode` como **campos** (D142/D136).
- Sumarização de contexto ou "digest" semântico que comprima nota (viola a tese; o `--digest` é
  só **ponteiro**).
- Instrumentação de busca removida por D152 (`recall_stats`, `--explain-dropped`).
- Perfis de host codificados (`--profile codex`) ou qualquer acoplamento a limites externos.
- LLM/servidor obrigatório, dep nova sem ganho medido (R16/R43).
