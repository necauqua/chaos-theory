use std::borrow::Cow;
use std::collections::VecDeque;
use std::f64::consts::TAU;

use ld_game_engine::{
    Context,
    event::{
        Event::{self, KeyUp, MouseDown, MouseMove, MouseUp},
        KeyMeta,
    }, GameState, StateTransition, v2, V2,
};
use ld_game_engine::surface::{SurfaceContext, SurfaceContextExt};
use ld_game_engine::ui::Button;

use crate::{ChaosTheory, rope::Rope};

#[derive(Debug)]
enum SimStatus {
    Setup,
    Running,
    Paused,
}

#[derive(Debug)]
enum WinStatus {
    NotYet,
    Won { bonuses: usize },
}

#[derive(Debug)]
pub struct MainGame {
    rope: Rope,
    setup: Option<Rope>,
    sim_status: SimStatus,
    win_status: WinStatus,
    prev_trails: VecDeque<VecDeque<V2>>,
    trail: VecDeque<V2>,
    creating: Option<V2>,
    touched_targets: Box<[u8]>,
    touching_target: Option<usize>,
    level: Level,

    exit_button: Button,
}

#[derive(Debug)]
struct Circle {
    pos: V2,
    radius: f64,
}

impl Circle {
    pub fn extend(&self, extra_radius: f64) -> Circle {
        Self {
            pos: self.pos,
            radius: self.radius + extra_radius,
        }
    }

    pub fn contains(&self, pos: V2) -> bool {
        (self.pos - pos).magnitude_squared() <= self.radius * self.radius
    }

    pub fn project(&self, pos: V2) -> V2 {
        self.pos + (pos - self.pos).normalize() * self.radius
    }
}

#[derive(Debug)]
struct Target {
    zone: Circle,
    closed: f64,
}

#[derive(Debug)]
pub struct Level {
    init_state: Rope,
    targets: Vec<Target>,
    red_zones: Vec<Circle>,
}

impl Level {
    pub fn test() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into(), [0.0, 1000.0].into());
        rope.add([0.0, -300.0].into());
        Level {
            init_state: rope,
            targets: vec![
                Target {
                    zone: Circle {
                        pos: [0.0, 0.0].into(),
                        radius: 50.0,
                    },
                    closed: 250.0,
                },
                // Target {
                //     zone: Circle {
                //         pos: [650.0, 0.0].into(),
                //         radius: 100.0,
                //     },
                //     closed: 100.0,
                // },
            ],
            red_zones: vec![],
        }
    }
}

impl MainGame {
    pub fn new(level: Level, game: &mut ChaosTheory) -> Self {
        Self {
            rope: level.init_state.clone(),
            setup: None,
            sim_status: SimStatus::Setup,
            win_status: WinStatus::NotYet,
            prev_trails: VecDeque::new(),
            trail: VecDeque::new(),
            creating: None,
            touched_targets: vec![0; level.targets.len()].into_boxed_slice(),
            touching_target: None,
            level,

            exit_button: game.button(""),
        }
    }
}

const BG_COLOR: &str = "black";
const BG_LINE_COLOR: &str = "#333040";
const TARGET_COLOR: &str = "#183769";
const BONUS_COLOR: &str = "#ffdf00";
const DANGER_COLOR: &str = "#730c05";

fn draw_background(context: &Context<ChaosTheory>) {
    let size = context.surface().size();
    let surface = context.surface().context();

    surface.fill_color(BG_COLOR);
    surface.fill_rect(0.0, 0.0, size.x, size.y);

    surface.stroke_color(BG_LINE_COLOR);
    surface.set_line_width(1.0);

    let offset = size / 2.0;
    let mut i = offset.x % 100.0;
    while i < size.x {
        surface.line(v2![i, 0.0], v2![i, size.y]);
        i += 100.0;
    }
    i = offset.y % 100.0;
    while i < size.y {
        surface.line(v2![0.0, i], v2![size.x, i]);
        i += 100.0;
    }
}

fn draw_trail(surface: &SurfaceContext, trail: &VecDeque<V2>) {
    surface.set_line_width(1.0);

    let mut opacity = 0.0;

    if trail.len() > 1 {
        let mut prev = trail[0];
        for pos in trail.range(1..).copied() {
            opacity += 1.0 / trail.len() as f64;
            surface.set_global_alpha(opacity);
            surface.set_line_width(1.0 + opacity);
            surface.line(prev, pos);
            prev = pos;
        }
    }
    surface.set_global_alpha(1.0);
}

