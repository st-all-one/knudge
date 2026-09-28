# 04 · `kd prime` — o protocolo

## Para que serve

Imprime o **protocolo de uso** do knudge: o "help da IA". É um texto **estático** (o mesmo para uma
mesma versão do binário), feito para ser colado no início de uma sessão de agente. Ele diz o que
usar, quando usar e — o mais importante — **quando não usar**.

`kd` sozinho é igual a `kd help`; o protocolo é `kd prime`.

## Quando usar

- **Use** no começo de cada sessão de agente (ou quando quiser lembrar a superfície).
- **Use `--long`** quando o agente precisa **escrever** notas corretamente.
- **Não use** como busca ([`kd ask`](05_ask.md)) nem como histórico
  ([`kd rewind`](08_rewind.md)): o `prime` é estático e não conhece o seu corpus.

## Sintaxe

```
kd prime [--long] [--json]
```

## Exemplos

### 1. Carregar o protocolo

```bash
kd prime
```

Saída (resumida):

```
knudge (kd) — memória por projeto, otimizada para LLM.
CICLO: kd ask (buscar) → kd write (gravar) → kd task (executar) → kd sync (commit).
GUIA RÁPIDO (o que usar, quando e quando NÃO usar):
  kd ask <QUERY>   RECUPERAR antes de agir; ...
  kd write <...>   GRAVAR fato/decisão/erro/risco/pergunta. ...
```

### 2. Schema completo para escrita

```bash
kd prime --long
```

Inclui a gramática do formato, a ordem canônica das chaves e o significado de cada campo.

### 3. Consumo por um script/agente

```bash
kd --json prime | jq -r '.data.protocol' | head -40
```

O protocolo vem em `data.protocol`; a versão, em `data.version`.

## Flags

| Flag | O que faz |
|---|---|
| `--long` | Inclui o schema completo e a gramática |
| `--json` | Devolve o envelope de máquina |

## Resultado esperado

- **`stdout`:** o protocolo em texto (ou o envelope JSON).
- **`stderr`:** nada — o `prime` não emite log.
- O texto é **estável por versão**: automações devem preferir `--json`.

## Quando não usar

- Não substitui a busca nem o histórico. Ele descreve **como usar** o knudge, não **o que** você já
  sabe.

## Veja também

➡️ [`kd init`](03_init.md) · [`kd ask`](05_ask.md) · [MCP](17_mcp.md)
