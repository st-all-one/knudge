# Guia de uso do knudge

**knudge** (`kd`) é a memória por projeto de um agente de IA: notas Markdown versionadas ao lado do
código, que o agente consulta antes de agir e atualiza depois de aprender.

Estas páginas são para **usar** o `kd` no dia a dia. Elas cobrem a superfície da linha de comando —
cada comando, cada flag, cada ação — com foco em **o que faz, como usar, quando usar e o que
esperar**. Para a referência técnica do código, veja [`../specs/`](../specs/README.md),
[`../../plan/`](../../plan) e [`../../AGENTS.md`](../../AGENTS.md).

---

## Comece aqui

1. **[00 · Quickstart](00_quickstart.md)** — instalar, fundar, gravar, buscar e retomar. Inclui
   instalação rápida/manual do `llama.cpp` e o worker de embeddings.
2. **[01 · Filosofia](01_filosofia.md)** — o que o knudge é, por que existe e como ajuda, em
   linguagem simples.
3. **[02 · O ciclo e a CLI](02_ciclo.md)** — vocabulário e convenções válidas em todos os comandos.

---

## Comandos (um guia por comando)

| Comando | Para quê |
|---|---|
| [`kd init`](03_init.md) | Fundar a memória no projeto (uma vez) |
| [`kd prime`](04_prime.md) | O protocolo estático ("help da IA") |
| [`kd ask`](05_ask.md) | Toda a **pesquisa**: recall, get, expand, rank, tags, suggest |
| [`kd write`](06_write.md) | Toda a **escrita**: criar, atualizar, ligar, evidência, lote |
| [`kd task`](07_task.md) | Planejar e executar trabalho (épico → issue → tarefa) |
| [`kd rewind`](08_rewind.md) | Retomar o contexto entre sessões |
| [`kd map`](09_map.md) | Mapa do conhecimento (clusters e comunidades) |
| [`kd doctor`](10_doctor.md) | Saúde do corpus: diagnósticos e reparo reversível |
| [`kd maintenance`](11_maintenance.md) | Propostas de limpeza (`compact`/`learn`/`prune`) |
| [`kd drain`](12_drain.md) | Fila de embeddings e worker (`service`) |
| [`kd config`](13_config.md) | Configuração em dois níveis + `promote` |
| [`kd forget`](14_forget.md) | Esquecer/restaurar/purgar notas |
| [`kd sync`](15_sync.md) | Versionar a memória no git |
| [`kd self`](16_self.md) | Setup de cliente, completions, upgrade, versão |

Guias transversais:

| Guia | Para quê |
|---|---|
| [MCP](17_mcp.md) | Usar de dentro de um agente (Claude/Cursor/Codex/pi) |
| [Embeddings](18_embeddings.md) | Ligar a busca semântica (opcional) |
| [Troubleshooting](19_troubleshooting.md) | Problemas comuns e como resolver |

---

## O ciclo central

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

**Sempre busque antes de gravar.** O `kd ask "<rascunho>"` evita duplicata e mostra a nota que
talvez você só precise atualizar.

---

## Convenções da CLI (valem em todos os comandos)

- **`stdout` = dados** (pipe/`--json`); **`stderr` = logs**. Nunca se misturam.
- **`--json`** devolve um envelope de máquina estável.
- **`--brief`** encurta a saída (gasta menos contexto).
- **Posicional = conteúdo:** em `write`/`task new` é o **corpo**; em `ask` é a **consulta**. `-` lê
  de stdin; sem posicional, com pipe/heredoc, também lê de stdin.
- **`--params '<json>'`** envia o objeto completo de uma vez (`-` lê de stdin).
- **`kd` sozinho = `kd help`; o protocolo é `kd prime`.**

Detalhes em [02 · O ciclo e a CLI](02_ciclo.md#5-convenções-da-cli-valem-em-todo-lugar).
