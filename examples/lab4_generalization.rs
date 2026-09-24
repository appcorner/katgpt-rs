//! Lab 4: memorizing observed contexts versus generalizing to a new one.
//!
//! Run with `cargo run --example lab4_generalization`.

use katgpt_rs::pruners::BanditStats;
use katgpt_rs::types::Rng;

const ACTIONS: [&str; 3] = ["LLM", "RAG", "SQL"];
const TRAIN_CONTEXTS: [f32; 4] = [0.0, 0.25, 0.75, 1.0];
const EVAL_CONTEXTS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
const SWEEP_CONTEXTS: [f32; 9] = [0.0, 0.1, 0.25, 0.4, 0.5, 0.6, 0.75, 0.9, 1.0];
const EPISODES: usize = 20_000;
const SEED: u64 = 3_202_609_25;
const LEARNING_RATE: f32 = 0.02;
const EXPLORE_RATE: f32 = 0.15;

#[derive(Default)]
struct EvaluationMetrics {
    absolute_error: [f32; 3],
    squared_error: f32,
    correct_best: usize,
    contexts: usize,
}

impl EvaluationMetrics {
    fn record(&mut self, truth: [f32; 3], predicted: [f32; 3]) {
        for action in 0..ACTIONS.len() {
            let error = predicted[action] - truth[action];
            self.absolute_error[action] += error.abs();
            self.squared_error += error * error;
        }
        self.correct_best += usize::from(best_action(&predicted) == best_action(&truth));
        self.contexts += 1;
    }

    fn print(&self, label: &str) {
        let action_values = self.contexts as f32;
        let all_values = (self.contexts * ACTIONS.len()) as f32;
        let per_action_mae = self.absolute_error.map(|error| error / action_values);
        let overall_mae = self.absolute_error.iter().sum::<f32>() / all_values;
        let mse = self.squared_error / all_values;
        let accuracy = self.correct_best as f32 / action_values;

        println!("{label} ({} contexts)", self.contexts);
        println!(
            "  MAE: LLM={:.5}  RAG={:.5}  SQL={:.5}  Overall={overall_mae:.5}",
            per_action_mae[0], per_action_mae[1], per_action_mae[2]
        );
        println!(
            "  MSE={mse:.5}  Best-action matches={}/{}  Accuracy={:.1}%",
            self.correct_best,
            self.contexts,
            accuracy * 100.0
        );
    }
}

fn is_training_context(context: f32) -> bool {
    TRAIN_CONTEXTS.contains(&context)
}

fn true_probabilities(knowledge_score: f32) -> [f32; 3] {
    [
        0.3 + 0.6 * knowledge_score,
        0.6,
        0.9 - 0.7 * knowledge_score,
    ]
}

fn best_action(values: &[f32; 3]) -> usize {
    (1..ACTIONS.len()).fold(0, |best, candidate| {
        if values[candidate] > values[best] {
            candidate
        } else {
            best
        }
    })
}

fn linear_prediction(weights: &[[f32; 2]; 3], knowledge_score: f32) -> [f32; 3] {
    let features = [knowledge_score, 1.0];
    std::array::from_fn(|action| {
        weights[action][0] * features[0] + weights[action][1] * features[1]
    })
}

