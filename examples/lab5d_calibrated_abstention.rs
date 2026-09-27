//! Lab 5D: compare margin-based and histogram-calibrated confidence abstention.
//!
//! Run with `cargo run --example lab5d_calibrated_abstention`.

use katgpt_rs::types::Rng;

const ACTIONS: [&str; 3] = ["LLM", "RAG", "SQL"];
const TRAIN_CONTEXTS: [f32; 4] = [0.0, 0.25, 0.75, 1.0];
const EPISODES: usize = 20_000;
const CALIBRATION_SIZE: usize = 10_000;
const TEST_SIZE: usize = 20_000;
const SEED: u64 = 3_202_609_25;
const CALIBRATION_SEED: u64 = 3_202_609_28;
const TEST_SEED: u64 = 3_202_609_30;
const LEARNING_RATE: f32 = 0.02;
const EXPLORE_RATE: f32 = 0.15;
const BIN_COUNT: usize = 10;

const MARGIN_THRESHOLDS: [f32; 8] = [0.0, 0.02, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30];
const CONFIDENCE_THRESHOLDS: [f32; 9] = [0.0, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.95, 0.99];
const TARGET_THRESHOLDS: [f32; 4] = [0.80, 0.90, 0.95, 0.99];
const CONTEXT_REGIONS: [(&str, f32, f32, bool); 5] = [
    ("[0.00, 0.20)", 0.0, 0.2, false),
    ("[0.20, 0.40)", 0.2, 0.4, false),
    ("[0.40, 0.60)", 0.4, 0.6, false),
    ("[0.60, 0.80)", 0.6, 0.8, false),
    ("[0.80, 1.00]", 0.8, 1.0, true),
];

#[derive(Clone, Copy)]
struct Decision {
    knowledge_score: f32,
    predicted_action: usize,
    true_action: usize,
    correct: bool,
    margin: f32,
    naive_confidence: f32,
}

#[derive(Clone, Copy)]
struct TestDecision {
    decision: Decision,
    calibrated_confidence: f32,
}

#[derive(Default)]
struct CalibrationBin {
    count: usize,
    correct: usize,
}

impl CalibrationBin {
    fn record(&mut self, correct: bool) {
        self.count += 1;
        self.correct += usize::from(correct);
    }

    fn accuracy(&self) -> f32 {
        self.correct as f32 / self.count as f32
    }
}

struct HistogramCalibrator {
    bins: [CalibrationBin; BIN_COUNT],
}

impl HistogramCalibrator {
    fn fit(decisions: &[Decision]) -> Self {
        let mut bins: [CalibrationBin; BIN_COUNT] =
            std::array::from_fn(|_| CalibrationBin::default());
        for decision in decisions {
            bins[bin_index(decision.naive_confidence)].record(decision.correct);
        }
        Self { bins }
    }

    fn predict(&self, naive_confidence: f32) -> f32 {
        let requested = bin_index(naive_confidence);
        let selected = if self.bins[requested].count > 0 {
            requested
        } else {
            (0..BIN_COUNT)
                .filter(|&index| self.bins[index].count > 0)
                // Tuple ordering makes the lower bin win equal-distance ties.
                .min_by_key(|&index| (index.abs_diff(requested), index))
                .expect("calibration data populates at least one bin")
        };
        self.bins[selected].accuracy()
    }

    fn print_mapping(&self) {
        println!("=== Learned Calibration Mapping ===");
        println!("Naive Bin | Count | Correct | Learned Calibrated Probability");
        let mut has_empty = false;
        for (index, bin) in self.bins.iter().enumerate() {
            if bin.count == 0 {
                has_empty = true;
                println!("{:<9} | {:>5} | {:>7} | n/a", bin_label(index), 0, 0);
            } else {
                println!(
                    "{:<9} | {:>5} | {:>7} | {:.3}",
                    bin_label(index),
                    bin.count,
                    bin.correct,
                    bin.accuracy()
                );
            }
        }
        if has_empty {
            println!("Warning: empty calibration bins use the nearest populated bin; equal-distance ties use the lower index.");
        }
    }
}

