use blp::core::decode::decode_to_rgba;

pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>, // tightly packed RGBA8
}

/// Decode a BLP byte buffer (BLP1 JPEG or paletted, BLP2 DXT or paletted)
/// into RGBA8 mipmap level 0. Returns Err with a short reason on failure.
pub fn decode_blp(bytes: &[u8]) -> Result<DecodedImage, String> {
    let dyn_img = decode_to_rgba(bytes).map_err(|e| format!("blp decode failed: {}", e))?;
    let width = dyn_img.width();
    let height = dyn_img.height();
    let rgba = dyn_img.to_rgba8().into_raw();
    Ok(DecodedImage {
        width,
        height,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn decode_peasant_blp() {
        let data = fs::read("/Volumes/samGames/WC3/test_peasant.blp").ok();
        if let Some(bytes) = data {
            let img = decode_blp(&bytes).unwrap();
            assert!(img.width > 0 && img.height > 0);
            assert_eq!(img.rgba.len(), (img.width * img.height * 4) as usize);
        }
    }
}
