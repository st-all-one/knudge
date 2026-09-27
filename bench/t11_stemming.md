# Stemming PT conservador — medição E16/T11

> **Recorte da bancada de qualidade** (`bench/src/quality.rs`, família `morfologia` adicionada por
> T11). Mede o **baseline** (D172/D173, sem stemmer) contra o **texto stemmado** pelo mesmo
> pré-processamento que virou produção (D206). **Observação, não gate** (E13-T09/R43).
>
> Comando (pré-adoção): `cargo run --release --manifest-path bench/Cargo.toml -- stemming`.

## Resultado

| família | n | R@1 base | R@1 stem | nDCG@5 base | nDCG@5 stem |
|---|---:|---:|---:|---:|---:|
| com-acento | 12 | 12.5% | 12.5% | 100.0% | 100.0% |
| morfologia | 12 | 0.0% | 12.5% | 0.0% | 100.0% |
| sem-acento | 12 | 12.5% | 12.5% | 100.0% | 100.0% |
| sinonimo | 12 | 12.5% | 12.5% | 100.0% | 100.0% |
| working-set | 12 | 100.0% | 100.0% | 100.0% | 100.0% |
| **geral** | 60 | 27.5% | 30.0% | 80.0% | **100.0%** |

Ganho de **nDCG@5 (geral): +25,0 %**; de R@1 (geral): +9,1 %. Nenhuma família regride.

## Decisão

**Adotado (D206).** O ganho de +25 % ≥ o limiar de 20 % do épico. A família `morfologia`
(consulta plural × nota singular, ex.: `configurações` × `configuração`) sai de **0 % para 100 %**
de nDCG@5 — a lacuna que D172/D173 não cobriam. O stemmer é conservador (radical mínimo de 4
bytes) e o plural é normalizado antes do corte derivacional, para `decoder`/`decoders` convergirem.

Muda só o derivado `.idx/` (`INDEX_FORMAT` → `retrieval-v4`); os bytes de `notas/` são idênticos.
