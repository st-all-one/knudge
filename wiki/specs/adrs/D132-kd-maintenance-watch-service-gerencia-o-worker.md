# D132 — kd maintenance watch-service gerencia o worker ocioso — com

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**`kd maintenance watch-service` gerencia o worker ocioso — com consentimento e sem supply-chain.** Ações exclusivas (default `--status`): `--install` (pré-flight de `systemd --user`/`kd`/`llama`/GGUF/projeto → escreve `idle.conf` + unidades, habilita o timer e cadastra o projeto atual), `--subscribe`/`--unsubscribe` (cadastram/descadastram **um** projeto, multi-projeto; **não** desinstalam o sistema), `--status` (saúde: timer, servidor, fila por projeto) e `--uninstall` (remove o sistema). Ações que mutam perguntam no stderr (`s/N`; stdin não-TTY cancela; `--yes` pula). O worker (`knudge-idle.sh`) é **embutido no binário** (`include_str!`) e materializado no staging de cache — **sem download por padrão**; `--script` (local) e `--url`/`KNUDGE_SCRIPT_URL` (HTTPS via `curl`) sobrescrevem. O GGUF mora ao lado do `config.toml` global (`${XDG_CONFIG_HOME:-~/.config}/local/knudge/`). Multi-projeto = lista `PROJECT=` no `idle.conf` + **um** timer; `run` sem args drena todos; ao esvaziar, o timer é parado, mas as unidades permanecem.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