fn draw_stripes(surface: &SurfaceContext, from: V2, to: V2) {
    surface.set_global_alpha(0.5);
    surface.set_line_width(2.0);

    let width_steps = (to.x - from.x) as u32 / 20;
    let height_steps = (to.y - from.y) as u32 / 20;
    let corner_steps = width_steps.min(height_steps);

    surface.begin_path();
    for i in 0..corner_steps {
        surface.move_to(from.x + i as f64 * 20.0, from.y);
        surface.line_to(from.x, from.y + i as f64 * 20.0);
    }
    if width_steps > height_steps {
        for i in 0..(width_steps - height_steps) {
            surface.move_to(from.x + (i + height_steps) as f64 * 20.0, from.y);
            surface.line_to(from.x + i as f64 * 20.0, to.y);
        }
    } else {
        for i in 0..(height_steps - width_steps) {
            surface.move_to(to.x, from.y + i as f64 * 20.0);
            surface.line_to(from.x, from.y + (i + width_steps) as f64 * 20.0);
        }
    }
    for i in 0..corner_steps {
        surface.move_to(to.x - (i + 1) as f64 * 20.0, to.y);
        surface.line_to(to.x, to.y - (i + 1) as f64 * 20.0);
    }
    surface.stroke();
}

impl MainGame {
    fn constrain(&self, pos: V2) -> V2 {
        for red_zone in &self.level.red_zones {
            if red_zone.contains(pos) {
                return red_zone.project(pos);
            }
        }
        for target in &self.level.targets {
            let constraint = target.zone.extend(target.closed);
            if constraint.contains(pos) {
                return constraint.project(pos);
            }
        }
        pos
    }

    fn reset(&mut self, soft: bool) {
        self.win_status = WinStatus::NotYet;
        for t in self.touched_targets.iter_mut() {
            *t = 0;
        }
        match &self.setup {
            Some(setup) if soft => {
                self.rope = setup.clone();
                self.rope.jiggle();
                self.prev_trails.push_back(std::mem::take(&mut self.trail));
                if self.prev_trails.len() > 127 {
                    self.prev_trails.pop_front();
                }
            }
            _ => {
                self.rope = self.level.init_state.clone();
                self.setup = None;
                self.trail.clear();
                self.prev_trails.clear();
                self.sim_status = SimStatus::Setup;
            }
        }
    }
}

