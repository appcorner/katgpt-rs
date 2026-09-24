//! Lab 3A: a standard bandit learning from rewards without seeing context.
//!
//! Run with `cargo run --example lab3_context_baseline`.

use katgpt_rs::pruners::BanditStats;
use katgpt_rs::types::Rng;

const ARM_NAMES: [&str; 3] = ["LLM", "RAG", "SQL"];
const CONTEXT_NAMES: [&str; 2] = ["Knowledge", "Customer"];
const REWARD_PROBS: [[f32; 3]; 2] = [
    [0.9, 0.6, 0.2], // Knowledge
    [0.3, 0.6, 0.9], // Customer
];
const EPISODES: usize = 5_000;
const SEED: u64 = 3_202_609_24;

fn main() {
    let mut rng = Rng::new(SEED);
    let mut bandit = BanditStats::new(ARM_NAMES.len());
    let mut actions_by_context = [[0_u32; 3]; 2];
    let mut rewards_by_context = [0_u32; 2];
    let mut total_reward = 0_u32;

    for _ in 0..EPISODES {
        // The environment samples context, but only the reward generator reads it.
        let context = usize::from(rng.uniform() >= 0.5);

        // Thompson Sampling receives only global arm statistics and the RNG.
        let mut arm = 0;
        let mut best_sample = f32::NEG_INFINITY;
        for candidate in 0..ARM_NAMES.len() {
            let sample = bandit.thompson_sample(candidate, &mut rng);
            if sample > best_sample {
                arm = candidate;
                best_sample = sample;
            }
        }

        let reward = u32::from(rng.uniform() < REWARD_PROBS[context][arm]);
        actions_by_context[context][arm] += 1;
        rewards_by_context[context] += reward;
        total_reward += reward;
        bandit.update(arm, reward as f32);
    }

    println!("=== Lab 3A: No-Context Baseline ===");
    println!("Episodes: {EPISODES}  Seed: {SEED}");
    println!("The environment has context, but the bandit cannot see it.\n");

    println!("Theoretical hidden-context reward probabilities");
    println!("Context      LLM    RAG    SQL");
    for (context, probs) in CONTEXT_NAMES.iter().zip(REWARD_PROBS) {
        println!("{context:<12} {:.2}   {:.2}   {:.2}", probs[0], probs[1], probs[2]);
    }
    println!();

    println!("Global Bandit Statistics");
    println!("Arm          Q-value  Visits");
    for (arm, name) in ARM_NAMES.iter().enumerate() {
        println!("{name:<12} {:.4}   {}", bandit.q_value(arm), bandit.visit_count(arm));
    }
    println!();

    println!("Performance by Actual Context");
    println!("Context      LLM    RAG    SQL    Reward");
    for context in 0..CONTEXT_NAMES.len() {
        println!(
            "{:<12} {:<6} {:<6} {:<6} {}/{}",
            CONTEXT_NAMES[context],
            actions_by_context[context][0],
            actions_by_context[context][1],
            actions_by_context[context][2],
            rewards_by_context[context],
            actions_by_context[context].iter().sum::<u32>(),
        );
    }

    let average_reward = total_reward as f32 / EPISODES as f32;
    println!("\nBest global arm: {}", ARM_NAMES[bandit.best_arm()]);
    println!("Total reward: {total_reward}");
    println!("Average reward: {average_reward:.4}");

    // Experiment B: context selects its own standard bandit statistics.
    // Context is not encoded as a feature and no specialized algorithm is used.
    let mut rng = Rng::new(SEED);
    let mut context_bandits = [
        BanditStats::new(ARM_NAMES.len()),
        BanditStats::new(ARM_NAMES.len()),
    ];
    let mut b_actions = [[0_u32; 3]; 2];
    let mut b_rewards = [0_u32; 2];
    let mut b_total_reward = 0_u32;

    for _ in 0..EPISODES {
        let context = usize::from(rng.uniform() >= 0.5);
        let stats = &mut context_bandits[context];

        let mut arm = 0;
        let mut best_sample = f32::NEG_INFINITY;
        for candidate in 0..ARM_NAMES.len() {
            let sample = stats.thompson_sample(candidate, &mut rng);
            if sample > best_sample {
                arm = candidate;
                best_sample = sample;
            }
        }

        let reward = u32::from(rng.uniform() < REWARD_PROBS[context][arm]);
        b_actions[context][arm] += 1;
        b_rewards[context] += reward;
        b_total_reward += reward;
        stats.update(arm, reward as f32);
    }

    println!("\n=== Lab 3B: Context-Aware Baseline ===");
    println!("Episodes: {EPISODES}  Seed: {SEED}");
    println!("The bandit sees the context and uses separate statistics for each one.\n");
    println!("Context-specific Bandit Statistics");
    for context in 0..CONTEXT_NAMES.len() {
        let stats = &context_bandits[context];
        println!("\n{}", CONTEXT_NAMES[context]);
        println!("Arm          Q-value  Visits  Actions");
        for (arm, name) in ARM_NAMES.iter().enumerate() {
            println!(
                "{name:<12} {:.4}   {:<6} {}",
                stats.q_value(arm),
                stats.visit_count(arm),
                b_actions[context][arm],
            );
        }
        println!(
            "Best action: {}  Reward: {}/{}",
            ARM_NAMES[stats.best_arm()],
            b_rewards[context],
            b_actions[context].iter().sum::<u32>(),
        );
    }
    println!("\nTotal reward: {b_total_reward}");
    println!("Average reward: {:.4}", b_total_reward as f32 / EPISODES as f32);
}
