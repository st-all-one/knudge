# E07 — Escrita e protocolo

> **MVP.** O `write` é o ponto onde quase toda duplicata nasce. Aqui entram o write
> **idempotente por conteúdo**, o **protocolo em duas fases** (dedup), `update`/`link`/
> `forget`/`restore`, e a separação entre dedup **lexical** (imediato) e **semântico**
> (eventual).
>
> **Decisões:** D01, D05, D16, D17, D18, D21, D26, D42, D47, D48, D49, D52, D80, D83.
> **Políticas:** R31, R33 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Escrever **sem medo de duplicar nem de travar**: o write estrito na forma, tolerante na
operação, idempotente sob retry e com dedup configurável.

## Pré-requisitos

E03, E05, E06.

## Tarefas

### E07-T01 ☐ `write` idempotente por conteúdo
- **Objetivo:** ID endereçado por conteúdo (E02-T06): reescrever o mesmo conteúdo devolve o
  mesmo id (sem duplicar); retry do LLM é seguro.
- **Entregáveis:** `write()` com checagem de existência por id.
- **Decisões:** D01.
- **Aceite:** N writes idênticos → 1 nota; segundo write devolve o id e registra evento.

### E07-T02 ☐ Protocolo de escrita em duas fases
- **Objetivo:** `recall` obrigatório antes do `write`; **< 0.75 cria**, **0.75–0.92 merge**,
  **≥ 0.92 rejeita**; limiares são config; com embeddings ausentes o score é **lexical** e o
  dedup semântico é **eventual**.
- **Entregáveis:** máquina de decisão; estados; mensagens.
- **Decisões:** D26, D80.
- **Aceite:** cada faixa coberta por teste; limiar alterável por config sem tocar código.

### E07-T03 ☐ `update` versionado e supersede caminhável
- **Objetivo:** `update(id, patch)` incrementa `revision`; mudança de `type` cria **nova nota
  + `replaces`**; cadeia de supersessão caminhável.
- **Entregáveis:** `update`; `replaces`; `get(id, history=true)`.
- **Decisões:** D21, D48.
- **Aceite:** histórico recuperável; supersede mantém a linhagem.

### E07-T04 ☐ `link`, `forget`, `restore`
- **Objetivo:** `link(from, kind, to)`; `forget`/`restore` via `update(status=...)` — **soft**,
  nunca apaga; `status` unifica o ciclo de vida.
- **Entregáveis:** tool `link`; transições de `status`.
- **Decisões:** D49 (arestas), D52 (status).
- **Aceite:** `forget` não remove arquivo; `restore` volta ao ativo; transições inválidas
  rejeitadas.

### E07-T05 ☐ Escrita estrita e omissão de opcionais
- **Objetivo:** rejeitar chave desconhecida e `type` desconhecido; opcionais **omitidos**,
  nunca `null`/vazio; mensagens de erro congeladas por teste.
- **Entregáveis:** validadores de write; catálogo de mensagens (compartilha E12-T02).
- **Decisões:** D05, D16, D17.
- **Aceite:** goldens de rejeição (`write` falha) e de omissão (bytes sem chave).

### E07-T06 ☐ Dedup semântico eventual
- **Objetivo:** quando houver embeddings, a reconciliação **propõe** merge/supersede de
  quase-duplicados — **nunca funde em silêncio**.
- **Entregáveis:** hook de reconciliação (roda com E11); modo `propose` (default).
- **Decisões:** D42, D47, D83.
- **Aceite:** quase-duplicados viram proposta revisável por `compact`; nada é fundido sem
  aprovação.

## Definition of Done

- [ ] `write` idempotente e com as três faixas de dedup testadas.
- [ ] `update`/`forget`/`restore`/`link` com aceite.
- [ ] Escrita estrita na forma, tolerante na operação.

## Não-objetivos

- Orquestração de `compact` (E08/E10).
- Modelo de embeddings (E11).
