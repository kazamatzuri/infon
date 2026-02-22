// Elo rating calculation system.
//
// Per-version Elo for 1v1 and 2v2 matches.
// FFA uses placement-based scoring (no Elo).

use serde::{Deserialize, Serialize};

pub const STARTING_ELO: i32 = 1500;
pub const RATING_FLOOR: i32 = 100;

// K-factor thresholds
const K_PROVISIONAL_GAMES: i32 = 30;
const K_ELITE_RATING: i32 = 2400;

const K_PROVISIONAL: f64 = 40.0;
const K_ESTABLISHED: f64 = 20.0;
const K_ELITE: f64 = 10.0;

/// Match outcome from perspective of one player.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Outcome {
    Win,
    Draw,
    Loss,
}

impl Outcome {
    pub fn score(self) -> f64 {
        match self {
            Outcome::Win => 1.0,
            Outcome::Draw => 0.5,
            Outcome::Loss => 0.0,
        }
    }
}

/// Get the K-factor for a player based on their games played and current rating.
fn k_factor(games_played: i32, rating: i32) -> f64 {
    if games_played < K_PROVISIONAL_GAMES {
        K_PROVISIONAL
    } else if rating >= K_ELITE_RATING {
        K_ELITE
    } else {
        K_ESTABLISHED
    }
}

/// Calculate expected score for player A against player B.
pub fn expected_score(rating_a: i32, rating_b: i32) -> f64 {
    1.0 / (1.0 + 10.0_f64.powf((rating_b - rating_a) as f64 / 400.0))
}

/// Calculate new rating after a 1v1 match.
pub fn calculate_new_rating(
    rating: i32,
    opponent_rating: i32,
    outcome: Outcome,
    games_played: i32,
) -> i32 {
    let k = k_factor(games_played, rating);
    let expected = expected_score(rating, opponent_rating);
    let new_rating = rating as f64 + k * (outcome.score() - expected);
    (new_rating.round() as i32).max(RATING_FLOOR)
}

/// Soft reset for new version: (parent_elo + 1500) / 2
pub fn soft_reset_elo(parent_elo: i32) -> i32 {
    (parent_elo + STARTING_ELO) / 2
}

/// Calculate FFA placement points. 1st place = n_players, last = 1.
pub fn ffa_placement_points(placement: i32, n_players: i32) -> i32 {
    n_players - placement + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_score_equal_ratings() {
        let e = expected_score(1500, 1500);
        assert!((e - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_expected_score_higher_rated() {
        let e = expected_score(1800, 1500);
        assert!(e > 0.8);
        assert!(e < 1.0);
    }

    #[test]
    fn test_expected_score_lower_rated() {
        let e = expected_score(1200, 1500);
        assert!(e < 0.2);
        assert!(e > 0.0);
    }

    #[test]
    fn test_new_rating_win_equal() {
        let new = calculate_new_rating(1500, 1500, Outcome::Win, 0);
        assert_eq!(new, 1520); // K=40, expected=0.5, 1500 + 40*(1-0.5) = 1520
    }

    #[test]
    fn test_new_rating_loss_equal() {
        let new = calculate_new_rating(1500, 1500, Outcome::Loss, 0);
        assert_eq!(new, 1480);
    }

    #[test]
    fn test_new_rating_draw_equal() {
        let new = calculate_new_rating(1500, 1500, Outcome::Draw, 0);
        assert_eq!(new, 1500);
    }

    #[test]
    fn test_k_factor_provisional() {
        // Win with fewer than 30 games: K=40
        let new = calculate_new_rating(1500, 1500, Outcome::Win, 10);
        assert_eq!(new, 1520);
    }

    #[test]
    fn test_k_factor_established() {
        // Win with 30+ games: K=20
        let new = calculate_new_rating(1500, 1500, Outcome::Win, 50);
        assert_eq!(new, 1510);
    }

    #[test]
    fn test_k_factor_elite() {
        // Win with high rating: K=10
        let new = calculate_new_rating(2500, 2500, Outcome::Win, 100);
        assert_eq!(new, 2505);
    }

    #[test]
    fn test_rating_floor() {
        // Rating shouldn't go below floor even with huge loss
        let new = calculate_new_rating(110, 2000, Outcome::Loss, 0);
        assert!(new >= RATING_FLOOR);
    }

    #[test]
    fn test_soft_reset() {
        assert_eq!(soft_reset_elo(1500), 1500); // (1500+1500)/2
        assert_eq!(soft_reset_elo(2000), 1750); // (2000+1500)/2
        assert_eq!(soft_reset_elo(1000), 1250); // (1000+1500)/2
    }

    #[test]
    fn test_ffa_placement_points() {
        assert_eq!(ffa_placement_points(1, 5), 5); // 1st of 5 = 5 pts
        assert_eq!(ffa_placement_points(5, 5), 1); // last of 5 = 1 pt
        assert_eq!(ffa_placement_points(3, 5), 3); // 3rd of 5 = 3 pts
    }

    #[test]
    fn test_zero_sum_1v1() {
        let r_a = 1500;
        let r_b = 1500;
        let new_a = calculate_new_rating(r_a, r_b, Outcome::Win, 0);
        let new_b = calculate_new_rating(r_b, r_a, Outcome::Loss, 0);
        // Zero-sum: gains + losses = 0
        assert_eq!((new_a - r_a) + (new_b - r_b), 0);
    }
}
