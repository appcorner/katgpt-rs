//! Fixture gate for the Issue 919 T1 spike-census sidecars.
//!
//! Each committed sidecar (`tests/fixtures/spike_census/*.json`, emitted by
//! `examples/spike_census.rs` from the riir-train data packs) must:
//! 1. hash to exactly its `.blake3` pin (the committable-artifact law), and
//! 2. stay internally consistent: the summary's spike/emerging block lists
//!    re-derive from the per-block `max_s` values at the census's own bars
//!    (10× / 2× the median) — a hand-edited or stale fixture reds.
//!
//! The gates assert COUNTS and digests, never exit codes.

use std::path::Path;

const FIXTURES: &[&str] = &[
    "gemma-2-2b-it-f16",
    "Ternary-Bonsai-8B-Q2_0",
    "Ternary-Bonsai-27B-Q2_0",
];

fn fixture_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spike_census")
}

#[test]
fn sidecars_match_their_blake3_pins() {
    for name in FIXTURES {
        let dir = fixture_dir();
        let body = std::fs::read_to_string(dir.join(format!("{name}.json")))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let pin = std::fs::read_to_string(dir.join(format!("{name}.json.blake3")))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let want = pin.split_whitespace().next().expect("pin hex");
        let got = blake3::hash(body.as_bytes()).to_hex().to_string();
        assert_eq!(got, want, "{name}: sidecar content does not match its pin");
    }
}

#[test]
fn sidecars_stay_internally_consistent() {
    for name in FIXTURES {
        let dir = fixture_dir();
        let body = std::fs::read_to_string(dir.join(format!("{name}.json")))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let v: serde_json::Value = serde_json::from_str(&body).expect("parse");
        assert_eq!(v["schema"], "spike_census_v1", "{name}: schema");

        let blocks = v["blocks"].as_array().expect("blocks array");
        assert!(!blocks.is_empty(), "{name}: no blocks");
        let mut maxima: Vec<f64> = blocks
            .iter()
            .map(|b| b["max_s"].as_f64().expect("max_s f64"))
            .collect();
        assert!(maxima.iter().all(|s| s.is_finite()), "{name}: non-finite max_s");
        maxima.sort_by(|a, b| a.total_cmp(b));
        let median = maxima[maxima.len() / 2];

        let summary = &v["summary"];
        let spike: Vec<usize> = summary["spike_blocks"]
            .as_array()
            .expect("spike_blocks")
            .iter()
            .map(|x| x.as_u64().unwrap() as usize)
            .collect();
        let emerging: Vec<usize> = summary["emerging_blocks_2x"]
            .as_array()
            .expect("emerging_blocks_2x")
            .iter()
            .map(|x| x.as_u64().unwrap() as usize)
            .collect();

        // Re-derive both lists at the census's own bars.
        let want_spike: Vec<usize> = blocks
            .iter()
            .filter(|b| median > 0.0 && b["max_s"].as_f64().unwrap() >= median * 10.0)
            .map(|b| b["idx"].as_u64().unwrap() as usize)
            .collect();
        let want_emerging: Vec<usize> = blocks
            .iter()
            .filter(|b| {
                let s = b["max_s"].as_f64().unwrap();
                median > 0.0 && s >= median * 2.0 && s < median * 10.0
            })
            .map(|b| b["idx"].as_u64().unwrap() as usize)
            .collect();
        assert_eq!(spike, want_spike, "{name}: spike_blocks drifted");
        assert_eq!(emerging, want_emerging, "{name}: emerging_blocks_2x drifted");

        // Every block carries its top_k channels with the block's max_s at
        // the head, and spike/emerging blocks carry a normalized s⋆.
        for b in blocks {
            let channels = b["channels"].as_array().expect("channels array");
            assert!(!channels.is_empty(), "{name} blk {}: empty channels", b["idx"]);
            let top_s = channels[0]["s"].as_f64().expect("s f64");
            assert!(
                (top_s - b["max_s"].as_f64().unwrap()).abs() < 1e-30,
                "{name} blk {}: max_s != head channel s",
                b["idx"]
            );
            let is_flagged = spike.contains(&(b["idx"].as_u64().unwrap() as usize))
                || emerging.contains(&(b["idx"].as_u64().unwrap() as usize));
            if is_flagged {
                let star = b["s_star"].as_array().expect("s_star array");
                assert!(!star.is_empty(), "{name} blk {}: flagged but no s⋆", b["idx"]);
                let norm: f64 = star
                    .iter()
                    .map(|x| x.as_f64().unwrap().powi(2))
                    .sum::<f64>()
                    .sqrt();
                assert!(
                    (norm - 1.0).abs() < 1e-2,
                    "{name} blk {}: s⋆ not normalized (‖·‖={norm})",
                    b["idx"]
                );
            }
        }
    }
}
