# wiki — documentação do knudge

Documentação organizada em três camadas:

| Caminho | Conteúdo |
|---|---|
| [`CHANGELOG.md`](CHANGELOG.md) | histórico de versões (formato *Keep a Changelog*). |
| [`specs/`](specs/README.md) | **documentação técnica**: arquitetura, contrato de bytes, bordas e um documento por subsistema. |
| [`specs/adrs/`](specs/adrs/README.md) | **decisões** (`D01`–`D209`), uma por arquivo, com contexto e impacto. |
| [`usage/`](usage/README.md) | **guias de uso** práticos: um documento por comando, com exemplos e resultados esperados. |
| [`integration/`](integration/README.md) | **integração do `knudge-core`** em outro projeto Rust: fachada, cada subsistema e otimização. |

## Por onde começar

- **Entender o sistema:** [`specs/ARCHITECTURE.md`](specs/ARCHITECTURE.md) → [`specs/README.md`](specs/README.md).
- **Usar o `kd`:** [`usage/00_quickstart.md`](usage/00_quickstart.md) → [`usage/README.md`](usage/README.md).
- **Embutir o núcleo em Rust:** [`integration/README.md`](integration/README.md).
- **Por que uma decisão foi tomada:** [`specs/adrs/README.md`](specs/adrs/README.md).
- **Contrato de bytes:** [`specs/TOON.md`](specs/TOON.md).
- **Bordas conhecidas:** [`specs/DIVERGENCES.md`](specs/DIVERGENCES.md).

## Convenções

- **Idioma:** português; identificadores de código em inglês.
- **Fonte da verdade:** `plan/03_decisoes-fechadas.md` (decisões) e o código. Os documentos
  técnicos derivam dela; quando divergirem, o código e as decisões vencem.
- **Rastreio:** cada documento técnico cita os `Dxx` que o regem e os arquivos onde o
  comportamento vive.
