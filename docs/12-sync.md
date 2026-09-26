# `kd sync` — versionar o corpus

## O que faz

Commita a **verdade** do knudge no git: `notas/` + `eventos/` (e o cache vetorial, quando
versionado). O **derivado** (`.idx/`, `cache/`, `.locks/`, `contexts/`) fica **fora** do git —
ele se reconstrói.

## Em 30 segundos

```bash
kd sync --message "notas: decisão do rate limit"
```

## Uso

```
kd sync [--message <TXT>]
```

| Flag | Efeito |
|---|---|
| `--message <TXT>` | Mensagem de commit |

## Exemplos

### Nível 1 — commit simples

```bash
kd sync --message "notas: decisão do rate limit"
```

### Nível 2 — o que entra e o que fica fora

| Versionado | Derivado (fora) |
|---|---|
| `.knudge/notas/**` | `.knudge/.idx/` |
| `.knudge/eventos/events*.jsonl` | `.knudge/cache/` |
| `.knudge/config.toml`, `templates.toml`, `validators.toml` | `.knudge/.locks/` |
| `.knudge/emb_cache.jsonl` (se `version_cache`) | `.idx/contexts/` |

O `kd init` escreve as regras em `.git/info/exclude` e o bloco de `.gitattributes`:

```
/.knudge/eventos/events*.jsonl text eol=lf merge=union
/.knudge/emb_cache.jsonl text eol=lf merge=union
```

### Nível 3 — multi-dev

Clone com o **mesmo modelo** de embeddings: o índice se reconstrói de notas + cache, **sem
inferência**:

```bash
git pull
kd drain --digest    # reindexa do cache, zero inferência
```

Conflito de nota (mesma `statement`, corpos divergentes): o git deixa marcadores; o TOON não
parseia, a leitura tolerante **pula** a nota e o `doctor` a reporta. O agente resolve/supersede —
o knudge **nunca** auto-mergeia um derivado. Ver [Conceitos §9](01-conceitos.md#9-multi-dev-sincronização).

## Resultados

- Texto: resumo do commit (arquivos/estado).
- `--json`: `{committed, files[], message}`.
- Fora de um repositório git, o `sync` degrada com `warnings[]` (não é fatal).

## Quando (não) usar

- **Use** ao fim de uma sessão de escrita, ou quando quiser um ponto de retorno.
- **Não use** para sincronizar o índice — ele é derivado; cada clone reconstrói.
- Para ver o que está pendente antes de commitar, use `git status` e
  [`kd doctor`](09-maintenance.md).

## Próximo passo

➡️ [`kd self`](13-self.md) · [MCP](14-mcp.md)
