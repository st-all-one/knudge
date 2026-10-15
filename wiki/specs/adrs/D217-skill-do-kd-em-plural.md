# D217 — Skill do `kd` passa para `.agents/skills/kd` (plural)

- **Status:** Aceita
- **Categoria:** Git e artefatos gerados (D60/D162)

## Contexto

D162 fez o `kd init`/`onboard` criar a skill do agente em `.agents/skill/kd/SKILL.md` (singular).
O repositório passou a usar a convenção **plural** para as demais skills do agente (ex.: a skill de
Rust em `.agents/skills/rust/`), o que criava duas convenções coexistindo (`.agents/skill/` e
`.agents/skills/`).

## Decisão

**A skill do `kd` passa a viver em `.agents/skills/kd/SKILL.md`** (plural). `crates/knudge-core/src/git/skill.rs::DIR`
muda de `.agents/skill/kd` para `.agents/skills/kd`; o `kd init` deixa de ter o caminho hardcoded e
passa a usar `git::SKILL_DIR`/`SKILL_FILE` (fonte única). O diretório antigo **não** é mantido nem
migrado (D14, sem retrocompatibilidade): a skill é regenerada no path novo pelo `init`, que já
sobrescreve o próprio arquivo via version marker (D162).

Testes e goldens (`git::tests::onboard`, `tests/golden/init.txt`) acompanham o novo path.

## Consequências

- Uma convenção só: `.agents/skills/<nome>/SKILL.md` para todas as skills do agente.
- Projetos já fundados ganham a skill no path novo na próxima execução de `kd init`; o arquivo
  antigo em `.agents/skill/kd/` não é removido automaticamente (fica órfão e pode ser apagado à mão).

Revisa D162.
