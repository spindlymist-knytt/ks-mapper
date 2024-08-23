use anyhow::Result;

use super::{
    draw_object, Cursor, DrawContext
};

#[inline]
pub fn draw_bank_0_object(ctx: &mut DrawContext, curs: Cursor) -> Result<()> {
    match curs.tile.1 {
        14 => draw_shift(ctx, curs, "ShiftVisible(A)", "ShiftType(A)"),
        15 => draw_shift(ctx, curs, "ShiftVisible(B)", "ShiftType(B)"),
        16 => draw_shift(ctx, curs, "ShiftVisible(C)", "ShiftType(C)"),
        32 => draw_shift(ctx, curs, "TrigVisible(A)", "TrigType(A)"),
        33 => draw_shift(ctx, curs, "TrigVisible(B)", "TrigType(B)"),
        34 => draw_shift(ctx, curs, "TrigVisible(C)", "TrigType(C)"),
        _ => draw_object(ctx, curs),
    }
}

fn draw_shift(ctx: &mut DrawContext, curs: Cursor, vis_prop: &str, type_prop: &str) -> Result<()> {
    let shift_visible = !ctx.ini_section
        .as_ref()
        .and_then(|section| section.get(vis_prop))
        .unwrap_or("True")
        .eq_ignore_ascii_case("False");

    if !shift_visible {
        return Ok(());
    }

    let shift_type = match ctx.ini_section
        .as_ref()
        .and_then(|section| section.get(type_prop))
        .unwrap_or("0")
    {
        "0" => "Spot",
        "1" => "Floor",
        "2" => "Circle",
        "3" => "Square",
        _ => "Spot",
    };

    draw_object(ctx, curs.into_variant(shift_type))
}
