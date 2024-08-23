use anyhow::Result;
use rand::{thread_rng, seq::SliceRandom};

use super::{
    draw_object, Cursor, DrawContext
};

#[inline]
pub fn draw_bank_2_object(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    match curs.tile.1 {
        18 | 19 => draw_elemental(ctx, curs),
        _ => draw_object(ctx, curs),
    }
}

pub fn draw_elemental(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    let mut rng = thread_rng();
    let variant = &["A", "B", "C", "D"]
        .choose(&mut rng)
        .unwrap();

    draw_object(ctx, curs.into_variant(variant))
}
