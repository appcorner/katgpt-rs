//! Lab 5C: learn a histogram mapping from margin-derived scores to empirical
//! correctness, then evaluate it on an independent frozen test set.
//!
//! Run with `cargo run --example lab5c_calibrated_confidence`.

use katgpt_rs::types::Rng;

const ACTIONS: [&str; 3] = ["LLM", "RAG", "SQL"];
const TRAIN_CONTEXTS: [f32; 4] = [0.0, 0.25, 0.75, 1.0];
const EPISODES: usize = 20_000;
const DATASET_SIZE: usize = 10_000;
const SEED: u64 = 3_202_609_25;
const CALIBRATION_SEED: u64 = 3_202_609_28;
const TEST_SEED: u64 = 3_202_609_29;
const LEARNING_RATE: f32 = 0.02;
const EXPLORE_RATE: f32 = 0.15;
const BIN_COUNT: usize = 10;

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
    naive_confidence: f32,
    correct: bool,
}

#[derive(Default)]
struct CalibrationBin {
    count: usize,
    naive_confidence_sum: f64,
    correct: usize,
}

impl CalibrationBin {
    fn record(&mut self, confidence: f32, correct: bool) {
        self.count += 1;
        self.naive_confidence_sum += f64::from(confidence);
        self.correct += usize::from(correct);
    }

    fn mean_naive_confidence(&self) -> f64 {
        self.naive_confidence_sum / self.count as f64
    }

    fn empirical_accuracy(&self) -> f64 {
        self.correct as f64 / self.count as f64
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
            bins[bin_index(decision.naive_confidence)]
                .record(decision.naive_confidence, decision.correct);
        }
        Self { bins }
    }

    fn calibrated_probability(&self, naive_confidence: f32) -> f32 {
        let requested_bin = bin_index(naive_confidence);
        let selected_bin = if self.bins[requested_bin].count > 0 {
            requested_bin
        } else {
            self.nearest_populated_bin(requested_bin)
        };
        self.bins[selected_bin].empirical_accuracy() as f32
    }

    fn nearest_populated_bin(&self, requested_bin: usize) -> usize {
        (0..BIN_COUNT)
            .filter(|&index| self.bins[index].count > 0)
            .min_by_key(|&index| (index.abs_diff(requested_bin), index))
            .expect("the non-empty calibration dataset populates at least one bin")
    }

    fn print_mapping(&self) {
        println!("=== Learned Calibration Mapping ===");
        println!("Naive Bin | Calibration Count | Mean Naive Confidence | Empirical Accuracy | Learned Calibrated Probability");
        let mut has_empty_bin = false;
        for (index, bin) in self.bins.iter().enumerate() {
            if bin.count == 0 {
                has_empty_bin = true;
                println!(
                    "{:<9} | {:>17} |         n/a          |        n/a         |             n/a",
                    bin_label(index),
                    0
                );
            } else {
                let empirical_accuracy = bin.empirical_accuracy();
                println!(
                    "{:<9} | {:>17} |        {:>6.3}         |       {:>6.3}       |             {:>6.3}",
                    bin_label(index),
                    bin.count,
                    bin.mean_naive_confidence(),
                    empirical_accuracy,
                    empirical_accuracy,
                );
            }
        }
        if has_empty_bin {
            println!("Warning: empty calibration bins use the nearest populated bin; equal-distance ties select the lower-index bin.");
        }
        println!("The learned mapping is now frozen; test outcomes will not modify it.\n");
    }
}

#[derive(Default)]
struct MetricBin {
    count: usize,
    confidence_sum: f64,
    correct: usize,
}

impl MetricBin {
    fn record(&mut self, confidence: f32, correct: bool) {
        self.count += 1;
        self.confidence_sum += f64::from(confidence);
        self.correct += usize::from(correct);
    }

    fn mean_confidence(&self) -> f64 {
        self.confidence_sum / self.count as f64
    }

    fn accuracy(&self) -> f64 {
        self.correct as f64 / self.count as f64
    }

    fn gap(&self) -> f64 {
        (self.mean_confidence() - self.accuracy()).abs()
    }
}

#[derive(Default)]
struct ConfidenceMetrics {
    bins: [MetricBin; BIN_COUNT],
    total: usize,
    correct: usize,
    confidence_sum: f64,
}

