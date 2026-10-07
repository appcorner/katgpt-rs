//! Plan 619 T1.4/T1.6 gates — the LatticeMemory hot-path allocation ceiling
//! — plus the T1.9 freeze/thaw gates (snapshot/restore/commitment, the Plan
//! 199 T1.B consumer surface).
//!
//! The G4 law: after construction, every write/read/score is heap-free (the
//! slab, the cell table and the delta-rule scratch are allocated once in
//! `new`; the read path is direct row-wise accumulation needing no scratch).
//! Counted with the shared `TrackingAllocator` per-thread counters.
//!
//! Run (dev or the Issue-741 release posture):
//!
//! ```sh
//! cargo test -p katgpt-core --features lattice_memory --test lattice_memory_gates
//! cargo test -p katgpt-core --features lattice_memory,alloc_tracking \
//!     --release --test lattice_memory_gates
//! ```
//!
//! The allocator arms live behind
//! `any(debug_assertions, feature = "alloc_tracking")` (Issue 741 — the
//! profile is not a capability, the feature is); without either, the
//! disclosure test below still runs and prints the LOUD skip — a green run
//! that measured nothing is not a G4 pass.
#![cfg(feature = "lattice_memory")]

#[cfg(any(debug_assertions, feature = "alloc_tracking"))]
#[global_allocator]
static GATE_ALLOC: katgpt_core::alloc::TrackingAllocator =
    katgpt_core::alloc::TrackingAllocator;

use katgpt_core::lattice_memory::{
    BumpKernel, LatticeCell, LatticeConfig, LatticeMemory, LatticeSnapshot, LatticeSnapshotError,
};

fn config() -> LatticeConfig {
    LatticeConfig {
        d: 16,
        d_k: 4,
        d_v: 4,
        grid: [64, 64],
        w: [2.0, 2.0],
        seed: 42,
        byte_budget: None,
        warp: None,
        bump_kernel: BumpKernel::Tent,
    }
}

/// Always runs: names the posture that measured nothing, so a green count
/// over zero tests can't be read as a G4 pass (the bench_818 convention).
#[test]
fn alloc_gate_posture_disclosure() {
    if cfg!(any(debug_assertions, feature = "alloc_tracking")) {
        println!("alloc gate: ARMED (TrackingAllocator installed)");
    } else {
        println!(
            "G4 alloc-free ⛔ NOT MEASURED — this profile compiles no allocator. \
             Re-run with --features alloc_tracking (release) or in dev; \
             a green run without it is not a G4 pass."
        );
    }
}

#[test]
#[cfg(any(debug_assertions, feature = "alloc_tracking"))]
fn hot_path_is_alloc_free() {
    let mut lat = LatticeMemory::new(config()).unwrap();
    let x = [0.35_f32; 16];
    let k = [0.5_f32, 0.5, 0.5, 0.5];
    let v = [1.0_f32, 0.0, 0.0, 0.0];
    let mut out = [0.0_f32; 4];
    // Warm both paths (first-touch must not count against the hot loop).
    lat.write_delta(&x, &k, &v, 1.0);
    lat.read_cells(&x, &k, &mut out);
    lat.read_blend(&x, &mut out);
    let _ = lat.read_scored(&x, &k, &mut out, 4.0);
    let mut w = v;
    katgpt_core::alloc::reset_alloc_stats();
    for i in 0..1000_u32 {
        w[0] = (i & 0xFF) as f32;
        lat.write_delta(&x, &k, &w, 0.5);
        lat.write_value(&x, &w);
        lat.read_cells(&x, &k, &mut out);
        lat.read_blend(&x, &mut out);
        let score = lat.read_scored(&x, &k, &mut out, 4.0);
        std::hint::black_box(score);
        std::hint::black_box(&out);
    }
    let (count, bytes) = katgpt_core::alloc::get_alloc_stats();
    assert_eq!(
        count, 0,
        "hot path allocated {count} times ({bytes} B) — the G4 law is 0 after construction"
    );
}

