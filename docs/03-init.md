# `kd init` — fundar a memória

## O que faz

Funda o `.knudge/` no projeto atual: cria a estrutura de diretórios, copia o template de config
global, prepara o `.git/info/exclude` e o bloco `.gitattributes`, e escreve um **bloco gerenciado**
no `AGENTS.md` do projeto ensinando o agente a usar o `kd`. Por fim, emite o **prompt inicial** de
fundação (a menos que `--no-prompt`).

Rode **uma vez por projeto**, na raiz do repositório git.

## Em 30 segundos

```bash
kd init
```

Isso é tudo. Se você só quer a estrutura sem o prompt:

```bash
kd init --no-prompt
```

## Uso

```
kd init [--force] [--no-prompt] [--json]
```

| Flag | Efeito |
|---|---|
| `--force` | Sobrescreve a configuração existente (cuidado: não mexe nas notas) |
| `--no-prompt` | Não emite o prompt inicial de fundação |
| `--json` | Envelope de máquina |

## Exemplos

### Nível 1 — fundar e seguir

```bash
cd meu-projeto
kd init
```

Saída (texto):

```
projeto meu-projeto fundado em /home/voce/meu-projeto/.knudge
```

### Nível 2 — script/CI

```bash
kd --json init --no-prompt | jq -r '.data.root'
```

### Nível 3 — re-fundar com a config template

```bash
kd init --force          # recopia a config global para o projeto
```

Útil depois de mudar o template global. As **notas não são tocadas**.

## Resultados

Cria:

```
.knudge/
  config.toml
  notas/<tipo>/          # uma pasta por tipo (fact, decision, …)
  eventos/
  templates.toml
  validators.toml
  .idx/ .locks/ cache/
```

E atualiza, se houver git:

- `.git/info/exclude` — exclui o derivado (`.idx/`, `cache/`, `.locks/`);
- `.gitattributes` — bloco gerenciado com `merge=union` para `eventos/events*.jsonl` e
  `.knudge/emb_cache.jsonl`;
- `AGENTS.md` — bloco `<!-- knudge:start --> … <!-- knudge:end -->` (idempotente).

Envelope JSON: `{project, root}`.

## Quando (não) usar

- **Use** uma vez, no começo.
- **Não use** para atualizar notas nem para migrar corpus: para isso há
  `kd doctor --fix` (ver [Manutenção](09-maintenance.md)).
- Fora de um repositório git, o `init` funciona, mas as integrações de git ficam no-op.

## Próximo passo

➡️ [`kd ask`](04-ask.md) · [Quickstart](00-quickstart.md)