fn main() {
    // Both experiments see the same repeating sequence of training contexts.
    // The reward RNGs use the same seed so this remains a controlled demo.
    let mut baseline_rng = Rng::new(SEED);
    let mut per_context: [BanditStats; TRAIN_CONTEXTS.len()] =
        std::array::from_fn(|_| BanditStats::new(ACTIONS.len()));
    let mut baseline_reward = 0_u32;

    for episode in 0..EPISODES {
        let context_index = episode % TRAIN_CONTEXTS.len();
        let knowledge_score = TRAIN_CONTEXTS[context_index];
        let stats = &mut per_context[context_index];

        let action = if baseline_rng.uniform() < EXPLORE_RATE {
            (baseline_rng.uniform() * ACTIONS.len() as f32) as usize
        } else {
            stats.best_arm()
        };

        let reward =
            u32::from(baseline_rng.uniform() < true_probabilities(knowledge_score)[action]);
        baseline_reward += reward;
        stats.update(action, reward as f32);
    }

    println!("=== Lab 4A: Memorized Context Baseline ===");
    println!("Training contexts: 0.00 0.25 0.75 1.00");
    println!("Unseen context: 0.50");
    println!("Training episodes: {EPISODES}  Reward: {baseline_reward}/{EPISODES}");
    println!("Per-context learned Q-values:");
    println!("Context  LLM      RAG      SQL");
    for (context, stats) in TRAIN_CONTEXTS.iter().zip(&per_context) {
        println!(
            "{context:.2}     {:.4}   {:.4}   {:.4}",
            stats.q_value(0),
            stats.q_value(1),
            stats.q_value(2),
        );
    }
    println!("\nQuery context 0.50:");
    println!("No learned statistics for this exact context. The baseline does not interpolate.\n");

    // One independent linear estimate per action: Q(x, a) = w[a] · [x, 1].
    // This model is intentionally local to the example: the repository's
    // existing ContextualBandit is tied to Bomberman action/context types.
    let mut generalizer_rng = Rng::new(SEED);
    let mut weights = [[0.0_f32; 2]; 3];
    let mut generalizer_reward = 0_u32;

    for episode in 0..EPISODES {
        let knowledge_score = TRAIN_CONTEXTS[episode % TRAIN_CONTEXTS.len()];
        let prediction = linear_prediction(&weights, knowledge_score);
        let action = if generalizer_rng.uniform() < EXPLORE_RATE {
            (generalizer_rng.uniform() * ACTIONS.len() as f32) as usize
        } else {
            best_action(&prediction)
        };

        let reward =
            u32::from(generalizer_rng.uniform() < true_probabilities(knowledge_score)[action]);
        generalizer_reward += reward;

        let features = [knowledge_score, 1.0];
        let error = prediction[action] - reward as f32;
        weights[action][0] -= LEARNING_RATE * error * features[0];
        weights[action][1] -= LEARNING_RATE * error * features[1];
    }

    println!("=== Lab 4B: Generalizing Context Model ===");
    println!("Training contexts: 0.00 0.25 0.75 1.00");
    println!("Training episodes: {EPISODES}  Reward: {generalizer_reward}/{EPISODES}");
    println!("Feature vector: [knowledge_score, 1.0]");
    println!("\nContext  True LLM Pred LLM True RAG Pred RAG True SQL Pred SQL Best");

    let mut squared_error = 0.0_f32;
    let mut values_count = 0_u32;
    for context in EVAL_CONTEXTS {
        let truth = true_probabilities(context);
        let predicted = linear_prediction(&weights, context);
        for action in 0..ACTIONS.len() {
            squared_error += (predicted[action] - truth[action]).powi(2);
            values_count += 1;
        }
        let marker = if context == 0.5 { "UNSEEN" } else { "" };
        println!(
            "{context:.2}     {:.2}      {:.3}    {:.2}      {:.3}    {:.2}      {:.3}    {} {marker}",
            truth[0], predicted[0], truth[1], predicted[1], truth[2], predicted[2],
            ACTIONS[best_action(&predicted)],
        );
    }

    let mse = squared_error / values_count as f32;
    let unseen_prediction = linear_prediction(&weights, 0.5);
    let unseen_truth = true_probabilities(0.5);
    println!("\nMean squared prediction error (all evaluation contexts/actions): {mse:.5}");
    println!(
        "Best predicted action at 0.50: {}",
        ACTIONS[best_action(&unseen_prediction)]
    );
    println!(
        "True best action at 0.50: {}",
        ACTIONS[best_action(&unseen_truth)]
    );
    if unseen_truth[0] == unseen_truth[1] {
        println!("At 0.50, LLM and RAG tie at {:.2}.", unseen_truth[0]);
    }
    println!("\nMemorization learns estimates at seen contexts; the linear model estimates the unseen one.");

    println!("\n=== Lab 4B+: Generalization Sweep ===");
    println!("Evaluation only; model weights are unchanged after Lab 4B training.");
    println!("Context | Seen?  | True LLM | Pred LLM | Error | True RAG | Pred RAG | Error | True SQL | Pred SQL | Error | Pred Best | True Best");

    let mut all_metrics = EvaluationMetrics::default();
    let mut train_metrics = EvaluationMetrics::default();
    let mut unseen_metrics = EvaluationMetrics::default();
    let mut llm_predictions = Vec::with_capacity(SWEEP_CONTEXTS.len());
    let mut sql_predictions = Vec::with_capacity(SWEEP_CONTEXTS.len());
    let mut rag_predictions = Vec::with_capacity(SWEEP_CONTEXTS.len());

    for context in SWEEP_CONTEXTS {
        let truth = true_probabilities(context);
        let predicted = linear_prediction(&weights, context);
        let seen = is_training_context(context);
        let metrics = if seen {
            &mut train_metrics
        } else {
            &mut unseen_metrics
        };
        metrics.record(truth, predicted);
        all_metrics.record(truth, predicted);

        let errors =
            std::array::from_fn::<_, 3, _>(|action| (predicted[action] - truth[action]).abs());
        println!(
            "{context:.2}   | {:<6} | {:.2}     | {:.3}    | {:.3} | {:.2}     | {:.3}    | {:.3} | {:.2}     | {:.3}    | {:.3} | {:<9} | {}",
            if seen { "TRAIN" } else { "UNSEEN" },
            truth[0], predicted[0], errors[0],
            truth[1], predicted[1], errors[1],
            truth[2], predicted[2], errors[2],
            ACTIONS[best_action(&predicted)], ACTIONS[best_action(&truth)],
        );
        llm_predictions.push(predicted[0]);
        rag_predictions.push(predicted[1]);
        sql_predictions.push(predicted[2]);
    }

    println!("\nSummary metrics:");
    all_metrics.print("All evaluation contexts");
    train_metrics.print("TRAIN contexts only");
    unseen_metrics.print("UNSEEN contexts only");

    let llm_non_decreasing = llm_predictions
        .windows(2)
        .filter(|pair| pair[1] >= pair[0])
        .count();
    let sql_non_increasing = sql_predictions
        .windows(2)
        .filter(|pair| pair[1] <= pair[0])
        .count();
    let rag_min = rag_predictions
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    let rag_max = rag_predictions
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let rag_mae_from_target = rag_predictions
        .iter()
        .map(|value| (value - 0.6).abs())
        .sum::<f32>()
        / rag_predictions.len() as f32;

    println!("\nTrend sanity checks:");
    println!(
        "Predicted LLM generally increases: {} ({llm_non_decreasing}/{} adjacent steps non-decreasing; endpoints {:.3} → {:.3}).",
        if llm_predictions.last().unwrap() > &llm_predictions[0] { "yes" } else { "no" },
        llm_predictions.len() - 1,
        llm_predictions[0],
        llm_predictions.last().unwrap(),
    );
    println!(
        "Predicted SQL generally decreases: {} ({sql_non_increasing}/{} adjacent steps non-increasing; endpoints {:.3} → {:.3}).",
        if sql_predictions.last().unwrap() < &sql_predictions[0] { "yes" } else { "no" },
        sql_predictions.len() - 1,
        sql_predictions[0],
        sql_predictions.last().unwrap(),
    );
    println!(
        "Predicted RAG relatively stable around 0.60: {} (range {:.3}–{:.3}; mean absolute distance {:.3}; criterion: range ≤ 0.10 and mean distance ≤ 0.05).",
        if rag_max - rag_min <= 0.10 && rag_mae_from_target <= 0.05 { "yes" } else { "no" },
        rag_min,
        rag_max,
        rag_mae_from_target,
    );
}
