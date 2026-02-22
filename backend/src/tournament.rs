// Tournament format support: bracket generation for single elimination,
// round robin, and Swiss-style pairings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TournamentFormat {
    SingleElimination,
    RoundRobin,
    Swiss { rounds: usize },
}

impl TournamentFormat {
    /// Parse a format string (from DB) into a TournamentFormat.
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "single_elimination" => Some(Self::SingleElimination),
            "round_robin" => Some(Self::RoundRobin),
            s if s.starts_with("swiss") => {
                // Parse "swiss_N" or just "swiss" (defaults to 3 rounds)
                let rounds = s
                    .strip_prefix("swiss_")
                    .and_then(|n| n.parse::<usize>().ok())
                    .unwrap_or(3);
                Some(Self::Swiss { rounds })
            }
            _ => None,
        }
    }

    /// Serialize to a DB-storable string.
    pub fn to_str_name(&self) -> String {
        match self {
            Self::SingleElimination => "single_elimination".to_string(),
            Self::RoundRobin => "round_robin".to_string(),
            Self::Swiss { rounds } => format!("swiss_{rounds}"),
        }
    }
}

/// Generate single-elimination bracket pairings.
///
/// Participants are paired sequentially: (0 vs 1), (2 vs 3), etc.
/// If the number of participants is odd, the last one gets a bye
/// (not included in any pair for this round).
///
/// Returns Vec of (version_id_a, version_id_b) match pairs.
pub fn generate_single_elimination_bracket(participants: &[i64]) -> Vec<(i64, i64)> {
    let mut pairs = Vec::new();
    let mut i = 0;
    while i + 1 < participants.len() {
        pairs.push((participants[i], participants[i + 1]));
        i += 2;
    }
    pairs
}

/// Generate round-robin pairings (all vs all).
///
/// Every participant plays against every other participant exactly once.
/// Returns Vec of (version_id_a, version_id_b) match pairs.
pub fn generate_round_robin_pairings(participants: &[i64]) -> Vec<(i64, i64)> {
    let mut pairs = Vec::new();
    for i in 0..participants.len() {
        for j in (i + 1)..participants.len() {
            pairs.push((participants[i], participants[j]));
        }
    }
    pairs
}

