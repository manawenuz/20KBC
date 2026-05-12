/// HUD built with Fyrox's retained-mode UI system.
///
/// Displays three labels in the top-left corner:
///   Wood: <n>   Stone: <n>   Tick: <n>
///
/// Fyrox API notes (0.34):
///   - `BuildContext` is obtained via `ctx.user_interface.build_ctx()`.
///   - `TextMessage::text(...)` with `MessageDirection::ToWidget` updates label text.
///   - `WidgetBuilder::with_desired_position` sets screen-space position.
///   - All handles are `Handle<UiNode>`.
use fyrox::{
    core::pool::Handle,
    gui::{
        message::{MessageDirection, UiMessage},
        text::{TextBuilder, TextMessage},
        widget::WidgetBuilder,
        BuildContext, UiNode, UserInterface,
    },
};

pub struct GameHud {
    pub wood_label: Handle<UiNode>,
    pub stone_label: Handle<UiNode>,
    pub tick_label: Handle<UiNode>,
}

impl GameHud {
    pub fn new(ctx: &mut BuildContext) -> Self {
        let wood_label = TextBuilder::new(
            WidgetBuilder::new()
                .with_desired_position(fyrox::core::algebra::Vector2::new(10.0, 10.0)),
        )
        .with_text("Wood: 0")
        .build(ctx);

        let stone_label = TextBuilder::new(
            WidgetBuilder::new()
                .with_desired_position(fyrox::core::algebra::Vector2::new(10.0, 34.0)),
        )
        .with_text("Stone: 0")
        .build(ctx);

        let tick_label = TextBuilder::new(
            WidgetBuilder::new()
                .with_desired_position(fyrox::core::algebra::Vector2::new(10.0, 58.0)),
        )
        .with_text("Tick: 0")
        .build(ctx);

        Self {
            wood_label,
            stone_label,
            tick_label,
        }
    }

    /// Push updated resource counters into the UI.
    /// Call this once per frame (or whenever values change).
    pub fn update(&self, ui: &mut UserInterface, wood: u32, stone: u32, tick: u64) {
        ui.send_message(TextMessage::text(
            self.wood_label,
            MessageDirection::ToWidget,
            format!("Wood: {wood}"),
        ));
        ui.send_message(TextMessage::text(
            self.stone_label,
            MessageDirection::ToWidget,
            format!("Stone: {stone}"),
        ));
        ui.send_message(TextMessage::text(
            self.tick_label,
            MessageDirection::ToWidget,
            format!("Tick: {tick}"),
        ));
    }
}
