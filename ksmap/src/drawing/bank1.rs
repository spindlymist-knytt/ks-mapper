use anyhow::Result;

use super::{
    draw_object, Cursor, DrawContext
};

#[inline]
pub fn draw_bank_1_object(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    match curs.proxy_id.0.1 {
        5 | 10 | 12 | 22 => draw_with_glow(ctx, curs),
        _ => draw_object(ctx, curs.i, curs.actual_id),
    }
}

fn draw_with_glow(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    draw_object(ctx, curs.i, curs.proxy_id.with_variant("Glow"))?;
    draw_object(ctx, curs.i, curs.actual_id)?;

    Ok(())
}