impl GameState<ChaosTheory> for MainGame {
    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<ChaosTheory>,
    ) -> StateTransition<ChaosTheory> {
        let center = context.surface().size() / 2.0;
        match event {
            MouseDown { pos, .. } if matches!(self.sim_status, SimStatus::Setup) => {
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
            KeyUp { code: 32, .. } => {
                self.sim_status = match self.sim_status {
                    SimStatus::Setup | SimStatus::Paused => SimStatus::Running,
                    SimStatus::Running => SimStatus::Paused,
                };
                if matches!(self.sim_status, SimStatus::Running) && self.setup.is_none() {
                    self.setup = Some(self.rope.clone());
                    self.rope.jiggle();
                }
            }
            KeyUp { code: 67, .. } => self.prev_trails.clear(),
            KeyUp {
                code: 82,
                meta: KeyMeta { shift, .. },
                ..
            } => self.reset(!shift),
            _ => {}
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<ChaosTheory>) -> StateTransition<ChaosTheory> {
        let size = context.surface().size();
        draw_background(context);

        let tail = self.rope.tail();

        if matches!(self.sim_status, SimStatus::Running) {
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

        surface.stroke_color("#7734eb");
        draw_trail(&surface, &self.trail);

        surface.stroke_color("gray");
        for trail in &self.prev_trails {
            draw_trail(&surface, trail);
        }

        surface.fill_color("white");
        surface.set_line_width(4.0);

        if matches!(self.sim_status, SimStatus::Setup) {
            surface.line_dash(&[10.0, 10.0]);
            surface.stroke_color(DANGER_COLOR);

            for red_zone in &self.level.red_zones {
                surface.circle(red_zone.pos, red_zone.radius);

                surface.save();
                surface.clip();

                surface.set_global_alpha(0.5);
                surface.line_dash(&[]);
                surface.set_line_width(2.0);

                draw_stripes(
                    &surface,
                    red_zone.pos - v2![red_zone.radius],
                    red_zone.pos + v2![red_zone.radius],
                );

                surface.restore();
            }

            // surface.stroke_color(TARGET_COLOR);
            for target in &self.level.targets {
                let zone = &target.zone;
                let closed = zone.radius + target.closed;
                surface.circle(zone.pos, closed);

                surface.begin_path();
                surface
                    .arc(zone.pos.x, zone.pos.y, zone.radius, 0.0, TAU)
                    .unwrap();
                surface
                    .arc(zone.pos.x, zone.pos.y, closed, 0.0, TAU)
                    .unwrap();
                surface.close_path();

                surface.save();
                surface.clip_evenodd();

                surface.line_dash(&[]);
                surface.set_line_width(2.0);

                draw_stripes(&surface, zone.pos - v2![closed], zone.pos + v2![closed]);

                surface.restore();
                surface.set_global_alpha(1.0);
            }
            surface.line_dash(&[]);
        }

        surface.stroke_color(TARGET_COLOR);
        surface.fill_color(TARGET_COLOR);
        surface.set_line_width(4.0);

        let mut reset_touching_target = true;
        for (i, target) in self.level.targets.iter().enumerate() {
            let zone = &target.zone;
            if zone.contains(tail) {
                if self.touching_target.is_none() {
                    self.touched_targets[i] += 1;
                }
                self.touching_target = Some(i);
                reset_touching_target = false;
            }
            let level = self.touched_targets[i];
            if level > 0 {
                surface.set_global_alpha(0.5);
                if level > 1 {
                    surface.fill_color(BONUS_COLOR);
                }
                surface.fill_circle(zone.pos, zone.radius);
                surface.set_global_alpha(1.0);
                if level > 1 {
                    surface.fill_color(TARGET_COLOR);
                }
                if level > 2 {
                    let half_radius = zone.radius / 2.0;
                    surface.set_font("24px monospace");
                    surface
                        .fill_text_with_max_width(
                            &level.to_string(),
                            zone.pos.x + half_radius,
                            zone.pos.y + half_radius,
                            half_radius,
                        )
                        .unwrap();
                }
            }
            surface.circle(zone.pos, zone.radius);
        }

        if matches!(self.win_status, WinStatus::Won { .. })
            || matches!(self.win_status, WinStatus::NotYet)
            && self.touched_targets.iter().all(|&i| i > 0)
        {
            let sum = self
                .touched_targets
                .iter()
                .map(|&b| b as usize)
                .sum::<usize>();
            self.win_status = WinStatus::Won {
                bonuses: sum - self.touched_targets.len(),
            };
        }

        if reset_touching_target {
            self.touching_target = None;
        }

        surface.stroke_color("white");
        surface.fill_color("white");

        for constraint in &self.rope.constraints {
            let pos_b = constraint.point_b.borrow().pos();
            surface.line(constraint.point_a.borrow().pos(), pos_b);
            surface.fill_circle(pos_b, 7.0);
        }

        surface.fill_circle(self.rope.root, 15.0);

        if let WinStatus::Won { bonuses } = self.win_status {
            let top = -context.surface().size().y / 2.0;

            let text =
                if bonuses == 0 {
                    Cow::Borrowed("You win")
                } else {
                    Cow::Owned(format!("You win (+{})", bonuses))
                };
            surface.fill_text(text.as_ref(), 0.0, top + 200.0).unwrap();

            self.exit_button.set_text("finish");
            self.exit_button.on_update(context, v2![0.0, top + 200.0 + context.rem_to_px(1.6)]);
        }

        if let Some(pos) = self.creating {
            let tail = self.rope.tail();

            surface.stroke_color("gray");
            surface.set_line_width(4.0);

            surface.line_dash(&[2.0, 10.0]);

            surface.circle(tail, (pos - tail).magnitude());

            surface.line_dash(&[15.0, 5.0]);
            surface.stroke_color("white");
            surface.set_line_width(4.0);
            surface.line(tail, pos);
            surface.line_dash(&[]);

            surface.fill_circle(pos, 7.0);
        }

        StateTransition::None
    }
}
