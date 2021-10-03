use std::collections::VecDeque;
use std::f64::consts::TAU;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use ld_game_engine::{
    Context,
    GameState,
    StateTransition,
    event::Event,
    event::Event::{KeyUp, MouseDown}
};
use ld_game_engine::event::Event::{MouseMove, MouseUp};
use ld_game_engine::event::KeyMeta;
use crate::{
    ChaosTheory,
    Pos,
    rope::Rope
};

#[derive(Debug)]
pub struct MainGame {
    rope: Rope,
    setup: Option<Rope>,
    started: bool,

    prev_trails: VecDeque<VecDeque<Pos>>,
    trail: VecDeque<Pos>,
    creating: Option<Pos>,
    level: Level,
}

#[derive(Debug)]
struct Circle {
    pos: Pos,
    radius: f64,
}

impl Circle {

    pub fn extend(&self, extra_radius: f64) -> Circle {
        Self {
            pos: self.pos,
            radius: self.radius + extra_radius,
        }
    }

    pub fn contains(&self, pos: Pos) -> bool {
        (self.pos - pos).magnitude_squared() <= self.radius * self.radius
    }

    pub fn project(&self, pos: Pos) -> Pos {
        self.pos + (pos - self.pos).normalize() * self.radius
    }
}

#[derive(Debug)]
pub struct Level {
    init_state: Rope,
    targets: Vec<Circle>,
    red_zones: Vec<Circle>,
}

impl Level {
    pub fn test() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into(), [0.0, 1000.0].into());
        rope.add([0.0, -300.0].into());
        Level {
            init_state: rope,
            targets: vec![
                Circle {
                    pos: [0.0, 0.0].into(),
                    radius: 50.0,
                },
                Circle {
                    pos: [650.0, 0.0].into(),
                    radius: 100.0,
                },
            ],
            red_zones: vec![
                Circle {
                    pos: [0.0, 0.0].into(),
                    radius: 300.0,
                },
            ],
        }
    }
}

impl MainGame {
    pub fn new(level: Level) -> Self {
        Self {
            rope: level.init_state.clone(),
            setup: None,
            started: false,
            prev_trails: VecDeque::new(),
            trail: VecDeque::new(),
            creating: None,
            level,
        }
    }
}

pub const BG_COLOR: &str = "black";
pub const BG_LINE_COLOR: &str = "#333040";

fn draw_background(context: &Context<ChaosTheory>) {
    let size = context.surface().size();
    let surface = context.surface().context();

    surface.set_fill_style(&BG_COLOR.into());
    surface.fill_rect(0.0, 0.0, size.x, size.y);

    surface.set_stroke_style(&BG_LINE_COLOR.into());
    surface.set_line_width(1.0);

    let offset = size / 2.0;
    let mut i = offset.x % 100.0;
    while i < size.x {
        surface.begin_path();
        surface.move_to(i, 0.0);
        surface.line_to(i, size.y);
        surface.stroke();
        i += 100.0;
    }
    i = offset.y % 100.0;
    while i < size.y {
        surface.begin_path();
        surface.move_to(0.0, i);
        surface.line_to(size.x, i);
        surface.stroke();
        i += 100.0;
    }
}

fn draw_trail(surface: &CanvasRenderingContext2d, trail: &VecDeque<Pos>) {
    surface.set_line_width(1.0);

    let mut opacity = 0.0;

    if trail.len() > 1 {
        let mut start = trail[0];
        for pos in trail.range(1..) {
            opacity += 1.0 / trail.len() as f64;
            surface.set_global_alpha(opacity);
            surface.set_line_width(1.0 + opacity);
            surface.begin_path();
            surface.move_to(start.x, start.y);
            start = *pos;
            surface.line_to(pos.x, pos.y);
            surface.stroke();
        }
    }
    surface.set_global_alpha(1.0);
}

impl MainGame {
    fn constrain(&self, pos: Pos) -> Pos {
        for red_zone in &self.level.red_zones {
            if red_zone.contains(pos) {
                return red_zone.project(pos);
            }
        }
        for target in &self.level.targets {
            if target.contains(pos) {

            }
        }
        pos
    }
}