#[derive(Clone, Copy)]
struct PolicyResult {
    threshold: f32,
    total: usize,
    execute: usize,
    abstain: usize,
    correct_execute: usize,
    wrong_execute: usize,
}

impl PolicyResult {
    fn from_margin(decisions: &[TestDecision], threshold: f32) -> Self {
        Self::from_predicate(decisions, threshold, |row| row.decision.margin >= threshold)
    }

    fn from_confidence(decisions: &[TestDecision], threshold: f32) -> Self {
        Self::from_predicate(decisions, threshold, |row| {
            row.calibrated_confidence >= threshold
        })
    }

    fn from_predicate(
        decisions: &[TestDecision],
        threshold: f32,
        execute: impl Fn(&TestDecision) -> bool,
    ) -> Self {
        let mut result = Self {
            threshold,
            total: decisions.len(),
            execute: 0,
            abstain: 0,
            correct_execute: 0,
            wrong_execute: 0,
        };
        for row in decisions {
            if execute(row) {
                result.execute += 1;
                if row.decision.correct {
                    result.correct_execute += 1;
                } else {
                    result.wrong_execute += 1;
                }
            } else {
                result.abstain += 1;
            }
        }
        result
    }

    fn coverage(self) -> f64 {
        self.execute as f64 / self.total as f64
    }

    fn selective_accuracy(self) -> Option<f64> {
        (self.execute > 0).then(|| self.correct_execute as f64 / self.execute as f64)
    }

    fn risk(self) -> Option<f64> {
        self.selective_accuracy().map(|accuracy| 1.0 - accuracy)
    }
}

#[derive(Default)]
struct RegionResult {
    total: usize,
    execute: usize,
    correct_execute: usize,
    wrong_execute: usize,
}

fn true_probabilities(knowledge_score: f32) -> [f32; 3] {
    [
        0.3 + 0.6 * knowledge_score,
        0.6,
        0.9 - 0.7 * knowledge_score,
    ]
}

// Match Labs 4B-5C: the first action retains ties since replacement requires >.
fn best_action(scores: &[f32; 3]) -> usize {
    (1..ACTIONS.len()).fold(0, |best, candidate| {
        if scores[candidate] > scores[best] {
            candidate
        } else {
            best
        }
    })
}

fn top_two(scores: &[f32; 3]) -> (usize, usize) {
    let best = best_action(scores);
    let second = (0..ACTIONS.len())
        .filter(|&action| action != best)
        .max_by(|&left, &right| scores[left].total_cmp(&scores[right]))
        .expect("three actions provide a second-best score");
    (best, second)
}

fn linear_prediction(weights: &[[f32; 2]; 3], knowledge_score: f32) -> [f32; 3] {
    let features = [knowledge_score, 1.0];
    std::array::from_fn(|action| {
        weights[action][0] * features[0] + weights[action][1] * features[1]
    })
}

fn train_decision_model() -> ([[f32; 2]; 3], u32) {
    let mut rng = Rng::new(SEED);
    let mut weights = [[0.0_f32; 2]; 3];
    let mut total_reward = 0_u32;

    for episode in 0..EPISODES {
        let knowledge_score = TRAIN_CONTEXTS[episode % TRAIN_CONTEXTS.len()];
        let prediction = linear_prediction(&weights, knowledge_score);
        let action = if rng.uniform() < EXPLORE_RATE {
            (rng.uniform() * ACTIONS.len() as f32) as usize
        } else {
            best_action(&prediction)
        };
        let reward = u32::from(rng.uniform() < true_probabilities(knowledge_score)[action]);
        total_reward += reward;

        let features = [knowledge_score, 1.0];
        let error = prediction[action] - reward as f32;
        weights[action][0] -= LEARNING_RATE * error * features[0];
        weights[action][1] -= LEARNING_RATE * error * features[1];
    }
    (weights, total_reward)
}

fn make_decision(weights: &[[f32; 2]; 3], knowledge_score: f32) -> Decision {
    let predicted_scores = linear_prediction(weights, knowledge_score);
    let (predicted_action, second) = top_two(&predicted_scores);
    let margin = predicted_scores[predicted_action] - predicted_scores[second];
    let naive_confidence = (margin * 4.0).clamp(0.0, 1.0);
    let true_action = best_action(&true_probabilities(knowledge_score));
    Decision {
        knowledge_score,
        predicted_action,
        true_action,
        correct: predicted_action == true_action,
        margin,
        naive_confidence,
    }
}

