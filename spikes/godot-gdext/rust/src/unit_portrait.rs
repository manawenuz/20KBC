use godot::prelude::*;
use godot::classes::{Control, IControl, Label, ProgressBar};

/// Bottom-left portrait panel showing the selected unit's stats.
///
/// Driven by GDScript (`main.gd`) via `show_single`, `show_multi`, and
/// `hide_panel`. Expects child nodes:
///   - NameLabel  (Label)
///   - HpBar      (ProgressBar)
///   - SupplyBar  (ProgressBar)
///   - CountLabel (Label)
#[derive(GodotClass)]
#[class(base = Control)]
pub struct UnitPortrait {
    base: Base<Control>,
}

#[godot_api]
impl IControl for UnitPortrait {
    fn init(base: Base<Control>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl UnitPortrait {
    /// Show detailed stats for a single selected unit.
    #[func]
    pub fn show_single(&mut self, hp: f32, max_hp: f32, supply: f32, max_supply: f32) {
        if let Some(mut label) = self
            .base()
            .get_node_or_null("NameLabel")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text("Worker");
        }

        if let Some(mut bar) = self
            .base()
            .get_node_or_null("HpBar")
            .and_then(|n| n.try_cast::<ProgressBar>().ok())
        {
            bar.set_max(max_hp as f64);
            bar.set_value(hp as f64);
        }

        if let Some(mut bar) = self
            .base()
            .get_node_or_null("SupplyBar")
            .and_then(|n| n.try_cast::<ProgressBar>().ok())
        {
            bar.set_max(max_supply as f64);
            bar.set_value(supply as f64);
            bar.set_visible(true);
        }

        if let Some(mut label) = self
            .base()
            .get_node_or_null("CountLabel")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text("1 unit");
        }

        self.base_mut().set_visible(true);
    }

    /// Show aggregate stats when multiple units are selected.
    #[func]
    pub fn show_multi(&mut self, count: i64, total_hp: f32, total_max_hp: f32) {
        if let Some(mut label) = self
            .base()
            .get_node_or_null("NameLabel")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text(&format!("{count} units selected"));
        }

        if let Some(mut bar) = self
            .base()
            .get_node_or_null("HpBar")
            .and_then(|n| n.try_cast::<ProgressBar>().ok())
        {
            bar.set_max(total_max_hp as f64);
            bar.set_value(total_hp as f64);
        }

        if let Some(mut bar) = self
            .base()
            .get_node_or_null("SupplyBar")
            .and_then(|n| n.try_cast::<ProgressBar>().ok())
        {
            bar.set_visible(false);
        }

        if let Some(mut label) = self
            .base()
            .get_node_or_null("CountLabel")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text(&format!("{count} units"));
        }

        self.base_mut().set_visible(true);
    }

    /// Hide the panel when nothing is selected.
    #[func]
    pub fn hide_panel(&mut self) {
        self.base_mut().set_visible(false);
    }
}
