use image::{imageops, GenericImage, GenericImageView, Rgba};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub enum BlendMode {
    #[default]
    Over,
    Add,
    Sub,
}

/// Adapted from image crate
/// Source: https://github.com/image-rs/image/blob/285496d4fab063645dc4ffafd7ccfa3e06c35052/src/imageops/mod.rs#L219
pub fn overlay<I, J>(bottom: &mut I, top: &J, x: i64, y: i64, blend_mode: BlendMode)
where
    I: GenericImage,
    J: GenericImageView<Pixel = I::Pixel>,
    I::Pixel: PixelBlendExt,
{
    if matches!(blend_mode, BlendMode::Over) {
        return imageops::overlay(bottom, top, x, y);
    }

    let bottom_dims = bottom.dimensions();
    let top_dims = top.dimensions();

    // Crop our top image if we're going out of bounds
    let (origin_bottom_x, origin_bottom_y, origin_top_x, origin_top_y, range_width, range_height) =
        overlay_bounds_ext(bottom_dims, top_dims, x, y);

    for y in 0..range_height {
        for x in 0..range_width {
            let p = top.get_pixel(origin_top_x + x, origin_top_y + y);
            let mut bottom_pixel = bottom.get_pixel(origin_bottom_x + x, origin_bottom_y + y);
            bottom_pixel.blend_with_mode(&p, blend_mode);

            bottom.put_pixel(origin_bottom_x + x, origin_bottom_y + y, bottom_pixel);
        }
    }
}

pub fn overlay_with_alpha<I, J>(bottom: &mut I, top: &J, x: i64, y: i64, blend_mode: BlendMode, alpha: f32)
where
    I: GenericImage,
    J: GenericImageView<Pixel = I::Pixel>,
    I::Pixel: PixelBlendExt,
{
    if alpha <= 0.0 {
        return;
    }
    else if alpha >= 1.0 {
        return overlay(bottom, top, x, y, blend_mode);
    }

    let bottom_dims = bottom.dimensions();
    let top_dims = top.dimensions();

    // Crop our top image if we're going out of bounds
    let (origin_bottom_x, origin_bottom_y, origin_top_x, origin_top_y, range_width, range_height) =
        overlay_bounds_ext(bottom_dims, top_dims, x, y);

    for y in 0..range_height {
        for x in 0..range_width {
            let mut p = top.get_pixel(origin_top_x + x, origin_top_y + y);
            p.mul_alpha(alpha);

            let mut bottom_pixel = bottom.get_pixel(origin_bottom_x + x, origin_bottom_y + y);
            bottom_pixel.blend_with_mode(&p, blend_mode);

            bottom.put_pixel(origin_bottom_x + x, origin_bottom_y + y, bottom_pixel);
        }
    }
}

/// Private function from image crate
/// Source: https://github.com/image-rs/image/blob/285496d4fab063645dc4ffafd7ccfa3e06c35052/src/imageops/mod.rs#L170
fn overlay_bounds_ext(
    (bottom_width, bottom_height): (u32, u32),
    (top_width, top_height): (u32, u32),
    x: i64,
    y: i64,
) -> (u32, u32, u32, u32, u32, u32) {
    // Return a predictable value if the two images don't overlap at all.
    if x > i64::from(bottom_width)
        || y > i64::from(bottom_height)
        || x.saturating_add(i64::from(top_width)) <= 0
        || y.saturating_add(i64::from(top_height)) <= 0
    {
        return (0, 0, 0, 0, 0, 0);
    }

    // Find the maximum x and y coordinates in terms of the bottom image.
    let max_x = x.saturating_add(i64::from(top_width));
    let max_y = y.saturating_add(i64::from(top_height));

    // Clip the origin and maximum coordinates to the bounds of the bottom image.
    // Casting to a u32 is safe because both 0 and `bottom_{width,height}` fit
    // into 32-bits.
    let max_inbounds_x = max_x.clamp(0, i64::from(bottom_width)) as u32;
    let max_inbounds_y = max_y.clamp(0, i64::from(bottom_height)) as u32;
    let origin_bottom_x = x.clamp(0, i64::from(bottom_width)) as u32;
    let origin_bottom_y = y.clamp(0, i64::from(bottom_height)) as u32;

    // The range is the difference between the maximum inbounds coordinates and
    // the clipped origin. Unchecked subtraction is safe here because both are
    // always positive and `max_inbounds_{x,y}` >= `origin_{x,y}` due to
    // `top_{width,height}` being >= 0.
    let x_range = max_inbounds_x - origin_bottom_x;
    let y_range = max_inbounds_y - origin_bottom_y;

    // If x (or y) is negative, then the origin of the top image is shifted by -x (or -y).
    let origin_top_x = x.saturating_mul(-1).clamp(0, i64::from(top_width)) as u32;
    let origin_top_y = y.saturating_mul(-1).clamp(0, i64::from(top_height)) as u32;

    (
        origin_bottom_x,
        origin_bottom_y,
        origin_top_x,
        origin_top_y,
        x_range,
        y_range,
    )
}

