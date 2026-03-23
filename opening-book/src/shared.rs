use chess_lib::board::{PieceKind, Square};

#[derive(Debug, Clone)]
pub struct SerialisedBookMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
    pub weight: u16,
}

pub const SERIALISED_MOVE_SIZE: usize = 6;

#[allow(dead_code)]
pub fn serialise_book_move(mv: SerialisedBookMove) -> [u8; SERIALISED_MOVE_SIZE] {
    let mut output = [0; SERIALISED_MOVE_SIZE];
    output[0] = mv.from.as_u8();
    output[1] = mv.to.as_u8();
    output[2] = if mv.promotion.is_some() { 1 } else { 0 };
    if let Some(p) = mv.promotion {
        output[3] = p.as_u8();
    }
    output[4] = (mv.weight >> 8) as u8;
    output[5] = ((mv.weight << 8) >> 8) as u8;

    output
}

#[allow(dead_code)]
pub fn deserialise_book_move(bytes: &[u8]) -> SerialisedBookMove {
    SerialisedBookMove {
        from: Square::from_u8(bytes[0]).unwrap(),
        to: Square::from_u8(bytes[1]).unwrap(),
        promotion: if bytes[2] == 0 {
            None
        } else {
            Some(PieceKind::from_u8(bytes[3]).unwrap())
        },
        weight: ((bytes[4] as u16) << 8) + (bytes[5] as u16),
    }
}