/// Generate Swiss-style pairings for a given round.
///
/// Participants are sorted by their current standings (score descending),
/// then paired top-down: 1st vs 2nd, 3rd vs 4th, etc.
/// If the number of participants is odd, the lowest-ranked player gets a bye.
///
/// `standings` is a slice of (version_id, score) tuples.
/// Returns Vec of (version_id_a, version_id_b) match pairs.
pub fn generate_swiss_pairings(
    participants: &[i64],
    standings: &[(i64, f64)],
    _round: usize,
) -> Vec<(i64, i64)> {
    // Sort participants by score descending
    let mut sorted: Vec<i64> = participants.to_vec();
    sorted.sort_by(|a, b| {
        let score_a = standings
            .iter()
            .find(|(id, _)| *id == *a)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        let score_b = standings
            .iter()
            .find(|(id, _)| *id == *b)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut pairs = Vec::new();
    let mut i = 0;
    while i + 1 < sorted.len() {
        pairs.push((sorted[i], sorted[i + 1]));
        i += 2;
    }
    pairs
}

/// Compute the total number of rounds needed for a tournament format.
pub fn total_rounds(format: &TournamentFormat, num_participants: usize) -> usize {
    match format {
        TournamentFormat::SingleElimination => {
            // Number of rounds = ceil(log2(n))
            if num_participants <= 1 {
                0
            } else {
                (num_participants as f64).log2().ceil() as usize
            }
        }
        TournamentFormat::RoundRobin => {
            // Round robin is effectively 1 "round" of all pairings
            1
        }
        TournamentFormat::Swiss { rounds } => *rounds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_elimination_4_players() {
        let participants = vec![1, 2, 3, 4];
        let pairs = generate_single_elimination_bracket(&participants);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (1, 2));
        assert_eq!(pairs[1], (3, 4));
    }

    #[test]
    fn test_single_elimination_8_players() {
        let participants = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let pairs = generate_single_elimination_bracket(&participants);
        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0], (10, 20));
        assert_eq!(pairs[1], (30, 40));
        assert_eq!(pairs[2], (50, 60));
        assert_eq!(pairs[3], (70, 80));
    }

    #[test]
    fn test_single_elimination_odd_players() {
        let participants = vec![1, 2, 3];
        let pairs = generate_single_elimination_bracket(&participants);
        // Only 1 pair, player 3 gets a bye
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 2));
    }

    #[test]
    fn test_single_elimination_2_players() {
        let participants = vec![5, 10];
        let pairs = generate_single_elimination_bracket(&participants);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (5, 10));
    }

    #[test]
    fn test_single_elimination_empty() {
        let participants: Vec<i64> = vec![];
        let pairs = generate_single_elimination_bracket(&participants);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_round_robin_4_players() {
        let participants = vec![1, 2, 3, 4];
        let pairs = generate_round_robin_pairings(&participants);
        // C(4,2) = 6 pairs
        assert_eq!(pairs.len(), 6);
        assert!(pairs.contains(&(1, 2)));
        assert!(pairs.contains(&(1, 3)));
        assert!(pairs.contains(&(1, 4)));
        assert!(pairs.contains(&(2, 3)));
        assert!(pairs.contains(&(2, 4)));
        assert!(pairs.contains(&(3, 4)));
    }

    #[test]
    fn test_round_robin_3_players() {
        let participants = vec![10, 20, 30];
        let pairs = generate_round_robin_pairings(&participants);
        // C(3,2) = 3 pairs
        assert_eq!(pairs.len(), 3);
        assert!(pairs.contains(&(10, 20)));
        assert!(pairs.contains(&(10, 30)));
        assert!(pairs.contains(&(20, 30)));
    }

    #[test]
    fn test_round_robin_2_players() {
        let participants = vec![1, 2];
        let pairs = generate_round_robin_pairings(&participants);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 2));
    }

    #[test]
    fn test_round_robin_empty() {
        let participants: Vec<i64> = vec![];
        let pairs = generate_round_robin_pairings(&participants);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_swiss_pairing_by_score() {
        let participants = vec![1, 2, 3, 4];
        // Player 3 has highest score, player 1 has lowest
        let standings = vec![(1, 0.0), (2, 5.0), (3, 10.0), (4, 3.0)];
        let pairs = generate_swiss_pairings(&participants, &standings, 1);
        assert_eq!(pairs.len(), 2);
        // Sorted by score desc: 3(10), 2(5), 4(3), 1(0)
        assert_eq!(pairs[0], (3, 2));
        assert_eq!(pairs[1], (4, 1));
    }

    #[test]
    fn test_swiss_pairing_no_standings() {
        let participants = vec![1, 2, 3, 4];
        let standings: Vec<(i64, f64)> = vec![];
        let pairs = generate_swiss_pairings(&participants, &standings, 0);
        assert_eq!(pairs.len(), 2);
        // All scores default to 0.0, so order is preserved
        assert_eq!(pairs[0], (1, 2));
        assert_eq!(pairs[1], (3, 4));
    }

    #[test]
    fn test_swiss_pairing_odd_players() {
        let participants = vec![1, 2, 3];
        let standings = vec![(1, 5.0), (2, 10.0), (3, 0.0)];
        let pairs = generate_swiss_pairings(&participants, &standings, 1);
        // Sorted: 2(10), 1(5), 3(0) -> only 1 pair, player 3 gets bye
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (2, 1));
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!(
            TournamentFormat::from_str_name("single_elimination"),
            Some(TournamentFormat::SingleElimination)
        );
        assert_eq!(
            TournamentFormat::from_str_name("round_robin"),
            Some(TournamentFormat::RoundRobin)
        );
        assert_eq!(
            TournamentFormat::from_str_name("swiss_5"),
            Some(TournamentFormat::Swiss { rounds: 5 })
        );
        assert_eq!(
            TournamentFormat::from_str_name("swiss"),
            Some(TournamentFormat::Swiss { rounds: 3 })
        );
        assert_eq!(TournamentFormat::from_str_name("unknown"), None);
    }

    #[test]
    fn test_format_to_string() {
        assert_eq!(
            TournamentFormat::SingleElimination.to_str_name(),
            "single_elimination"
        );
        assert_eq!(TournamentFormat::RoundRobin.to_str_name(), "round_robin");
        assert_eq!(
            TournamentFormat::Swiss { rounds: 5 }.to_str_name(),
            "swiss_5"
        );
    }

    #[test]
    fn test_total_rounds() {
        assert_eq!(total_rounds(&TournamentFormat::SingleElimination, 4), 2);
        assert_eq!(total_rounds(&TournamentFormat::SingleElimination, 8), 3);
        assert_eq!(total_rounds(&TournamentFormat::SingleElimination, 1), 0);
        assert_eq!(total_rounds(&TournamentFormat::RoundRobin, 4), 1);
        assert_eq!(total_rounds(&TournamentFormat::Swiss { rounds: 5 }, 10), 5);
    }
}