impl ConfidenceMetrics {
    fn record(&mut self, confidence: f32, correct: bool) {
        self.total += 1;
        self.correct += usize::from(correct);
        self.confidence_sum += f64::from(confidence);
        self.bins[bin_index(confidence)].record(confidence, correct);
    }

    fn accuracy(&self) -> f64 {
        self.correct as f64 / self.total as f64
    }

    fn mean_confidence(&self) -> f64 {
        self.confidence_sum / self.total as f64
    }

    fn ece(&self) -> f64 {
        self.bins
            .iter()
            .filter(|bin| bin.count > 0)
            .map(|bin| (bin.count as f64 / self.total as f64) * bin.gap())
            .sum()
    }

    fn maximum_gap(&self) -> f64 {
        self.bins
            .iter()
            .filter(|bin| bin.count > 0)
            .map(MetricBin::gap)
            .fold(0.0, f64::max)
    }

    fn print_table(&self, title: &str, confidence_label: &str) {
        println!("{title}");
        println!("Bin       | Count | Mean {confidence_label:<19} | Correct | Accuracy | Gap");
        for (index, bin) in self.bins.iter().enumerate() {
            if bin.count == 0 {
                println!(
                    "{:<9} | {:>5} |          n/a          |   n/a   |   n/a    | n/a",
                    bin_label(index),
                    0
                );
            } else {
                println!(
                    "{:<9} | {:>5} |         {:>6.3}        | {:>7} |  {:>6.3}  | {:.3}",
                    bin_label(index),
                    bin.count,
                    bin.mean_confidence(),
                    bin.correct,
                    bin.accuracy(),
                    bin.gap(),
                );
            }
        }
        println!("Decision accuracy: {:.3}", self.accuracy());
        println!("Mean reported confidence: {:.5}", self.mean_confidence());
        println!("ECE: {:.5}", self.ece());
        println!("Maximum calibration error: {:.5}\n", self.maximum_gap());
    }
}

#[derive(Default)]
struct ContextRegionMetrics {
    count: usize,
    correct: usize,
    naive_confidence_sum: f64,
    calibrated_confidence_sum: f64,
}

impl ContextRegionMetrics {
    fn record(&mut self, decision: &Decision, calibrated_confidence: f32) {
        self.count += 1;
        self.correct += usize::from(decision.correct);
        self.naive_confidence_sum += f64::from(decision.naive_confidence);
        self.calibrated_confidence_sum += f64::from(calibrated_confidence);
    }
}

fn true_probabilities(knowledge_score: f32) -> [f32; 3] {
    [
        0.3 + 0.6 * knowledge_score,
        0.6,
        0.9 - 0.7 * knowledge_score,
    ]
}

// Match Lab 4/5 tie handling: the earliest action retains a tie.
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
    let predictions = linear_prediction(weights, knowledge_score);
    let (best, second) = top_two(&predictions);
    let margin = predictions[best] - predictions[second];
    let naive_confidence = (margin * 4.0).clamp(0.0, 1.0);
    let correct = best == best_action(&true_probabilities(knowledge_score));
    Decision {
        knowledge_score,
        naive_confidence,
        correct,
    }
}