#[test]
#[cfg(any(debug_assertions, feature = "alloc_tracking"))]
fn sized_constructor_reports_but_hot_path_stays_clean() {
    // The sizer projects + fits at CONSTRUCTION time (documented); the hot
    // path after construction is the same law as above.
    let mut rng = fastrand::Rng::with_seed(9);
    let samples: Vec<Vec<f32>> = (0..64)
        .map(|_| (0..16).map(|_| rng.f32() * 2.0 - 1.0).collect())
        .collect();
    let refs: Vec<&[f32]> = samples.iter().map(|s| s.as_slice()).collect();
    let mut lat =
        LatticeMemory::sized(16, 4, 4, 7, &refs, 64, 0.1, None).expect("sized lattice");
    let x = samples[0].clone();
    let k = [0.5_f32; 4];
    let mut out = [0.0_f32; 4];
    lat.write_delta(&x, &k, &[1.0; 4], 1.0);
    lat.read_cells(&x, &k, &mut out);
    katgpt_core::alloc::reset_alloc_stats();
    for (i, s) in samples.iter().enumerate().skip(1) {
        let mut vv = [0.0_f32; 4];
        vv[0] = i as f32;
        lat.write_delta(s, &k, &vv, 1.0);
        lat.read_cells(s, &k, &mut out);
        std::hint::black_box(&out);
    }
    let (count, _) = katgpt_core::alloc::get_alloc_stats();
    assert_eq!(count, 0, "sized-lattice hot path allocated {count} times");
}

// ── T1.9 — freeze/thaw (Plan 199 T1.B consumer surface) ───────────────

#[test]
fn snapshot_roundtrip_restores_exact_state() {
    let mut lat = LatticeMemory::new(config()).unwrap();
    let k = [0.5_f32; 4];
    let xa = [0.35_f32; 16];
    let xb =
        [0.9_f32, 0.1, 0.0, 0.8, 0.2, 0.0, 0.7, 0.3, 0.0, 0.6, 0.4, 0.0, 0.5, 0.5, 0.0, 0.4];
    lat.write_delta(&xa, &k, &[1.0, 0.0, 0.0, 0.0], 1.0);
    lat.write_delta(&xb, &k, &[0.0, 1.0, 0.0, 0.0], 0.5);
    lat.write_value(&xa, &[2.0, 0.0, 0.0, 0.0]);
    let mut blend_a = [0.0_f32; 4];
    let mut cells_b = [0.0_f32; 4];
    lat.read_blend(&xa, &mut blend_a);
    lat.read_cells(&xb, &k, &mut cells_b);
    let written_at_snap = lat.written_cells();
    let cursor_at_snap = lat.cursor();

    let snap = lat.snapshot();

    // Mutate past the snapshot; the thaw must undo it exactly.
    lat.write_delta(&xb, &k, &[0.0, 0.0, 1.0, 0.0], 1.0);

    // The artifact survives a serde round trip untouched.
    let bytes = serde_json::to_vec(&snap).expect("serialize snapshot");
    let thawed: LatticeSnapshot = serde_json::from_slice(&bytes).expect("deserialize snapshot");
    assert_eq!(snap.cells, thawed.cells, "serde round trip altered the cell table");
    assert_eq!(snap.slab, thawed.slab, "serde round trip altered the slab");

    lat.restore(&thawed).expect("restore");
    let mut blend_after = [0.0_f32; 4];
    let mut cells_after = [0.0_f32; 4];
    lat.read_blend(&xa, &mut blend_after);
    lat.read_cells(&xb, &k, &mut cells_after);
    assert_eq!(blend_a, blend_after, "blend must be bitwise identical after thaw");
    assert_eq!(cells_b, cells_after, "cell read must be bitwise identical after thaw");
    assert_eq!(lat.written_cells(), written_at_snap);
    assert_eq!(lat.cursor(), cursor_at_snap, "thaw must restore the slab cursor exactly");

    // Writes continue from the restored cursor: a post-thaw write to a
    // FRESH cell (not one of the two already-written primaries) claims slab
    // from exactly where the snapshot froze it.
    let pa = lat.primary_cell(&xa);
    let pb = lat.primary_cell(&xb);
    let mut fresh = None;
    for t in 1..64_u32 {
        let mut cand = [0.0_f32; 16];
        cand[0] = t as f32 * 0.13;
        cand[1] = 0.7;
        cand[2] = -0.4;
        let pc = lat.primary_cell(&cand);
        if pc != pa && pc != pb {
            fresh = Some(cand);
            break;
        }
    }
    let xc = fresh.expect("no fresh primary cell in 64 probes");
    lat.write_delta(&xc, &k, &[0.0, 0.0, 0.0, 1.0], 1.0);
    assert_eq!(lat.written_cells(), written_at_snap + 1, "fresh cell not claimed post-thaw");
    assert!(lat.cursor() > cursor_at_snap, "cursor did not continue past thaw");
}

