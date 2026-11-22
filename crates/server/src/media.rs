use std::io::Cursor;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};

pub const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct ImageDerivative {
    pub variant: &'static str,
    pub mime_type: &'static str,
    pub width: i32,
    pub height: i32,
    pub payload_digest: String,
    pub bytes: Vec<u8>,
}

pub fn decode_avatar_data_url(raw: &str) -> Result<Vec<u8>, String> {
    let trimmed = raw.trim();
    let comma_idx = trimmed
        .find(',')
        .ok_or_else(|| "Invalid image upload payload.".to_string())?;
    let header = &trimmed[..comma_idx].to_ascii_lowercase();
    let b64_part = trimmed[comma_idx + 1..].trim();

    if !header.starts_with("data:image/")
        || !header.ends_with(";base64")
        || header.contains("svg")
    {
        return Err("Only PNG, JPEG, WebP, or GIF images are supported.".to_string());
    }

    let decoded = B64
        .decode(b64_part)
        .map_err(|_| "Could not decode uploaded image data.".to_string())?;

    if decoded.is_empty() {
        return Err("Uploaded image file is empty.".to_string());
    }
    if decoded.len() > MAX_AVATAR_BYTES {
        return Err("Image file must be 2 MB or smaller.".to_string());
    }
    Ok(decoded)
}

fn crop_square_center(img: &DynamicImage) -> DynamicImage {
    let w = img.width();
    let h = img.height();
    let side = w.min(h).max(1);
    let x = (w.saturating_sub(side)) / 2;
    let y = (h.saturating_sub(side)) / 2;
    img.crop_imm(x, y, side, side)
}

fn encode_derivative(
    img: &DynamicImage,
    variant: &'static str,
    preserve_alpha: bool,
) -> Result<ImageDerivative, String> {
    let mut out = Vec::new();
    let mime_type: &'static str = if preserve_alpha {
        let rgba = DynamicImage::ImageRgba8(img.to_rgba8());
        let mut cursor = Cursor::new(&mut out);
        rgba.write_to(&mut cursor, ImageFormat::WebP)
            .map_err(|e| format!("Failed to encode WebP derivative: {e}"))?;
        "image/webp"
    } else {
        let rgb = img.to_rgb8();
        let mut encoder = JpegEncoder::new_with_quality(&mut out, 84);
        encoder
            .encode_image(&rgb)
            .map_err(|e| format!("Failed to encode JPEG derivative: {e}"))?;
        "image/jpeg"
    };

    let mut hasher = Sha256::new();
    hasher.update(&out);
    let payload_digest = format!("{:x}", hasher.finalize());

    Ok(ImageDerivative {
        variant,
        mime_type,
        width: img.width() as i32,
        height: img.height() as i32,
        payload_digest,
        bytes: out,
    })
}

pub fn build_avatar_derivatives(raw_bytes: &[u8]) -> Result<Vec<ImageDerivative>, String> {
    if raw_bytes.len() > MAX_AVATAR_BYTES {
        return Err("Image file must be 2 MB or smaller.".to_string());
    }

    let decoded = image::ImageReader::new(Cursor::new(raw_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Unable to inspect image format: {e}"))?
        .decode()
        .map_err(|e| format!("Unsupported or invalid image file: {e}"))?;

    if decoded.width() > 8192 || decoded.height() > 8192 {
        return Err("Image dimensions must not exceed 8192x8192 pixels.".to_string());
    }

    let has_alpha = decoded.color().has_alpha();
    let square = crop_square_center(&decoded);

    let thumb_img = square.resize_exact(64, 64, FilterType::Lanczos3);
    let medium_img = square.resize_exact(256, 256, FilterType::Lanczos3);
    let large_img = if decoded.width() > 1024 || decoded.height() > 1024 {
        decoded.resize(1024, 1024, FilterType::Lanczos3)
    } else {
        decoded
    };

    Ok(vec![
        encode_derivative(&thumb_img, "thumb", has_alpha)?,
        encode_derivative(&medium_img, "medium", has_alpha)?,
        encode_derivative(&large_img, "large", has_alpha)?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn generates_three_avatar_sizes_from_image_bytes() {
        let img = ImageBuffer::<Rgba<u8>, _>::from_fn(400, 200, |x, y| {
            Rgba([(x % 255) as u8, (y % 255) as u8, 120, 255])
        });
        let dyn_img = DynamicImage::ImageRgba8(img);
        let mut png_bytes = Vec::new();
        dyn_img
            .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
            .expect("png encode");

        let derivatives = build_avatar_derivatives(&png_bytes).expect("derivatives");
        assert_eq!(derivatives.len(), 3);
        assert_eq!(derivatives[0].variant, "thumb");
        assert_eq!((derivatives[0].width, derivatives[0].height), (64, 64));
        assert_eq!(derivatives[1].variant, "medium");
        assert_eq!((derivatives[1].width, derivatives[1].height), (256, 256));
        assert_eq!(derivatives[2].variant, "large");
        assert_eq!((derivatives[2].width, derivatives[2].height), (400, 200));
        assert!(!derivatives[0].payload_digest.is_empty());
    }
}
