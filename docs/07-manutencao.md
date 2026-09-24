# 07 — Manutenção e handoff

## Handoff: `kd rewind`

Reconstrói o contexto no início de uma sessão, dentro de um **orçamento de tokens**.

```bash
kd rewind --budget 2000            # manifest + tarefas prontas (next:)
kd rewind --files src/gateway.rs   # só o working set de arquivos
kd rewind --scope plan_01abc       # limita a um escopo/domínio
kd rewind --since 2026-01-01 --until 2026-06-01
kd rewind --resume <context_id>    # retoma um contexto 1:1
```

O orçamento é `ceil(len/4)` tokens (default 4000). A saída traz `next:` (tarefas prontas por
impacto) e `fresh:`, além do `context_id` retomável. **Não** use como busca dirigida — para isso
há o [`kd ask`](03-ask.md).

## Saúde: `kd maintenance doctor`

```bash
kd maintenance doctor            # relatório de saúde
kd maintenance doctor --fix      # repara o reversível
kd maintenance doctor --audit    # integridade + arestas sugeridas + âncoras quebradas
```

O `--audit` é o que aponta **âncoras quebradas** (arquivo removido) e integridade do grafo.

## Propostas (read-only): `learn`, `compact`, `prune`

Estes comandos **só propõem** — nada muda sem o seu aceite (D47):

```bash
kd maintenance learn       # o que deveria virar nota? links? merges?
kd maintenance compact     # propõe merge/supersede de quase-duplicatas
kd maintenance prune       # propõe forget por shelf-life/decay
```

Aplique as propostas com [`kd write`](04-write.md) (`--link`, `--update`) ou `kd forget`.

## Índice: `kd maintenance index`

```bash
kd maintenance index --status    # fila de embeddings (pending)
kd maintenance index --drain     # drena um lote agora
```

## Avaliação: `kd maintenance eval`

```bash
kd maintenance eval          # Recall@k, nDCG@k, MRR
kd maintenance eval --ab     # compara canais (A/B)
```

## Esquecer: `kd forget`

```bash
kd forget <ID>            # soft-delete (status forgotten)
kd forget <ID> --restore  # restaura
kd forget <ID> --purge    # remove fisicamente (após a retenção)
```

`forgotten`/`superseded` ficam fora do `ask` por padrão. O `--purge` também remove as arestas de
entrada das demais notas (sem pontas soltas no grafo).

## Configuração: `kd config`

```bash
kd config list
kd config get embeddings.endpoint
kd config set recall.semantic false
kd config set mcp.hints_cap 3 --global   # config global (template)
kd config unset <chave>
```

Dois níveis: **global** (`~/.config/local/knudge/config.toml`, template) e **projeto**
(`.knudge/config.toml`, efetivo). `strict` é config, não flag (D94).

## Versionar: `kd sync`

```bash
kd sync --message "notas: decisão do rate limit"
```

Commita `notas/` + `eventos/`. O derivado (`.idx/`, `cache/`, `contexts/`) fica fora do git.

## Mapa: `kd knowledge map`

```bash
kd knowledge map --axis anchor            # agrupa por âncora (arquivo)
kd knowledge map --axis type --members
kd knowledge map --axis scope --semantic
kd knowledge map --write                    # materializa notas/MAP.md + hubs (versionado)
```

Clusters estruturais (fase 1) e, com `--semantic`, semântico complete-link (fase 2).

## Próximo passo

➡️ [MCP](08-mcp.md)
