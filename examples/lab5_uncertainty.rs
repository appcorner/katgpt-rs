//! Lab 5A: use decision margin as a simple signal for abstaining on close calls.
//!
//! Run with `cargo run --example lab5_uncertainty`.

use katgpt_rs::types::Rng;

const ACTIONS: [&str; 3] = ["LLM", "RAG", "SQL"];
const TRAIN_CONTEXTS: [f32; 4] = [0.0, 0.25, 0.75, 1.0];
const EVAL_CONTEXTS: [f32; 9] = [0.0, 0.1, 0.25, 0.4, 0.5, 0.6, 0.75, 0.9, 1.0];
const THRESHOLD_SWEEP: [f32; 7] = [0.0, 0.02, 0.05, 0.10, 0.15, 0.20, 0.30];
const EPISODES: usize = 20_000;
const SEED: u64 = 3_202_609_25;
const LEARNING_RATE: f32 = 0.02;
const EXPLORE_RATE: f32 = 0.15;
const ABSTAIN_THRESHOLD: f32 = 0.10;

struct EvaluationRow {
    context: f32,
    is_train: bool,
    predicted_best: usize,
    best_score: f32,
    second_best: usize,
    second_score: f32,
    margin: f32,
    true_best: usize,
    correct: bool,
}

#[derive(Default)]
struct DecisionSummary {
    total: usize,
    raw_correct: usize,
    execute: usize,
    abstain: usize,
    correct_execute: usize,
    wrong_execute: usize,
    incorrect_caught: usize,
    correct_abstained: usize,
}

impl DecisionSummary {
    fn record(&mut self, row: &EvaluationRow, threshold: f32) {
        self.total += 1;
        if row.correct {
            self.raw_correct += 1;
        }

        if row.margin >= threshold {
            self.execute += 1;
            if row.correct {
                self.correct_execute += 1;
            } else {
                self.wrong_execute += 1;
            }
        } else {
            self.abstain += 1;
            if row.correct {
                self.correct_abstained += 1;
            } else {
                self.incorrect_caught += 1;
            }
        }
    }

    fn from_rows<'a>(rows: impl Iterator<Item = &'a EvaluationRow>, threshold: f32) -> Self {
        let mut summary = Self::default();
        for row in rows {
            summary.record(row, threshold);
        }
        summary
    }

    fn print(&self, label: &str) {
        let raw_accuracy = percent(self.raw_correct, self.total);
        let coverage = percent(self.execute, self.total);
        let selective_accuracy = percent(self.correct_execute, self.execute);
        println!("{label} ({} contexts):", self.total);
        println!(
            "  Raw best-action correct: {}/{} ({raw_accuracy:.1}%)",
            self.raw_correct, self.total
        );
        println!(
            "  EXECUTE: {}  ABSTAIN: {}  Coverage: {coverage:.1}%",
            self.execute, self.abstain
        );
        println!(
            "  Correct EXECUTE: {}  Incorrect EXECUTE: {}  Selective accuracy: {selective_accuracy:.1}%",
            self.correct_execute, self.wrong_execute
        );
        println!(
            "  Incorrect raw predictions caught by ABSTAIN: {}  Correct predictions also abstained: {}",
            self.incorrect_caught, self.correct_abstained
        );
    }
}

fn percent(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 * 100.0 / denominator as f32
    }
}

fn true_probabilities(knowledge_score: f32) -> [f32; 3] {
    [
        0.3 + 0.6 * knowledge_score,
        0.6,
        0.9 - 0.7 * knowledge_score,
    ]
}

// Keep Lab 4's tie behavior: the first action wins ties because replacement
// happens only when a candidate score is strictly greater.
fn best_action(scores: &[f32; 3]) -> usize {
    (1..ACTIONS.len()).fold(0, |best, candidate| {
        if scores[candidate] > scores[best] {
            candidate
        } else {
            best
        }
    })
}

