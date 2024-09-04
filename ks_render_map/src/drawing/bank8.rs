use std::ops::RangeInclusive;

use anyhow::Result;
use rand::{thread_rng, Rng};

use super::{
    draw_object, draw_object_with_params, Cursor, DrawContext
};

#[inline]
pub fn draw_bank_8_object(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    match curs.proxy_id.0.1 {
        10 => draw_with_random_offset(ctx, curs, -6..=6),
        15 => draw_with_random_offset(ctx, curs, -12..=12),
        _ => draw_object(ctx, curs.i, curs.actual_id),
    }
}

fn draw_with_random_offset(ctx: &mut DrawContext, curs: Cursor, range: RangeInclusive<i64>) -> Result<()> {
    let mut rng = thread_rng();
    let offset_x = rng.gen_range(range.clone());
    let offset_y = rng.gen_range(range);

    let mut draw_params = ctx.gfx.object_def(&curs.actual_id)
        .map_or_else(Default::default, |def| def.draw_params.clone());
    draw_params.offset = Some((offset_x, offset_y));

    draw_object_with_params(ctx, curs.i, curs.actual_id, &draw_params)
}
