use std::cmp::Ordering;
use std::ops::Neg;

#[derive(Eq, Copy, Clone)]
pub enum Score {
    Checkmate((bool, u32)),
    Draw,
    Playing(i32),
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Score::Playing(0), Score::Draw) | (Score::Draw, Score::Playing(0)) => true,
            (Score::Playing(a), Score::Playing(b)) => a == b,
            (Score::Checkmate((b1, n1)), Score::Checkmate((b2, n2))) => b1 == b2 && n1 == n2,
            (Score::Draw, Score::Draw) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other { return Some(Ordering::Equal) }
        match (self, other) {
            (Score::Checkmate((b1, n1)), Score::Checkmate((b2, n2))) => {
                if b1 == b2 {
                    if *b1 { Some(n1.cmp(n2).reverse()) } else { Some(n1.cmp(n2)) }
                }
                else if *b1 {
                    Some(Ordering::Greater)
                }
                else {
                    Some(Ordering::Less)
                }
            },

            (Score::Checkmate((true, _)), _) => Some(Ordering::Greater),
            (_, Score::Checkmate((true, _))) => Some(Ordering::Less),

            (Score::Checkmate((false, _)), _) => Some(Ordering::Less),
            (_, Score::Checkmate((false, _))) => Some(Ordering::Greater),

            (Score::Playing(a), Score::Playing(b)) => Some(a.cmp(b)),

            (Score::Draw, Score::Playing(x)) => Some(x.cmp(&0).reverse()),
            (Score::Playing(x), Score::Draw) => Some(0.cmp(x).reverse()),

            _ => None,
        }
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(o) = self.partial_cmp(other) {
            return o;
        }
        Ordering::Equal
    }
}

impl Neg for Score {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Score::Checkmate((b, n)) => Score::Checkmate((!b, n)),
            Score::Playing(n) => Score::Playing(-n),
            Score::Draw => Score::Draw,
        }
    }
}