pub trait PixelBlendExt {
    fn mul_alpha(&mut self, alpha: f32);
    fn blend_with_mode(&mut self, fore: &Self, mode: BlendMode);
}

impl PixelBlendExt for Rgba<u8> {
    fn mul_alpha(&mut self, alpha: f32) {
        let new_alpha = self.0[3] as f32 * alpha;
        self.0[3] = new_alpha.clamp(0.0, 255.0) as u8;
    }

    fn blend_with_mode(&mut self, fore: &Self, mode: BlendMode) {
        // Convert to 0.0-1.0 f32
        let mut self_r = self.0[0] as f32 / 255.0f32;
        let mut self_g = self.0[1] as f32 / 255.0f32;
        let mut self_b = self.0[2] as f32 / 255.0f32;
        let mut self_a = self.0[3] as f32 / 255.0f32;

        let mut fore_r = fore.0[0] as f32 / 255.0f32;
        let mut fore_g = fore.0[1] as f32 / 255.0f32;
        let mut fore_b = fore.0[2] as f32 / 255.0f32;
        let fore_a = fore.0[3] as f32 / 255.0f32;

        // Premultiply alpha
        self_r *= self_a;
        self_g *= self_a;
        self_b *= self_a;

        fore_r *= fore_a;
        fore_g *= fore_a;
        fore_b *= fore_a;

        // Blend
        match mode {
            BlendMode::Over => {
                let fore_t = 1.0f32 - fore_a;
                self_r = fore_r + self_r * fore_t;
                self_g = fore_g + self_g * fore_t;
                self_b = fore_b + self_b * fore_t;
                self_a = fore_a + self_a * fore_t;
            },
            BlendMode::Add => {
                self_r += fore_r;
                self_g += fore_g;
                self_b += fore_b;
                // Alpha regulates how much of the foreground color is added
                // But the final alpha is unchanged
            },
            BlendMode::Sub => {
                self_r -= fore_r;
                self_g -= fore_g;
                self_b -= fore_b;
                // Alpha regulates how much of the foreground color is subtracted
                // But the final alpha is unchanged
            },
        }

        // Clamp
        self_r = self_r.clamp(0.0, 1.0);
        self_g = self_g.clamp(0.0, 1.0);
        self_b = self_b.clamp(0.0, 1.0);
        self_a = self_a.clamp(0.0, 1.0);

        // "Unmultiply" alpha
        self_r /= self_a;
        self_g /= self_a;
        self_b /= self_a;

        // Convert back to 0-255 u8
        self.0[0] = ((self_r / self_a) * 255.0f32) as u8;
        self.0[1] = ((self_g / self_a) * 255.0f32) as u8;
        self.0[2] = ((self_b / self_a) * 255.0f32) as u8;
        self.0[3] = (self_a * 255.0f32) as u8;
    }
}