fn generate_decisions(weights: &[[f32; 2]; 3], count: usize, seed: u64) -> Vec<Decision> {
    let mut rng = Rng::new(seed);
    (0..count)
        .map(|_| make_decision(weights, rng.uniform()))
        .collect()
}

fn bin_index(confidence: f32) -> usize {
    ((confidence * BIN_COUNT as f32).floor() as usize).min(BIN_COUNT - 1)
}

fn bin_label(index: usize) -> String {
    if index == BIN_COUNT - 1 {
        format!("[{:.1}, 1.0]", index as f32 / BIN_COUNT as f32)
    } else {
        format!(
            "[{:.1}, {:.1})",
            index as f32 / BIN_COUNT as f32,
            (index + 1) as f32 / BIN_COUNT as f32
        )
    }
}

fn print_baseline(decisions: &[TestDecision]) {
    let correct = decisions
        .iter()
        .filter(|row| row.decision.predicted_action == row.decision.true_action)
        .count();
    let incorrect = decisions.len() - correct;
    println!("No-abstention baseline (execute every predicted action):");
    println!("Total decisions: {}", decisions.len());
    println!("Correct: {correct}  Incorrect: {incorrect}");
    println!(
        "Accuracy: {:.3}%  Coverage: 100.0%\n",
        correct as f64 * 100.0 / decisions.len() as f64
    );
}

fn print_margin_sweep(results: &[PolicyResult]) {
    println!("\nPolicy A — Margin Threshold Sweep");
    println!("Margin Threshold | Execute | Abstain | Coverage | Correct Execute | Wrong Execute | Selective Accuracy");
    for result in results {
        let accuracy = result
            .selective_accuracy()
            .map(|value| format!("{:.2}%", value * 100.0))
            .unwrap_or_else(|| "n/a".to_owned());
        println!(
            "{:<16.2} | {:>7} | {:>7} | {:>7.2}% | {:>15} | {:>13} | {}",
            result.threshold,
            result.execute,
            result.abstain,
            result.coverage() * 100.0,
            result.correct_execute,
            result.wrong_execute,
            accuracy,
        );
    }
}

fn print_confidence_sweep(results: &[PolicyResult]) {
    println!("\nPolicy B — Calibrated-confidence Threshold Sweep");
    println!("Confidence Threshold | Execute | Abstain | Coverage | Correct Execute | Wrong Execute | Selective Accuracy");
    for result in results {
        let accuracy = result
            .selective_accuracy()
            .map(|value| format!("{:.2}%", value * 100.0))
            .unwrap_or_else(|| "n/a".to_owned());
        println!(
            "{:<20.2} | {:>7} | {:>7} | {:>7.2}% | {:>15} | {:>13} | {}",
            result.threshold,
            result.execute,
            result.abstain,
            result.coverage() * 100.0,
            result.correct_execute,
            result.wrong_execute,
            accuracy,
        );
    }
    println!("A 0.90 threshold means execute when the frozen calibrator estimates about 90% or greater empirical correctness under the calibration distribution; it is not a guarantee of 90% correctness.");
}

fn print_similar_coverage(margin: &[PolicyResult], confidence: &[PolicyResult]) {
    println!("\n=== Similar-Coverage Comparison ===");
    println!("Calibrated Threshold | Cal Coverage | Cal Selective Accuracy | Wrong Cal | Closest Margin Threshold | Margin Coverage | Margin Selective Accuracy | Wrong Margin");
    for calibrated in confidence {
        let closest = margin
            .iter()
            .min_by(|left, right| {
                (left.coverage() - calibrated.coverage())
                    .abs()
                    .total_cmp(&(right.coverage() - calibrated.coverage()).abs())
            })
            .expect("margin sweep has thresholds");
        let cal_accuracy = format_accuracy(calibrated.selective_accuracy());
        let margin_accuracy = format_accuracy(closest.selective_accuracy());
        println!(
            "{:<20.2} | {:>11.2}% | {:>22} | {:>9} | {:>24.2} | {:>14.2}% | {:>26} | {:>12}",
            calibrated.threshold,
            calibrated.coverage() * 100.0,
            cal_accuracy,
            calibrated.wrong_execute,
            closest.threshold,
            closest.coverage() * 100.0,
            margin_accuracy,
            closest.wrong_execute,
        );
    }
    println!(
        "Exploratory comparison only; no universal winner is implied by this synthetic dataset."
    );
}

