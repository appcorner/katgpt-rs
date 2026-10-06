//! Plan 619 T1.4/T1.6 gates — the LatticeMemory hot-path allocation ceiling.
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

use katgpt_core::lattice_memory::{BumpKernel, LatticeConfig, LatticeMemory};

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
