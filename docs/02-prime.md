# `kd prime` — o protocolo

## O que faz

Imprime o **protocolo de uso** do knudge: o "help da IA". É um texto **estático**, byte-idêntico
para uma mesma versão do binário, pensado para ser colado no início de uma sessão de agente. Diz o
que usar, quando usar e — principalmente — **quando não usar**.

`kd` sem argumentos é exatamente `kd prime`.

## Em 30 segundos

```bash
kd prime            # protocolo completo (o "help da IA")
kd prime --long     # + gramática TOON e schema completo
kd prime --json     # envelope de máquina
```

## Uso

```
kd prime [--long] [--json]
```

## Exemplos

### Nível 1 — carregar o protocolo

```bash
kd prime
```

Saída (resumida):

```
knudge (kd) — memória por projeto, otimizada para LLM.
TIPOS: fact, decision, question, task, def, error, snippet, link, meta, risk
...
CICLO: kd ask (buscar) → kd write (gravar) → kd task (executar) → kd sync (commit).
...
GUIA RÁPIDO (o que usar, quando e quando NÃO usar):
  kd ask <QUERY>     RECUPERAR antes de agir; ...
  kd write <...>     GRAVAR fato/decisão/erro/risco/pergunta. ...
  ...
```

### Nível 2 — schema completo

```bash
kd prime --long
```

Inclui a gramática TOON, a ordem canônica das 25 chaves e o significado de cada campo. Use quando
o agente precisa **escrever** notas corretamente ou depurar um frontmatter.

### Nível 3 — consumo programático

```bash
kd --json prime | jq -r '.data.protocol' | head -40
```

O protocolo vem em `data.protocol`; a versão em `data.version`.

## Resultados

| Canal | Conteúdo |
|---|---|
| `stdout` | O protocolo (texto) ou o envelope JSON |
| `stderr` | Nada (não há log) |

O texto é **estável por versão**: mudanças de prosa quebram o golden `prime.txt` de propósito.
Para automação, prefira `--json`.

## Quando (não) usar

- **Use** no começo de uma sessão de agente, ou quando quiser lembrar a superfície.
- **Não use** como busca (`kd ask`) nem como histórico (`kd rewind`). O `prime` é estático; não
  conhece o seu corpus.

## Próximo passo

➡️ [`kd init`](03-init.md) · [`kd ask`](04-ask.md)
