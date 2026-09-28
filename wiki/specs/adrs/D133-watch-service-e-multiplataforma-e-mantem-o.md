# D133 — watch-service é multiplataforma e mantém o servidor de embeddings

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**`watch-service` é multiplataforma e mantém o servidor de embeddings persistente.** O agendador é detectado em runtime: `systemd --user` (Linux) ou `launchd` (macOS, `~/Library/LaunchAgents`); sem nenhum, o `--install` recusa e imprime a linha de cron. O servidor llama.cpp vira unidade/agente próprio (`knudge-embed.service` / `local.knudge.embed.plist`, `Restart=on-failure` / `KeepAlive`), então o `--drain` manual e o auto-drain lazy sempre o encontram; o worker só drena e sobe um efêmero de fallback se o persistente estiver fora. `--uninstall` derruba agendador + servidor.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
