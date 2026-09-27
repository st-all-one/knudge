//! Pesos dos canais da fusão RRF (D123/D192).
//!
//! Um peso maior deixa o canal “vencer” o outro em RRF. Cada canal tem um `DEFAULT_*` e a config
//! (`recall.*_weight`) sobrescreve; a fusão só soma os canais presentes (D123).

/// Peso default do canal lexical na fusão (config `recall.lexical_weight`, D123).
pub const DEFAULT_LEXICAL_WEIGHT: f64 = 1.0;
/// Peso default do canal de âncoras (config `recall.anchor_weight`, D123/D179).
///
/// `2.0`: um **match exato** de âncora contra o working set (você está editando aquele arquivo) é
/// sinal mais forte que um único casamento lexical ruidoso. Medido na bancada de qualidade
/// (família `working-set`): com `1.0`/`rrf_k=60` o rank-1 da âncora **empata** com o rank-1
/// lexical e a nota ancorada não sobe de forma confiável (nDCG@5 87,7 % → 100 % com `2.0`).
pub const DEFAULT_ANCHOR_WEIGHT: f64 = 2.0;
/// Peso default do canal vetorial (config `recall.semantic_weight`, D123).
///
/// Alto de propósito: com `rrf_k=60` num corpus pequeno os ranks ficam comprimidos e o canal
/// lexical continua competitivo, então um peso modesto (1–20) deixa a fusão **pior** que o
/// neutro. Medido no corpus PT-BR da bancada, `30` já Pareto-domina o neutro (R@1, R@5, MRR e
/// nDCG@5 ≥ neutro) e `≥80` converge para o ranking vetorial puro. Ajuste por config.
pub const DEFAULT_SEMANTIC_WEIGHT: f64 = 30.0;
/// Peso default do canal de autoridade (`PageRank` personalizado — D192).
///
/// `0.0` **desliga** o canal (default): ele só compensa em corpora com arestas de autoridade
/// (`references`/`supports`/`extends`/`replaces`). Ligue com `recall.ppr_weight > 0` (ex.: `2.0`).
pub const DEFAULT_PPR_WEIGHT: f64 = 0.0;

/// Pesos dos canais na fusão RRF (D123).
///
/// O default dá ao canal vetorial o dobro do lexical, porque em PT-BR o lexical sozinho quase não
/// distingue sinônimos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FusionWeights {
    /// Canal lexical (BM25).
    pub lexical: f64,
    /// Canal de âncoras (working set).
    pub anchor: f64,
    /// Canal vetorial.
    pub semantic: f64,
    /// Canal de autoridade (`PageRank` personalizado — D192).
    pub ppr: f64,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            lexical: DEFAULT_LEXICAL_WEIGHT,
            anchor: DEFAULT_ANCHOR_WEIGHT,
            semantic: DEFAULT_SEMANTIC_WEIGHT,
            ppr: DEFAULT_PPR_WEIGHT,
        }
    }
}
