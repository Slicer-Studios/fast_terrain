use godot::prelude::*;
use godot::classes::{ISprite2D, Sprite2D};

struct FastTerrain;

#[gdextension]
unsafe impl ExtensionLibrary for FastTerrain {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Editor => {
                godot_print!("FastTerrain initialized!");
            }
            _ => {}
        }
    }
}

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct Player {
    speed: f64,
    angular_speed: f64,

    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for Player {
    fn init(base: Base<Sprite2D>) -> Self {
        Self {
            speed: 400.0,
            angular_speed: std::f64::consts::PI,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // In GDScript, this would be:
        // rotation += angular_speed * delta

        let radians = (self.angular_speed * delta) as f32;
        self.base_mut().rotate(radians);
        // The 'rotate' method requires a f32,
        // therefore we convert 'self.angular_speed * delta' which is a f64 to a f32
    }
}

