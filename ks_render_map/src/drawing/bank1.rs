use anyhow::Result;

use super::{
    draw_object, Cursor, DrawContext
};

#[inline]
pub fn draw_bank_1_object(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    match curs.tile.1 {
        5 | 10 | 12 | 22 => draw_with_glow(ctx, curs),
        _ => draw_object(ctx, curs),
    }
}

fn draw_with_glow(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    draw_object(ctx, curs.with_variant("Glow"))?;
    draw_object(ctx, curs)?;

    Ok(())
}
