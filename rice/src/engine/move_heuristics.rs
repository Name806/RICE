use crate::move_generation::EncodedMove;
use crate::engine::eval_data::EvalData;

pub struct MoveSorter;

impl MoveSorter {
    pub fn sort(moves: Vec<EncodedMove>) -> Vec<EncodedMove> {
        let mut scores = vec![0; moves.len()];
        for (i, m) in moves.clone().into_iter().enumerate() {
            scores[i] = Self::score_move(m);
        }
        let mut paired: Vec<_> = moves.into_iter().zip(scores).collect();
        paired.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let moves_sorted: Vec<_> = paired.into_iter().map(|(mv, _)| mv).collect();

        moves_sorted
    }

    fn score_move(m: EncodedMove) -> i32 {
        let mut total_score = 0;

        if let Some(captured_piece) = m.capture() {
            let moved_piece = m.piece_moved();
            let capture_value = EvalData::MATERIAL_VALUE[captured_piece as usize];
            let moved_value = EvalData::MOBILITY_VALUE[moved_piece as usize];

            total_score += 10_000 + (capture_value - moved_value);
        }

        if m.castle() || m.promoted_piece().is_some() {
            total_score += 5_000;
        }

        total_score
    }
}
