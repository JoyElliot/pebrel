//! DPI-specific sidebar textures, shared by tabs, panes and settings.

use std::collections::HashMap;
use std::sync::Arc;

use gpui::RenderImage;
use image::Frame;

fn decode_sidebar_logo(
    logo: crate::display::AiLogo,
    dark: bool,
    target_size: u32,
) -> Option<Arc<RenderImage>> {
    let mut rgba = image::load_from_memory(logo.png(dark)).ok()?.into_rgba8();
    logo.tint_pixels(&mut rgba, if dark { [236, 239, 245] } else { [35, 40, 50] });
    // 直接复用旧壳的 Lanczos3 物理像素预缩放与 alpha 质量中心校正。
    // 先 tint 再缩放，避免 1024px 原图在 GPUI paint 阶段临时压到十几个
    // 逻辑像素时产生灰边、锯齿与非整数 DPI 采样。
    let (prepared, width, height) = crate::display::prepare_ai_logo_texture(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
        target_size,
    );
    let mut rgba = image::RgbaImage::from_raw(width, height, prepared)?;
    // GPUI 的原始帧使用 BGRA；与壁纸解码走同一通道转换。
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Some(Arc::new(RenderImage::new([Frame::new(rgba)])))
}

pub(super) fn sidebar_logo_images(
    target_size: u32,
) -> HashMap<(crate::display::AiLogo, bool), Arc<RenderImage>> {
    use crate::display::AiLogo;

    let mut images = HashMap::new();
    for logo in AiLogo::ALL {
        if logo.shares_theme_texture() {
            if let Some(image) = decode_sidebar_logo(logo, false, target_size) {
                images.insert((logo, false), image.clone());
                images.insert((logo, true), image);
            }
            continue;
        }
        for dark in [false, true] {
            if let Some(image) = decode_sidebar_logo(logo, dark, target_size) {
                images.insert((logo, dark), image);
            }
        }
    }
    images
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::AiLogo;

    #[test]
    fn sidebar_logos_share_only_identical_theme_pixels_at_each_dpi() {
        // 100% and 150% DPI keep separate physical textures. There is no global cache.
        for target in [15, 23] {
            let images = sidebar_logo_images(target);
            assert_eq!(images.len(), AiLogo::ALL.len() * 2);
            let unique = images.values().map(Arc::as_ptr).collect::<std::collections::HashSet<_>>();
            assert_eq!(unique.len(), 15, "five color assets must not be prepared twice");
            for logo in AiLogo::ALL {
                let light = &images[&(logo, false)];
                let dark = &images[&(logo, true)];
                assert_eq!(u32::from(light.size(0).width), target);
                assert_eq!(u32::from(dark.size(0).height), target);
                if Arc::ptr_eq(light, dark) {
                    // Compare the omitted dark preparation against the actual output,
                    // so a later asset/tint change cannot silently change its pixels.
                    let reference = decode_sidebar_logo(logo, true, target).unwrap();
                    assert_eq!(dark.as_bytes(0), reference.as_bytes(0), "{logo:?}");
                } else {
                    assert_ne!(light.as_bytes(0), dark.as_bytes(0), "{logo:?}");
                }
            }
        }
    }
}
