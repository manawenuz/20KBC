use godot::prelude::*;
use godot::classes::{Image, ImageTexture, Texture2D};
use godot::classes::image::Format;
use std::sync::Arc;

use crate::datasource::DataSource;
use crate::blp::{decode_blp, DecodedImage};

/// The 12 official WC3 team colors (Red, Blue, Teal, Purple, Yellow,
/// Orange, Green, Pink, Grey, LightBlue, DarkGreen, Brown).
#[derive(Copy, Clone)]
pub enum TeamColor {
    Red, Blue, Teal, Purple, Yellow, Orange,
    Green, Pink, Grey, LightBlue, DarkGreen, Brown,
}

impl TeamColor {
    pub fn slot(self) -> u8 {
        match self {
            TeamColor::Red       => 0,
            TeamColor::Blue      => 1,
            TeamColor::Teal      => 2,
            TeamColor::Purple    => 3,
            TeamColor::Yellow    => 4,
            TeamColor::Orange    => 5,
            TeamColor::Green     => 6,
            TeamColor::Pink      => 7,
            TeamColor::Grey      => 8,
            TeamColor::LightBlue => 9,
            TeamColor::DarkGreen => 10,
            TeamColor::Brown     => 11,
        }
    }

    pub fn mpq_path(self) -> &'static str {
        match self {
            TeamColor::Red       => "ReplaceableTextures/TeamColor/TeamColor00.blp",
            TeamColor::Blue      => "ReplaceableTextures/TeamColor/TeamColor01.blp",
            TeamColor::Teal      => "ReplaceableTextures/TeamColor/TeamColor02.blp",
            TeamColor::Purple    => "ReplaceableTextures/TeamColor/TeamColor03.blp",
            TeamColor::Yellow    => "ReplaceableTextures/TeamColor/TeamColor04.blp",
            TeamColor::Orange    => "ReplaceableTextures/TeamColor/TeamColor05.blp",
            TeamColor::Green     => "ReplaceableTextures/TeamColor/TeamColor06.blp",
            TeamColor::Pink      => "ReplaceableTextures/TeamColor/TeamColor07.blp",
            TeamColor::Grey      => "ReplaceableTextures/TeamColor/TeamColor08.blp",
            TeamColor::LightBlue => "ReplaceableTextures/TeamColor/TeamColor09.blp",
            TeamColor::DarkGreen => "ReplaceableTextures/TeamColor/TeamColor10.blp",
            TeamColor::Brown     => "ReplaceableTextures/TeamColor/TeamColor11.blp",
        }
    }
}

/// Lazy-loaded cache of team-color Texture2Ds, one per slot. Looks up
/// the BLP via DataSource and decodes via BLP decoder on first access.
pub struct TeamColorCache {
    ds: Arc<dyn DataSource>,
    textures: [Option<Gd<Texture2D>>; 12],
}

impl TeamColorCache {
    pub fn new(ds: Arc<dyn DataSource>) -> Self {
        Self {
            ds,
            textures: [
                None, None, None, None,
                None, None, None, None,
                None, None, None, None,
            ],
        }
    }

    pub fn texture(&mut self, color: TeamColor) -> Option<Gd<Texture2D>> {
        let slot = color.slot() as usize;
        if self.textures[slot].is_none() {
            self.textures[slot] = self.load(color);
        }
        self.textures[slot].clone()
    }

    fn load(&self, color: TeamColor) -> Option<Gd<Texture2D>> {
        let path = color.mpq_path();
        let bytes = self.ds.read(path)?;
        let DecodedImage { width, height, rgba } = decode_blp(&bytes).ok()?;

        let mut image = Image::new_gd();
        image.set_data(
            width as i32,
            height as i32,
            false,
            Format::RGBA8,
            &PackedByteArray::from(rgba.as_slice()),
        );
        let texture = ImageTexture::create_from_image(&image)?;
        Some(texture.upcast::<Texture2D>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_12_team_color_paths_well_formed() {
        use TeamColor::*;
        let all = [Red, Blue, Teal, Purple, Yellow, Orange, Green, Pink, Grey, LightBlue, DarkGreen, Brown];
        for c in all {
            assert!(c.mpq_path().starts_with("ReplaceableTextures"));
            assert!(c.mpq_path().ends_with(".blp"));
            assert!(c.slot() < 12);
        }
    }
}
