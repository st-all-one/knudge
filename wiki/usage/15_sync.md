# 15 · `kd sync` — versionar o corpus

## Para que serve

Commita a **verdade** do knudge no git: as notas e o log de eventos. O índice derivado fica **fora**
do git — ele se reconstrói em cada clone.

É o comando que transforma a memória local em memória **do projeto** (compartilhada e preservada).

## Quando usar

- **Use** ao fim de uma sessão de escrita, ou quando quiser um ponto de retorno.
- **Use** antes de trocar de máquina ou de entrar outro dev.
- **Não use** para sincronizar o índice: ele é derivado; cada clone o reconstrói.

## Sintaxe

```
kd sync [--message <TXT>]
```

## Exemplos

### 1. Commit simples

```bash
kd sync --message "notas: decisão do rate limit"
```

### 2. Ver o que está pendente antes

```bash
# 1. o estado do repositório
git status

# 2. a saúde antes de commitar
kd doctor

# 3. commitar
kd sync --message "notas: revisão de hoje"
```

### 3. Retomar num clone novo

```bash
# 1. trazer as notas
git pull

# 2. reconstruir o índice
kd drain --digest

# 3. conferir
kd rewind --budget 2000
```

Com o cache de vetores versionado, a reindexação acontece **sem chamar o modelo**.

### 4. Commit programático

```bash
# 1. JSON com o que foi commitado
kd --json sync --message "notas: sessão" | jq '.data.files'

# 2. fora de um repositório git, degrada com aviso (não é erro)
kd sync --message "notas: rascunho"
```

## Flags

| Flag | Efeito |
|---|---|
| `--message <TXT>` | Mensagem de commit |

## Resultado esperado

| Versionado | Derivado (fora do git) |
|---|---|
| `.knudge/notas/**` | `.knudge/.idx/` |
| `.knudge/eventos/events*.jsonl` | `.knudge/cache/` |
| `.knudge/config.toml`, `templates.toml`, `validators.toml` | `.knudge/.locks/` |
| `.knudge/emb_cache.jsonl` (se versionado) | `.idx/contexts/` |

- **`--json`:** `{committed, files[], message}`.
- Fora de um repositório git, o `sync` degrada com aviso (não é fatal).

## Multi-dev

Clone com o **mesmo modelo** de embeddings: o índice se reconstrói de notas + cache, sem
reinferência. Se dois devs editarem a mesma nota, o git deixa marcadores; o `doctor` aponta e o
agente resolve — o knudge **nunca** mescla um derivado sozinho.

## Quando não usar

- Para buscar → [`kd ask`](05_ask.md).
- Para diagnosticar antes de commitar → [`kd doctor`](10_doctor.md).

## Veja também

➡️ [`kd self`](16_self.md) · [MCP](17_mcp.md) · [Embeddings](18_embeddings.md)
