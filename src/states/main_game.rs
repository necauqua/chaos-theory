use std::collections::VecDeque;
use std::f64::consts::TAU;
use nalgebra::Vector2;
use ld_game_engine::{Context, GameState, StateTransition};
use ld_game_engine::event::Event;
use ld_game_engine::event::Event::{KeyUp, MouseDown};
use crate::ChaosTheory;
use crate::rope::Rope;

#[derive(Debug)]
pub struct MainGame {
    rope: Rope,
    started: bool,
    trail: VecDeque<Vector2<f64>>
}

impl MainGame {

    pub fn new() -> Self {
        Self {
            rope: Rope::new([0.0, 0.0].into()),
            started: false,
            trail: VecDeque::new(),
        }
    }
}

pub const BG_COLOR: &str = "black";
pub const BG_LINE_COLOR: &str = "#333040";

pub fn draw_background(context: &Context<ChaosTheory>, offset: Vector2<f64>) {
    let size = context.surface().size();
    let surface = context.surface().context();

    surface.set_fill_style(&BG_COLOR.into());
    surface.fill_rect(0.0, 0.0, size.x, size.y);

    surface.set_stroke_style(&BG_LINE_COLOR.into());
    surface.set_line_width(1.0);

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

impl GameState<ChaosTheory> for MainGame {
    fn on_event(&mut self, event: Event, context: &mut Context<ChaosTheory>) -> StateTransition<ChaosTheory> {
        match event {
            MouseDown { pos, .. } if !self.started => {
                let size = context.surface().size();
                self.rope.add(pos - size / 2.0)
            }
            KeyUp { code: 32, ..} => {
                self.started = true
            }
            KeyUp { code: 82, ..} => {
                self.rope = Rope::new([0.0, 0.0].into());
                self.trail.clear();
                self.started = false;
            }
            _ => {}
        }
        StateTransition::None
    }


    fn on_update(&mut self, context: &mut Context<ChaosTheory>) -> StateTransition<ChaosTheory> {
        let size = context.surface().size();
        let center = size / 2.0;

        draw_background(context, center);

        if self.started {
            self.rope.simulate([0.0, 0.2].into(), 15);

            if let Some(tail) = self.rope.tail() {
                self.trail.push_back(tail.borrow().pos());

                if self.trail.len() > 60 * 10 {
                    self.trail.pop_front();
                }
            }
        }

        let surface = context.surface().context();
        surface.translate(center.x, center.y).unwrap();

        surface.set_fill_style(&"white".into());

        surface.set_stroke_style(&"gray".into());
        surface.set_line_width(1.0);

        if self.trail.len() > 1 {
            let start = self.trail[0];
            surface.begin_path();
            surface.move_to(start.x, start.y);
            for pos in self.trail.range(1..) {
                surface.line_to(pos.x, pos.y);
            }
            surface.stroke();
        }

        surface.set_stroke_style(&"white".into());
        surface.set_line_width(4.0);

        for stick in &self.rope.sticks {
            let pos_a = stick.point_a.borrow().pos();
            let pos_b = stick.point_b.borrow().pos();

            surface.begin_path();
            surface.move_to(pos_a.x, pos_a.y);
            surface.line_to(pos_b.x, pos_b.y);
            surface.stroke();

            surface.begin_path();
            surface.arc(pos_a.x, pos_a.y, 7.0, 0.0, TAU).unwrap();
            surface.fill();
            surface.begin_path();
            surface.arc(pos_b.x, pos_b.y, 7.0, 0.0, TAU).unwrap();
            surface.fill();
        }

        surface.begin_path();
        surface.arc(self.rope.root.x, self.rope.root.y, 15.0, 0.0, TAU).unwrap();
        surface.fill();

        StateTransition::None
    }
}
