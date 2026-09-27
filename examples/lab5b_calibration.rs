//! Lab 5B: measure how a margin-derived confidence-like score compares with
//! empirical action-selection correctness. This example does not calibrate it.
//!
//! Run with `cargo run --example lab5b_calibration`.

use katgpt_rs::types::Rng;

const ACTIONS: [&str; 3] = ["LLM", "RAG", "SQL"];
const TRAIN_CONTEXTS: [f32; 4] = [0.0, 0.25, 0.75, 1.0];
const EPISODES: usize = 20_000;
const EVALUATIONS: usize = 10_000;
const SEED: u64 = 3_202_609_25;
const EVAL_SEED: u64 = 3_202_609_27;
const LEARNING_RATE: f32 = 0.02;
const EXPLORE_RATE: f32 = 0.15;
const BIN_COUNT: usize = 10;

const BOUNDARY_RANGES: [(&str, f32, f32, bool); 5] = [
    ("[0.00, 0.20)", 0.0, 0.2, false),
    ("[0.20, 0.40)", 0.2, 0.4, false),
    ("[0.40, 0.60)", 0.4, 0.6, false),
    ("[0.60, 0.80)", 0.6, 0.8, false),
    ("[0.80, 1.00]", 0.8, 1.0, true),
];

#[derive(Default)]
struct CalibrationBin {
    count: usize,
    confidence_sum: f64,
    correct: usize,
}

impl CalibrationBin {
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
struct BoundaryBin {
    count: usize,
    confidence_sum: f64,
    correct: usize,
}

impl BoundaryBin {
    fn record(&mut self, confidence: f32, correct: bool) {
        self.count += 1;
        self.confidence_sum += f64::from(confidence);
        self.correct += usize::from(correct);
    }
}

fn true_probabilities(knowledge_score: f32) -> [f32; 3] {
    [
        0.3 + 0.6 * knowledge_score,
        0.6,
        0.9 - 0.7 * knowledge_score,
    ]
}

// Preserve Lab 4/5 tie handling: the first action wins because ties do not
// replace the current best.
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

fn train_model() -> ([[f32; 2]; 3], u32) {
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

fn confidence_classification(bin: &CalibrationBin) -> &'static str {
    let difference = bin.mean_confidence() - bin.accuracy();
    if difference.abs() < 0.02 {
        "CLOSE"
    } else if difference > 0.0 {
        "OVERCONFIDENT"
    } else {
        "UNDERCONFIDENT"
    }
}

fn main() {
    let (weights, training_reward) = train_model();
    println!("Training contexts: 0.00 0.25 0.75 1.00");
    println!("Training episodes: {EPISODES}  Reward: {training_reward}/{EPISODES}");
    println!("Feature vector: [knowledge_score, 1.0]");

    println!("\n=== Lab 5B: Measuring Calibration ===");
    println!("Evaluation decisions: {EVALUATIONS}; evaluation contexts are sampled uniformly from [0.0, 1.0].");
    println!("Naive confidence = clamp(margin × 4.0, 0.0, 1.0); 4.0 is arbitrary, not learned or calibrated.");
    println!("Correctness compares predicted best with the true best action from the environment, not a Bernoulli reward draw.\n");

    let mut rng = Rng::new(EVAL_SEED);
    let mut bins: [CalibrationBin; BIN_COUNT] = std::array::from_fn(|_| CalibrationBin::default());
    let mut boundary_bins: [BoundaryBin; BOUNDARY_RANGES.len()] =
        std::array::from_fn(|_| BoundaryBin::default());
    let mut correct_total = 0_usize;
    let mut confidence_total = 0.0_f64;

    for _ in 0..EVALUATIONS {
        let knowledge_score = rng.uniform();
        let predicted = linear_prediction(&weights, knowledge_score);
        let (predicted_best, second_best) = top_two(&predicted);
        let margin = predicted[predicted_best] - predicted[second_best];
        let naive_confidence = (margin * 4.0).clamp(0.0, 1.0);
        let correct = predicted_best == best_action(&true_probabilities(knowledge_score));

        if correct {
            correct_total += 1;
        }
        confidence_total += f64::from(naive_confidence);
        bins[bin_index(naive_confidence)].record(naive_confidence, correct);

        for (index, &(_, lower, upper, upper_inclusive)) in BOUNDARY_RANGES.iter().enumerate() {
            let in_range = knowledge_score >= lower
                && if upper_inclusive {
                    knowledge_score <= upper
                } else {
                    knowledge_score < upper
                };
            if in_range {
                boundary_bins[index].record(naive_confidence, correct);
                break;
            }
        }
    }

    println!("Bin       | Count | Mean Confidence | Correct | Empirical Accuracy | Gap           | Classification");
    let mut ece = 0.0_f64;
    let mut maximum_gap = 0.0_f64;
    for (index, bin) in bins.iter().enumerate() {
        if bin.count == 0 {
            println!("{:<9} | {:>5} |       n/a       |   n/a   |        n/a         |     n/a       | EMPTY", bin_label(index), 0);
            continue;
        }
        let mean_confidence = bin.mean_confidence();
        let accuracy = bin.accuracy();
        let gap = bin.gap();
        ece += (bin.count as f64 / EVALUATIONS as f64) * gap;
        maximum_gap = maximum_gap.max(gap);
        println!(
            "{:<9} | {:>5} |      {:>6.3}      | {:>7} |       {:>6.3}        |    {:>6.3}      | {}",
            bin_label(index),
            bin.count,
            mean_confidence,
            bin.correct,
            accuracy,
            gap,
            confidence_classification(bin),
        );
    }

    let overall_accuracy = correct_total as f64 / EVALUATIONS as f64;
    let mean_confidence = confidence_total / EVALUATIONS as f64;
    println!(
        "\nOverall decision accuracy: {correct_total}/{EVALUATIONS} ({:.3})",
        overall_accuracy
    );
    println!("ECE: {ece:.5}");
    println!("ECE summarizes how far reported confidence differs from observed correctness. Lower is better; 0 would be perfect on this evaluation.");
    println!("A low ECE alone does not prove that the model is safe or reliable.");
    println!("Maximum populated-bin calibration gap: {maximum_gap:.5}");
    println!("Mean naive confidence: {mean_confidence:.5}");
    println!("Actual overall accuracy: {overall_accuracy:.5}");
    println!(
        "Mean confidence minus actual accuracy: {:+.5} ({})",
        mean_confidence - overall_accuracy,
        if mean_confidence > overall_accuracy {
            "globally optimistic"
        } else {
            "globally pessimistic or equal"
        }
    );

    println!("\nBoundary Analysis");
    println!("Context Range | Count | Mean Confidence | Accuracy | Gap");
    for ((label, _, _, _), bin) in BOUNDARY_RANGES.iter().zip(&boundary_bins) {
        if bin.count == 0 {
            println!("{label:<14} | {:>5} |       n/a       |   n/a    | n/a", 0);
            continue;
        }
        let mean = bin.confidence_sum / bin.count as f64;
        let accuracy = bin.correct as f64 / bin.count as f64;
        println!(
            "{label:<14} | {:>5} |      {:.3}       |  {:.3}   | {:.3}",
            bin.count,
            mean,
            accuracy,
            (mean - accuracy).abs()
        );
    }

    println!("\nFinal educational summary:");
    println!("Predicted reward != probability decision is correct");
    println!("Decision margin != probability decision is correct");
    println!("Mapping margin into [0,1] != calibration");
    println!("Calibration must be checked against observed correctness over many decisions");
}
