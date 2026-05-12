use godot::prelude::*;
use godot::classes::{
    Control, IControl, InputEvent, InputEventMouseButton, InputEventMouseMotion,
};
use godot::global::MouseButton;

#[derive(GodotClass)]
#[class(base = Control)]
pub struct BoxSelector {
    dragging: bool,
    start: Vector2,
    end: Vector2,
    base: Base<Control>,
}

#[godot_api]
impl IControl for BoxSelector {
    fn init(base: Base<Control>) -> Self {
        Self {
            dragging: false,
            start: Vector2::ZERO,
            end: Vector2::ZERO,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        if self.dragging {
            self.base_mut().queue_redraw();
        }
    }

    fn draw(&mut self) {
        if !self.dragging {
            return;
        }

        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);

        let rect = Rect2::from_components(min_x, min_y, max_x - min_x, max_y - min_y);

        // Yellow translucent fill
        let fill_color = Color::from_rgba(1.0, 1.0, 0.0, 0.15);
        self.base_mut().draw_rect(rect, fill_color);

        // Yellow outline — draw four lines
        let outline_color = Color::from_rgba(1.0, 1.0, 0.0, 0.6);

        // Top
        self.base_mut().draw_line(
            Vector2::new(min_x, min_y),
            Vector2::new(max_x, min_y),
            outline_color,
        );
        // Bottom
        self.base_mut().draw_line(
            Vector2::new(min_x, max_y),
            Vector2::new(max_x, max_y),
            outline_color,
        );
        // Left
        self.base_mut().draw_line(
            Vector2::new(min_x, min_y),
            Vector2::new(min_x, max_y),
            outline_color,
        );
        // Right
        self.base_mut().draw_line(
            Vector2::new(max_x, min_y),
            Vector2::new(max_x, max_y),
            outline_color,
        );
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if let Ok(button) = event.clone().try_cast::<InputEventMouseButton>() {
            let btn = button.get_button_index();
            if btn == MouseButton::LEFT {
                if button.is_pressed() {
                    // Mouse down — start drag
                    self.dragging = true;
                    self.start = button.get_position();
                    self.end = self.start;
                    self.base_mut().queue_redraw();
                } else if self.dragging {
                    // Mouse up — end drag
                    self.end = button.get_position();
                    let dx = self.end.x - self.start.x;
                    let dy = self.end.y - self.start.y;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq > 16.0 {
                        // Distance > 4 pixels — emit signal
                        let start = self.start;
                        let end = self.end;
                        self.base_mut()
                            .emit_signal("selection_box", &[start.to_variant(), end.to_variant()]);
                    }

                    self.dragging = false;
                    self.base_mut().queue_redraw();
                }
            }
        } else if let Ok(motion) = event.try_cast::<InputEventMouseMotion>() {
            if self.dragging {
                self.end = motion.get_position();
                self.base_mut().queue_redraw();
            }
        }
    }
}

#[godot_api]
impl BoxSelector {
    /// Signal emitted on drag release. Args: Vector2 start, Vector2 end (screen px).
    #[signal]
    fn selection_box(start: Vector2, end: Vector2);
}