fn best_and_second(scores: &[f32; 3]) -> (usize, usize) {
    let best = best_action(scores);
    let second = (0..ACTIONS.len())
        .filter(|&action| action != best)
        .max_by(|&left, &right| scores[left].total_cmp(&scores[right]))
        .expect("there are three actions, so a second-best action exists");
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

fn print_threshold_sweep(rows: &[EvaluationRow]) {
    println!("\n=== Lab 5A+: Abstention Threshold Sweep ===");
    println!("Threshold | Execute | Abstain | Coverage | Correct Execute | Wrong Execute | Selective Accuracy");
    for threshold in THRESHOLD_SWEEP {
        let summary = DecisionSummary::from_rows(rows.iter(), threshold);
        let selective_accuracy = if summary.execute == 0 {
            "n/a".to_owned()
        } else {
            format!("{:.1}%", percent(summary.correct_execute, summary.execute))
        };
        println!(
            "{threshold:.2}      | {:>7} | {:>7} | {:>7.1}% | {:>15} | {:>13} | {}",
            summary.execute,
            summary.abstain,
            percent(summary.execute, summary.total),
            summary.correct_execute,
            summary.wrong_execute,
            selective_accuracy,
        );
    }
}

fn main() {
    let (weights, total_reward) = train_model();
    println!("=== Lab 5A: Decision Margin and Abstention ===");
    println!("Training contexts: 0.00 0.25 0.75 1.00");
    println!("Training episodes: {EPISODES}  Reward: {total_reward}/{EPISODES}");
    println!("Feature vector: [knowledge_score, 1.0]");
    println!("ABSTAIN_THRESHOLD: {ABSTAIN_THRESHOLD:.2}");
    println!("Decision margin is the relative score gap between the top two actions; it is not a probability or calibrated confidence.");
    println!("ABSTAIN means the fast decision system declines autonomous execution; it does not imply the prediction is wrong.\n");

    println!("Context | Seen?  | Pred Best | Best Score | Second | Second Score | Margin | True Best | Correct? | Decision");
    let rows: Vec<EvaluationRow> = EVAL_CONTEXTS
        .iter()
        .map(|&context| {
            let scores = linear_prediction(&weights, context);
            let (predicted_best, second_best) = best_and_second(&scores);
            let margin = scores[predicted_best] - scores[second_best];
            let true_best = best_action(&true_probabilities(context));
            EvaluationRow {
                context,
                is_train: TRAIN_CONTEXTS.contains(&context),
                predicted_best,
                best_score: scores[predicted_best],
                second_best,
                second_score: scores[second_best],
                margin,
                true_best,
                correct: predicted_best == true_best,
            }
        })
        .collect();

    for row in &rows {
        let decision = if row.margin >= ABSTAIN_THRESHOLD {
            "EXECUTE"
        } else {
            "ABSTAIN"
        };
        println!(
            "{:.2}   | {:<6} | {:<9} | {:>10.3} | {:<6} | {:>12.3} | {:>6.3} | {:<9} | {:<8} | {}",
            row.context,
            if row.is_train { "TRAIN" } else { "UNSEEN" },
            ACTIONS[row.predicted_best],
            row.best_score,
            ACTIONS[row.second_best],
            row.second_score,
            row.margin,
            ACTIONS[row.true_best],
            if row.correct { "yes" } else { "no" },
            decision,
        );
    }

    let summary = DecisionSummary::from_rows(rows.iter(), ABSTAIN_THRESHOLD);
    println!("\nSummary at threshold {ABSTAIN_THRESHOLD:.2}:");
    summary.print("All evaluation contexts");
    DecisionSummary::from_rows(rows.iter().filter(|row| row.is_train), ABSTAIN_THRESHOLD)
        .print("TRAIN contexts only");
    DecisionSummary::from_rows(rows.iter().filter(|row| !row.is_train), ABSTAIN_THRESHOLD)
        .print("UNSEEN contexts only");

    print_threshold_sweep(&rows);
}
