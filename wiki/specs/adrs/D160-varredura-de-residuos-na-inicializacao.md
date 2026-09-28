# D160 — Varredura de resíduos na inicialização

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Varredura de resíduos na inicialização.** `store::sweep_residues` (R10) passa a rodar ao abrir a sessão (`Session::sweep_residues`, chamado em `run_session`) sobre `notas/`, `.idx/`, `cache/` e `eventos/`, removendo `*.tmp`/`*.stale` mais velhos que `LockPolicy::default().stale_ms` (30 s) com `warn`. **Nunca** varre `*.lock` nem `.locks/`: o reclaim de lock é atômico e fica no `lock.rs`/`doctor --fix` — varrer lock aqui poderia roubar um lock vivo. Best-effort (R33): falha vira `warnings[]`, nunca derruba o comando; cada remoção é reportada em stderr pelo `Logger`. `prime`/`self version`/`completions` não abrem sessão e seguem byte-a-byte (D57). Fecha E03-T08/R10.

## Impacto

- Fecha E03-T08/R10.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