fn format_accuracy(accuracy: Option<f64>) -> String {
    accuracy
        .map(|value| format!("{:.2}%", value * 100.0))
        .unwrap_or_else(|| "n/a".to_owned())
}

fn print_risk_coverage(margin: &[PolicyResult], confidence: &[PolicyResult]) {
    println!("\nRisk-Coverage View");
    println!("Policy | Threshold | Coverage | Selective Accuracy | Risk");
    for (name, results) in [("Margin", margin), ("Calibrated", confidence)] {
        for result in results {
            let accuracy = format_accuracy(result.selective_accuracy());
            let risk = result
                .risk()
                .map(|value| format!("{:.2}%", value * 100.0))
                .unwrap_or_else(|| "n/a".to_owned());
            println!(
                "{name:<10} | {:>9.2} | {:>7.2}% | {:>18} | {}",
                result.threshold,
                result.coverage() * 100.0,
                accuracy,
                risk,
            );
        }
    }
    println!("Lower coverage may allow lower autonomous decision risk, but the observed values need not be monotonic.");
}

fn print_target_reliability(decisions: &[TestDecision]) {
    println!("\nTarget-Reliability Policy");
    println!("Target Confidence | Coverage | Selective Accuracy | Observed Error Rate | Wrong Autonomous Actions");
    for threshold in TARGET_THRESHOLDS {
        let result = PolicyResult::from_confidence(decisions, threshold);
        let accuracy = result.selective_accuracy();
        let error_rate = accuracy
            .map(|value| format!("{:.2}%", (1.0 - value) * 100.0))
            .unwrap_or_else(|| "n/a".to_owned());
        println!(
            "{threshold:<17.2} | {:>7.2}% | {:>18} | {:>19} | {:>26}",
            result.coverage() * 100.0,
            format_accuracy(accuracy),
            error_rate,
            result.wrong_execute,
        );
    }
    println!("The held-out test set checks whether the empirical interpretation transfers; confidence thresholds do not guarantee the matching accuracy.");
}

fn print_context_regions(decisions: &[TestDecision]) {
    let mut regions: [RegionResult; 5] = std::array::from_fn(|_| RegionResult::default());
    for row in decisions {
        let execute = row.calibrated_confidence >= 0.90;
        for (index, &(_, lower, upper, upper_inclusive)) in CONTEXT_REGIONS.iter().enumerate() {
            let in_region = row.decision.knowledge_score >= lower
                && if upper_inclusive {
                    row.decision.knowledge_score <= upper
                } else {
                    row.decision.knowledge_score < upper
                };
            if in_region {
                regions[index].total += 1;
                if execute {
                    regions[index].execute += 1;
                    if row.decision.correct {
                        regions[index].correct_execute += 1;
                    } else {
                        regions[index].wrong_execute += 1;
                    }
                }
                break;
            }
        }
    }

    println!("\nContext-Region Analysis — calibrated-confidence threshold 0.90");
    println!("Context Range | Total | Execute | Coverage | Correct Execute | Wrong Execute | Selective Accuracy");
    for ((label, _, _, _), region) in CONTEXT_REGIONS.iter().zip(&regions) {
        println!(
            "{label:<14} | {:>5} | {:>7} | {:>7.2}% | {:>15} | {:>13} | {}",
            region.total,
            region.execute,
            region.execute as f64 * 100.0 / region.total as f64,
            region.correct_execute,
            region.wrong_execute,
            format_accuracy(
                (region.execute > 0)
                    .then(|| { region.correct_execute as f64 / region.execute as f64 })
            ),
        );
    }
}

