//! `LatticeMemory` vocabulary types — decoupled structs, no logic deps.
//!
//! Plan 619 T1.1: a 2D lattice of delta-rule cells over a preallocated flat
//! slab with RUNTIME `d_k`/`d_v` (the const-generic `DeltaMemoryState` shape
//! needs `generic_const_exprs`, unstable on the pinned toolchain; the flat
//! slab is the recorded alternative). Cell state = offset+length into the
//! slab — never a per-cell heap allocation.
//!
//! Name disambiguation (Plan 619): this is NOT `analytic_lattice` (Plan 330
//! transport operators — unrelated domain).

/// One lattice cell's slab claim.
///
/// `offset`/`len` address the shared slab in `f32` elements; `len` is
/// `d_k * d_v` for every cell today (kept as a field for variable-shape
/// futures, exactly as the plan names it). `writes` counts delta-rule
/// updates — the forgetting axis (overwrite count drives recall decay).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatticeCell {
    /// First slab element this cell owns.
    pub offset: u32,
    /// Slab elements this cell owns (`d_k * d_v` today).
    pub len: u32,
    /// Delta-rule updates applied to this cell (0 = never written).
    pub writes: u32,
}

impl LatticeCell {
    /// A cell that has never been written: no slab claim yet.
    #[must_use]
    pub const fn unwritten() -> Self {
        Self {
            offset: 0,
            len: 0,
            writes: 0,
        }
    }

    /// A cell is live once it has claimed slab space.
    #[must_use]
    pub const fn is_written(&self) -> bool {
        self.writes > 0
    }
}

/// Lattice geometry + addressing configuration.
///
/// Built once, consumed by reference; the seeded E2LSH axes derive from it
/// (`seed` + the scalars → the shared `config_seed` discipline), so the same
/// configuration always produces the same lattice.
#[derive(Clone, Debug)]
pub struct LatticeConfig {
    /// Embedding dimension (the address space input width).
    pub d: usize,
    /// Cell key width (delta-rule state is `d_k * d_v`).
    pub d_k: usize,
    /// Cell value width (delta-rule state is `d_k * d_v`).
    pub d_v: usize,
    /// Cells per axis: `grid[0]` rows × `grid[1]` columns.
    pub grid: [u32; 2],
    /// Per-axis continuous-LSH width `w` (the quantization cell size in
    /// projection units).
    pub w: [f32; 2],
    /// Deterministic seed — same config → same projections (Issue 809 law).
    pub seed: u64,
    /// Hard byte ceiling for the slab; `None` = unbounded (tests/diagnostics).
    /// The constructor refuses configurations over the budget (the T1.5 law,
    /// armed at birth because the refusal is cheap and the failure silent).
    pub byte_budget: Option<usize>,
}

impl LatticeConfig {
    /// Slab elements a written cell owns.
    #[must_use]
    pub const fn cell_len(&self) -> usize {
        self.d_k * self.d_v
    }

    /// Total grid cells.
    #[must_use]
    pub const fn cell_count(&self) -> usize {
        self.grid[0] as usize * self.grid[1] as usize
    }

    /// Slab bytes at full occupancy (every cell written).
    #[must_use]
    pub const fn slab_bytes(&self) -> usize {
        self.cell_count() * self.cell_len() * core::mem::size_of::<f32>()
    }
}

/// A validation failure with the offending value named.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LatticeConfigError {
    /// A dimension or grid side was zero.
    ZeroDimension,
    /// A per-axis LSH width was zero or non-finite.
    BadWidth(usize),
    /// `slab_bytes()` exceeded `byte_budget`.
    OverBudget {
        /// Required slab bytes at full occupancy.
        required: usize,
        /// The configured ceiling.
        budget: usize,
    },
}

impl core::fmt::Display for LatticeConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroDimension => write!(f, "lattice config: d, d_k, d_v and both grid sides must be non-zero"),
            Self::BadWidth(axis) => write!(f, "lattice config: axis {axis} width must be finite and > 0"),
            Self::OverBudget { required, budget } => {
                write!(f, "lattice config: slab needs {required} B, byte_budget is {budget} B")
            }
        }
    }
}

impl std::error::Error for LatticeConfigError {}

/// Validation shared by every constructor.
pub fn validate(config: &LatticeConfig) -> Result<(), LatticeConfigError> {
    if config.d == 0 || config.d_k == 0 || config.d_v == 0 || config.grid[0] == 0 || config.grid[1] == 0 {
        return Err(LatticeConfigError::ZeroDimension);
    }
    for (axis, &w) in config.w.iter().enumerate() {
        if w <= 0.0 || !w.is_finite() {
            return Err(LatticeConfigError::BadWidth(axis));
        }
    }
    if let Some(budget) = config.byte_budget {
        let required = config.slab_bytes();
        if required > budget {
            return Err(LatticeConfigError::OverBudget { required, budget });
        }
    }
    Ok(())
}
