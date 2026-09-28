# 03 · `kd init` — fundar a memória do projeto

## Para que serve

Cria a memória do knudge **dentro do projeto atual**. É o primeiro comando que você roda, uma única
vez por projeto, na raiz do repositório git. Ele prepara a estrutura, **ensina o agente a usar o
`kd`** (via `AGENTS.md` e a skill `.agents/skill/kd/SKILL.md`) e deixa o git configurado para
versionar a memória sem versionar o derivado.

## Quando usar

- **Use** ao adotar o knudge em um projeto (uma vez).
- **Use `--force`** depois de mudar a configuração global e querer recopiá-la para o projeto.
- **Não use** para atualizar notas nem migrar corpus — para isso existe
  [`kd doctor --fix`](10_doctor.md).

## Sintaxe

```
kd init [--force] [--no-prompt] [--json]
```

## Exemplos

### 1. Fundar e seguir

```bash
cd meu-projeto
kd init
```

Saída:

```
projeto meu-projeto fundado em /home/voce/meu-projeto/.knudge
```

### 2. Sem o prompt inicial (scripts/CI)

```bash
kd init --no-prompt
```

### 3. Ler o caminho da memória num script

```bash
kd --json init --no-prompt | jq -r '.data.root'
```

### 4. Recopiar a configuração global

```bash
kd init --force
```

Útil depois de ajustar o template global. **As notas não são tocadas.**

## Flags

| Flag | O que faz |
|---|---|
| `--force` | Sobrescreve a configuração existente (não mexe nas notas) |
| `--no-prompt` | Não imprime o prompt inicial de fundação |
| `--json` | Devolve o envelope de máquina |

## Resultado esperado

Cria a estrutura:

```
.knudge/
  config.toml        # configuração do projeto
  notas/<tipo>/      # uma pasta por tipo (fact, decision, …)
  eventos/           # histórico append-only
  .idx/ cache/       # derivado (fora do git)
```

`templates.toml` (seções de plano por tipo) e `validators.toml` (catálogo de verificações) são
criados sob demanda, quando você usa os recursos correspondentes.

E, quando há git:

- **`.git/info/exclude`** — ignora o derivado (`.idx/`, `cache/`, `.locks/`);
- **`.gitattributes`** — bloco gerenciado para que os logs façam merge sem conflito.

Além disso, sempre cria/atualiza:

- **`AGENTS.md`** — bloco `<!-- knudge:start --> … <!-- knudge:end -->` ensinando o agente a usar o
  `kd` (idempotente: rodar de novo não duplica);
- **`.agents/skill/kd/SKILL.md`** — a **skill do agente** (frontmatter + guia de ação), com um
  marcador de versão. É **governada**: o `kd init` nunca sobrescreve um arquivo que você editou à
  mão (sem o marcador); só cria se faltar ou atualiza se a versão do knudge for mais nova.

No `--json`: `{project, root, knowledge_dir, in_repo, config_written, exclude_changed,
attributes_changed, agents_changed, skill_changed}`.

## Quando não usar

- Fora de um repositório git o `init` funciona, mas as integrações de git viram no-op.
- Não rode `kd init` para "consertar" um corpus estranho: use [`kd doctor`](10_doctor.md).

## Veja também

➡️ [Quickstart](00_quickstart.md) · [`kd prime`](04_prime.md) · [`kd ask`](05_ask.md)