fn generate_decisions(weights: &[[f32; 2]; 3], seed: u64) -> Vec<Decision> {
    let mut rng = Rng::new(seed);
    (0..DATASET_SIZE)
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

fn build_confidence_metrics(
    decisions: &[Decision],
    calibrator: &HistogramCalibrator,
) -> (
    ConfidenceMetrics,
    ConfidenceMetrics,
    [ContextRegionMetrics; 5],
) {
    let mut naive_metrics = ConfidenceMetrics::default();
    let mut calibrated_metrics = ConfidenceMetrics::default();
    let mut context_regions: [ContextRegionMetrics; 5] =
        std::array::from_fn(|_| ContextRegionMetrics::default());

    for decision in decisions {
        let calibrated = calibrator.calibrated_probability(decision.naive_confidence);
        naive_metrics.record(decision.naive_confidence, decision.correct);
        // ConfidenceMetrics bins each sample using the confidence passed here,
        // so this table bins by calibrated confidence after calibration.
        calibrated_metrics.record(calibrated, decision.correct);

        for (index, &(_, lower, upper, upper_inclusive)) in CONTEXT_REGIONS.iter().enumerate() {
            let in_region = decision.knowledge_score >= lower
                && if upper_inclusive {
                    decision.knowledge_score <= upper
                } else {
                    decision.knowledge_score < upper
                };
            if in_region {
                context_regions[index].record(decision, calibrated);
                break;
            }
        }
    }

    (naive_metrics, calibrated_metrics, context_regions)
}

fn print_context_regions(regions: &[ContextRegionMetrics; 5]) {
    println!("\nAdditional Generalization Check");
    println!("Context Range | Count | Accuracy | Mean Naive Conf | Mean Calibrated Conf | Naive Gap | Calibrated Gap");
    for ((label, _, _, _), region) in CONTEXT_REGIONS.iter().zip(regions) {
        let accuracy = region.correct as f64 / region.count as f64;
        let mean_naive = region.naive_confidence_sum / region.count as f64;
        let mean_calibrated = region.calibrated_confidence_sum / region.count as f64;
        println!(
            "{label:<14} | {:>5} |  {:.3}   |      {:.3}       |         {:.3}         |   {:.3}    |     {:.3}",
            region.count,
            accuracy,
            mean_naive,
            mean_calibrated,
            (mean_naive - accuracy).abs(),
            (mean_calibrated - accuracy).abs(),
        );
    }
}

fn main() {
    println!("Stage 1 — train the decision model, then freeze it.");
    println!("Training contexts: 0.00 0.25 0.75 1.00; episodes: {EPISODES}");
    println!("Feature vector: [knowledge_score, 1.0]; decision-model seed: {SEED}");
    let (weights, training_reward) = train_decision_model();
    println!("Decision model frozen after training; reward: {training_reward}/{EPISODES}.\n");

    println!("Stage 2 — generate a separate CALIBRATION dataset (seed {CALIBRATION_SEED}).");
    let calibration_decisions = generate_decisions(&weights, CALIBRATION_SEED);
    println!("Calibration decisions: {}. Correctness uses the true best action, not a random reward draw.", calibration_decisions.len());
    let calibrator = HistogramCalibrator::fit(&calibration_decisions);
    calibrator.print_mapping();

    println!("\nStage 3 — evaluate on an independent TEST dataset (seed {TEST_SEED}).");
    println!("The calibrator is frozen; TEST outcomes are used only for evaluation.");
    let test_decisions = generate_decisions(&weights, TEST_SEED);
    let (before, after, context_regions) = build_confidence_metrics(&test_decisions, &calibrator);

    println!("\n=== Lab 5C: Calibration on Independent Test Data ===");
    before.print_table("Before Calibration", "Naive Confidence");
    after.print_table("After Calibration", "Calibrated Confidence");

    println!("Comparison");
    println!("Metric | Before | After | Change (After - Before)");
    println!(
        "Decision accuracy | {:.5} | {:.5} | {:+.5}",
        before.accuracy(),
        after.accuracy(),
        after.accuracy() - before.accuracy()
    );
    println!(
        "Mean reported confidence | {:.5} | {:.5} | {:+.5}",
        before.mean_confidence(),
        after.mean_confidence(),
        after.mean_confidence() - before.mean_confidence()
    );
    println!(
        "ECE | {:.5} | {:.5} | {:+.5}",
        before.ece(),
        after.ece(),
        after.ece() - before.ece()
    );
    println!(
        "Maximum calibration error | {:.5} | {:.5} | {:+.5}",
        before.maximum_gap(),
        after.maximum_gap(),
        after.maximum_gap() - before.maximum_gap()
    );
    if before.correct == after.correct {
        println!(
            "Decision accuracy invariant: PASS (calibration did not change the selected actions)."
        );
    } else {
        println!("BUG: decision accuracy changed after calibration; calibration must not change selected actions.");
    }

    print_context_regions(&context_regions);

    println!("\nEducational Summary:");
    println!("Calibration changes reported confidence, not the chosen action.");
    println!("Decision accuracy before calibration == decision accuracy after calibration.");
    println!("The calibration mapping is learned only from the calibration dataset.");
    println!("The test dataset is used only to evaluate the frozen mapping.");
    println!("A confidence value being inside [0,1] does not make it a probability.");
    println!("Calibration gives a confidence score probabilistic meaning only to the extent supported by held-out empirical evidence.");

    println!("\nLimitations:");
    println!("- This is histogram binning for education, not a production recommendation.");
    println!("- Results depend on the calibration-data distribution.");
    println!("- Calibration can degrade under distribution shift.");
    println!("- Good ECE does not guarantee good decisions.");
    println!("- Calibration does not fix a wrong decision boundary.");
    println!("- Per-region calibration may differ from global calibration.");
}