#[test]
fn commitment_binds_content_and_geometry() {
    let build = || {
        let mut lat = LatticeMemory::new(config()).unwrap();
        lat.write_delta(&[0.35_f32; 16], &[0.5_f32; 4], &[1.0, 0.0, 0.0, 0.0], 1.0);
        lat
    };
    let a = build();
    let b = build();
    assert_eq!(a.commitment(), b.commitment(), "identical state must commit identically");

    // Empty lattices of different geometry never share a commitment (the
    // geometry header), even before any write.
    let empty_a = LatticeMemory::new(config()).unwrap();
    let mut small_cfg = config();
    small_cfg.grid = [32, 32];
    let empty_small = LatticeMemory::new(small_cfg).unwrap();
    assert_ne!(empty_a.commitment(), empty_small.commitment(), "geometry is in the header");

    // One extra write moves the commitment; the freeze/thaw identity holds.
    let x2 =
        [0.9_f32, 0.1, 0.0, 0.8, 0.2, 0.0, 0.7, 0.3, 0.0, 0.6, 0.4, 0.0, 0.5, 0.5, 0.0, 0.4];
    let mut c = build();
    let frozen = c.commitment();
    let snap = c.snapshot();
    c.write_delta(&x2, &[0.5_f32; 4], &[0.0, 1.0, 0.0, 0.0], 0.5);
    assert_ne!(frozen, c.commitment(), "a written byte must move the commitment");
    c.restore(&snap).expect("restore");
    assert_eq!(frozen, c.commitment(), "thaw must return the frozen commitment");
}

#[test]
fn restore_refuses_mismatched_geometry_and_corrupt_snapshots() {
    let mut lat = LatticeMemory::new(config()).unwrap();
    let snap = lat.snapshot();

    // Different geometry → refusal naming both sides.
    let mut other_cfg = config();
    other_cfg.grid = [32, 32];
    let mut other = LatticeMemory::new(other_cfg).unwrap();
    match other.restore(&snap) {
        Err(LatticeSnapshotError::GeometryMismatch { expected_cells, found_cells, .. }) => {
            assert_eq!(expected_cells, 32 * 32);
            assert_eq!(found_cells, snap.cells.len());
        }
        res => panic!("expected GeometryMismatch, got {res:?}"),
    }

    // Corrupt cursor → refusal.
    let mut bad_cursor = snap.clone();
    bad_cursor.cursor = bad_cursor.slab.len() + 1;
    assert!(matches!(
        lat.restore(&bad_cursor),
        Err(LatticeSnapshotError::CursorOutOfBounds { .. })
    ));

    // A cell claiming past the slab → refusal (a thawed lie can never
    // reach the hot path's slab indexing).
    let mut liar = snap.clone();
    liar.cells[0] = LatticeCell { offset: liar.slab.len() as u32 - 1, len: 8, writes: 1 };
    assert!(matches!(
        lat.restore(&liar),
        Err(LatticeSnapshotError::CellOutOfBounds { .. })
    ));

    // Every refusal happens before any write: the target is untouched.
    let untouched = lat.commitment();
    lat.restore(&bad_cursor).unwrap_err();
    lat.restore(&liar).unwrap_err();
    assert_eq!(untouched, lat.commitment(), "a refused restore must not tear state");
}