impl GameState<ChaosTheory> for MainGame {
    fn on_event(&mut self, event: Event, context: &mut Context<ChaosTheory>) -> StateTransition<ChaosTheory> {
        let center = context.surface().size() / 2.0;
        match event {
            MouseDown { pos, .. } if !self.started => {
                let pos = pos - center;
                if (self.rope.tail() - pos).magnitude() < 15.0 {
                    self.creating = Some(pos)
                }
            }
            MouseMove { pos, .. } => {
                let pos = pos - center;
                if self.creating.is_some() {
                    self.creating = Some(self.constrain(pos))
                }
            }
            MouseUp { pos, .. } => {
                let pos = pos - center;
                if self.creating.is_some() {
                    self.rope.add(self.constrain(pos));
                    self.creating = None;
                }
            }
            KeyUp { code: 32, ..} => {
                self.started = !self.started;
                if self.started && self.setup.is_none() {
                    self.setup = Some(self.rope.clone());
                    self.rope.jiggle();
                }
            }
            KeyUp { code: 67, ..} => {
                self.prev_trails.clear()
            }
            KeyUp { code: 82, meta: KeyMeta { shift , ..}, ..} => {
                match &self.setup {
                    Some(setup) if !shift => {
                        self.rope = setup.clone();
                        self.rope.jiggle();
                        self.prev_trails.push_back(std::mem::take(&mut self.trail));
                    },
                    _ => {
                        self.rope = self.level.init_state.clone();
                        self.setup = None;
                        self.trail.clear();
                        self.prev_trails.clear();
                        self.started = false;
                    }
                }
            }
            _ => {}
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<ChaosTheory>) -> StateTransition<ChaosTheory> {
        let size = context.surface().size();
        draw_background(context);

        let tail = self.rope.tail();

        if self.started {
            let mut delta_time = context.delta_time();

            if delta_time > 0.05 {
                log::warn!("delta time is > 50ms");
                delta_time = 0.0;
            }

            self.rope.simulate(delta_time, 15);

            self.trail.push_back(tail);
            if self.trail.len() > 60 * 10 {
                self.trail.pop_front();
            }
        }

        let surface = context.surface().context();
        let center = size / 2.0;
        surface.translate(center.x, center.y).unwrap();

        surface.set_stroke_style(&"#7734eb".into());
        draw_trail(&surface, &self.trail);

        surface.set_stroke_style(&"gray".into());
        for trail in &self.prev_trails {
            draw_trail(&surface, trail);
        }

        surface.set_fill_style(&"white".into());
        surface.set_stroke_style(&"#730c05".into());
        surface.set_line_width(4.0);
        let array = js_sys::Array::new();
        array.push(&10.into());
        array.push(&10.into());
        surface.set_line_dash(&JsValue::from(array)).unwrap();

        if !self.started {
            for red_zone in &self.level.red_zones {
                surface.begin_path();
                surface.arc(red_zone.pos.x, red_zone.pos.y, red_zone.radius, 0.0, TAU).unwrap();
                surface.stroke();

                surface.save();
                surface.clip();

                surface.set_global_alpha(0.5);
                surface.set_line_dash(&JsValue::from(js_sys::Array::new())).unwrap();
                surface.set_line_width(2.0);

                for i in 0..(red_zone.radius as u32 / 10) {
                    surface.begin_path();
                    surface.move_to(i as f64 * 20.0 + red_zone.pos.x - red_zone.radius, red_zone.pos.y - red_zone.radius);
                    surface.line_to(red_zone.pos.x - red_zone.radius, i as f64 * 20.0 + red_zone.pos.y - red_zone.radius);

                    surface.move_to(i as f64 * 20.0 + red_zone.pos.x - red_zone.radius, red_zone.pos.y + red_zone.radius);
                    surface.line_to(red_zone.pos.x + red_zone.radius, i as f64 * 20.0 + red_zone.pos.y - red_zone.radius);

                    surface.stroke();
                }

                surface.restore();
            }
        }

        surface.set_line_dash(&JsValue::from(js_sys::Array::new())).unwrap();

        surface.set_stroke_style(&"#183769".into());
        surface.set_line_width(4.0);

        for target in &self.level.targets {
            surface.begin_path();
            surface.arc(target.pos.x, target.pos.y, target.radius, 0.0, TAU).unwrap();

            if target.contains(tail) {
                surface.set_global_alpha(0.5);
                surface.fill();
                surface.set_global_alpha(1.0);
            }
            surface.stroke();
        }

        surface.set_stroke_style(&"white".into());

        for constraint in &self.rope.constraints {
            let pos_a = constraint.point_a.borrow().pos();
            let pos_b = constraint.point_b.borrow().pos();

            surface.begin_path();
            surface.move_to(pos_a.x, pos_a.y);
            surface.line_to(pos_b.x, pos_b.y);
            surface.stroke();

            surface.begin_path();
            surface.arc(pos_b.x, pos_b.y, 7.0, 0.0, TAU).unwrap();
            surface.fill();
        }

        surface.begin_path();
        surface.arc(self.rope.root.x, self.rope.root.y, 15.0, 0.0, TAU).unwrap();
        surface.fill();

        if let Some(pos) = self.creating {
            let tail = self.rope.tail();

            surface.set_stroke_style(&"gray".into());
            surface.set_line_width(4.0);

            let array = js_sys::Array::new();
            array.push(&2.into());
            array.push(&10.into());
            surface.set_line_dash(&JsValue::from(array)).unwrap();

            surface.begin_path();
            surface.arc(tail.x, tail.y, (pos - tail).magnitude(), 0.0, TAU).unwrap();
            surface.stroke();

            let array = js_sys::Array::new();
            array.push(&15.into());
            array.push(&5.into());
            surface.set_line_dash(&JsValue::from(array)).unwrap();
            surface.set_stroke_style(&"white".into());
            surface.set_line_width(4.0);

            surface.begin_path();
            surface.move_to(tail.x, tail.y);
            surface.line_to(pos.x, pos.y);
            surface.stroke();

            surface.set_line_dash(&JsValue::from(js_sys::Array::new())).unwrap();

            surface.begin_path();
            surface.arc(pos.x, pos.y, 7.0, 0.0, TAU).unwrap();
            surface.fill();
        }

        StateTransition::None
    }
}
