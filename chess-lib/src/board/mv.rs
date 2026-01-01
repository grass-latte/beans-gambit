use crate::board::PieceKind;
use crate::board::square::Square;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub source: Square,
    pub destination: Square,
    pub promotion: Option<PieceKind>,
}

impl Move {
    /// Parse a move from UCI move notation, e.g. "e7e8q"
    pub fn from_uci(s: &str) -> Option<Self> {
        if s.len() != 4 && s.len() != 5 {
            return None;
        }

        Some(Self {
            source: Square::from_name(&s[0..2])?,
            destination: Square::from_name(&s[2..4])?,
            promotion: if let Some(c) = s.chars().nth(4) {
                Some(PieceKind::from_char(c)?)
            } else {
                None
            },
        })
    }

    /// Convert a move to UCI move notation, e.g. "e7e8q".
    pub fn as_uci(&self) -> String {
        if let Some(promotion) = self.promotion {
            format!(
                "{}{}{}",
                self.source.name(),
                self.destination.name(),
                promotion.as_char()
            )
        } else {
            format!("{}{}", self.source.name(), self.destination.name(),)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_from_uci() {
        assert_eq!(
            Move::from_uci("e7e8"),
            Some(Move {
                source: Square::E7,
                destination: Square::E8,
                promotion: None,
            }),
        );
        assert_eq!(
            Move::from_uci("e7e8q"),
            Some(Move {
                source: Square::E7,
                destination: Square::E8,
                promotion: Some(PieceKind::Queen),
            }),
        );
        assert_eq!(Move::from_uci("lala"), None);
        assert_eq!(Move::from_uci("lalal"), None);
        assert_eq!(Move::from_uci("lalala"), None);
        assert_eq!(Move::from_uci("e7e8l"), None);
    }

    #[test]
    fn move_as_uci() {
        assert_eq!(
            Move {
                source: Square::E7,
                destination: Square::E8,
                promotion: None,
            }
            .as_uci(),
            "e7e8"
        );
        assert_eq!(
            Move {
                source: Square::E7,
                destination: Square::E8,
                promotion: Some(PieceKind::Queen)
            }
            .as_uci(),
            "e7e8q"
        );
    }
}
