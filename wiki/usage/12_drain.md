# 12 · `kd drain` — fila e worker de embeddings

## Para que serve

Cuida da **fila de embeddings** (a busca semântica opcional) e instala/gerencia o **worker** que
mantém o servidor de embeddings no ar e drena a fila periodicamente.

- `kd drain --status` mostra o estado da fila.
- `kd drain --digest` indexa agora.
- `kd drain service ...` instala e administra o worker em segundo plano.

> Se você não usa busca semântica, pode ignorar este comando. Veja o
> [guia de embeddings](18_embeddings.md) para instalar do zero.

## Quando usar

- **Use `--status`** quando quiser saber o que falta indexar.
- **Use `--digest`** para forçar a indexação agora.
- **Use `service --install`** para deixar a indexação automática e o servidor persistentes.
- **Não use** para buscar: a busca é [`kd ask`](05_ask.md).

## Sintaxe

```
kd drain --status
kd drain --digest [--force]
kd drain service [--install | --status | --subscribe | --unsubscribe | --reconcile | --uninstall]
```

## Exemplos

### Ver e drenar a fila

```bash
# 1. estado da fila
kd drain --status

# 2. indexar agora
kd drain --digest

# 3. reconstruir o índice do zero (último recurso)
kd drain --digest --force
```

`--force` apaga o índice derivado e reindexa tudo. Use só quando o índice estiver corrompido — a
reindexação pode ser demorada.

### Instalar e gerenciar o worker

```bash
# 1. instalar agendador + servidor + cadastrar este projeto
kd drain service --install

# 2. ver a saúde
kd drain service --status

# 3. só mostrar o plano, sem baixar nem executar
kd drain service --install --dry-run
```

### Multi-projeto

```bash
# 1. cadastrar outro projeto (rode dentro dele)
kd drain service --subscribe

# 2. descadastrar este projeto (mantém o sistema)
kd drain service --unsubscribe

# 3. alinhar endpoint/modelo do projeto ao worker
kd drain service --reconcile
```

### Desinstalar

```bash
# 1. remove agendador + servidor (preserva o modelo GGUF)
kd drain service --uninstall

# 2. move o GGUF para o lixo recuperável
kd drain service --uninstall --remove-model

# 3. manter o modelo (padrão explícito)
kd drain service --uninstall --keep-model
```

## Flags

### `drain`

| Flag | Efeito |
|---|---|
| `--status` | Mostra o estado da fila, sem drenar |
| `--digest` | Digere a fila em lotes até esvaziar/estagnar |
| `--force` | Com `--digest`: apaga o índice derivado e refaz tudo |

### `drain service`

| Flag | Efeito |
|---|---|
| `--install` | Instala agendador + servidor e cadastra o projeto |
| `--status` | Mostra a saúde (default) |
| `--subscribe` / `--unsubscribe` | Cadastra/descadastra o projeto atual |
| `--reconcile` | Alinha endpoint/modelo do projeto ao worker |
| `--uninstall` | Remove o sistema (preserva o GGUF) |
| `--keep-model` / `--remove-model` | Com `--uninstall`: preserva ou move o GGUF |
| `--dry-run` | Só mostra o plano |
| `--every <DUR>` | Período do drain (`30m`, `1h`, `1d`; default `1h`) |
| `--port <N>` | Porta do servidor de embeddings (default `8889`) |
| `--model <PATH>` | Caminho do GGUF |
| `--no-deps` | Não baixa dependências no `--install` |
| `--script` / `--url` / `--sha256` | Usar um script próprio (offline/verificação) |

## Resultado esperado

- **`drain --status`** — provedor, modo, dimensões, pendentes e desatualizados.
- **`drain --digest`** — quantas notas foram indexadas.
- **`drain service --status`** — saúde do agendador, do servidor e a fila por projeto.
- **`drain service --install`** — o que foi instalado e onde.
- O worker recusa instalar sem `systemd`/`launchd` e imprime a linha de `cron` equivalente.
- No Windows nativo, o script embutido é Unix: use `--script worker.ps1` ou o caminho manual do
  [guia de embeddings](18_embeddings.md).

## Quando não usar

- Para consultar o conhecimento → [`kd ask`](05_ask.md).
- Para ver a saúde do corpus → [`kd doctor`](10_doctor.md).

## Veja também

➡️ [Embeddings](18_embeddings.md) · [`kd doctor`](10_doctor.md) · [`kd config`](13_config.md)