fn main() {
    println!("=== Lab 5D: Calibrated Confidence and Abstention ===");
    println!("Stage 1 — train and freeze the decision model.");
    println!("Training contexts: 0.00 0.25 0.75 1.00; episodes: {EPISODES}; feature vector: [knowledge_score, 1.0]; seed: {SEED}");
    let (weights, training_reward) = train_decision_model();
    println!("Decision model frozen. Training reward: {training_reward}/{EPISODES}.\n");

    println!("Stage 2 — learn and freeze the histogram calibrator from CALIBRATION data (seed {CALIBRATION_SEED}).");
    let calibration_decisions = generate_decisions(&weights, CALIBRATION_SIZE, CALIBRATION_SEED);
    let calibrator = HistogramCalibrator::fit(&calibration_decisions);
    calibrator.print_mapping();
    println!(
        "Calibrator frozen before test generation. Test outcomes never modify this mapping.\n"
    );

    println!("Stage 3 — generate independent TEST decisions (seed {TEST_SEED}).");
    let test_decisions: Vec<TestDecision> = generate_decisions(&weights, TEST_SIZE, TEST_SEED)
        .into_iter()
        .map(|decision| TestDecision {
            calibrated_confidence: calibrator.predict(decision.naive_confidence),
            decision,
        })
        .collect();
    println!(
        "Test decisions: {}. Model and calibrator remain frozen.\n",
        test_decisions.len()
    );

    print_baseline(&test_decisions);

    let margin_results: Vec<_> = MARGIN_THRESHOLDS
        .iter()
        .map(|&threshold| PolicyResult::from_margin(&test_decisions, threshold))
        .collect();
    let confidence_results: Vec<_> = CONFIDENCE_THRESHOLDS
        .iter()
        .map(|&threshold| PolicyResult::from_confidence(&test_decisions, threshold))
        .collect();

    print_margin_sweep(&margin_results);
    print_confidence_sweep(&confidence_results);
    print_similar_coverage(&margin_results, &confidence_results);
    print_risk_coverage(&margin_results, &confidence_results);
    print_target_reliability(&test_decisions);
    print_context_regions(&test_decisions);

    let invariant_checks =
        test_decisions.len() * (MARGIN_THRESHOLDS.len() + CONFIDENCE_THRESHOLDS.len());
    let invariant_pass = test_decisions.iter().all(|row| {
        let selected_before_abstention = row.decision.predicted_action;
        let margin_candidates_match = MARGIN_THRESHOLDS.iter().all(|&threshold| {
            let candidate_action = row.decision.predicted_action;
            let executed_action = (row.decision.margin >= threshold).then_some(candidate_action);
            candidate_action == selected_before_abstention
                && executed_action.is_none_or(|action| action == selected_before_abstention)
        });
        let calibrated_candidates_match = CONFIDENCE_THRESHOLDS.iter().all(|&threshold| {
            let candidate_action = row.decision.predicted_action;
            let executed_action =
                (row.calibrated_confidence >= threshold).then_some(candidate_action);
            candidate_action == selected_before_abstention
                && executed_action.is_none_or(|action| action == selected_before_abstention)
        });
        margin_candidates_match && calibrated_candidates_match
    });
    println!("\nImportant Invariant");
    println!("predicted action before abstention == predicted action considered after abstention");
    println!(
        "{} ({invariant_checks} policy decisions checked)",
        if invariant_pass { "PASS" } else { "FAIL" }
    );

    println!("\nEducational Summary:");
    println!("Margin threshold asks: \"Is the winning score sufficiently separated from the runner-up?\"");
    println!("Calibrated-confidence threshold asks: \"Historically, how often were decisions with this signal correct?\"");
    println!("Abstention trades coverage for reliability.");
    println!("Calibration does not make the decision model smarter.");
    println!("Calibration does not fix the decision boundary.");
    println!("A calibrated probability is empirical evidence, not a guarantee.");
    println!("Global calibration can hide context-specific failure regions.");

    println!("\nLimitations:");
    println!("- Synthetic one-dimensional environment");
    println!("- Histogram calibration is coarse");
    println!("- Calibration and test distributions are intentionally similar");
    println!("- Distribution shift is not tested here");
    println!("- Confidence threshold is not a safety guarantee");
    println!("- A real AI Worker may require different thresholds for different action risks");
    println!("- More advanced calibration methods are not evaluated here");
}
