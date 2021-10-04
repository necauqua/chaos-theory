use std::{
    borrow::Cow,
    collections::VecDeque,
    f64::consts::TAU,
};

use serde::*;

use ld_game_engine::{
    Context,
    event::{
        Event::{self, KeyUp, MouseDown, MouseMove, MouseUp},
        KeyMeta,
    },
    event::MouseButton,
    GameState,
    StateTransition,
    surface::{SurfaceContext, SurfaceContextExt},
    ui::Button,
    v2,
    V2,
};

use crate::{
    BUTTON_COLOR,
    ChaosTheory,
    data::StoredData,
    HOVER_COLOR,
    rope::Rope,
    tutorial::Tutorial,
};

#[derive(Debug)]
enum SimStatus {
    Setup,
    Running { setup: Rope },
    Paused { setup: Rope },
}

#[derive(Debug)]
enum WinStatus {
    NotYet,
    Won { bonuses: usize },
}

#[derive(Debug)]
pub struct MainGame {
    rope: Rope,
    sim_status: SimStatus,
    win_status: WinStatus,
    prev_trails: VecDeque<VecDeque<V2>>,
    trail: VecDeque<V2>,
    creating: Option<V2>,
    touched_targets: Box<[u8]>,
    touching_target: Option<usize>,

    next_level_button: Button,

    menu_shown: bool,
    menu_hovered: bool,

    sound_button: Button,
    music_button: Button,
    skip_button: Button,
    tutorial_button: Button,

    tutorial: Option<Tutorial>,

    level: Level,
    next_level: Option<Level>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct Target {
    zone: Circle,
    closed: f64,
}

#[derive(Debug)]
pub struct Level {
    init_state: Rope,
    gravity: V2,
    targets: Vec<Target>,
    red_zones: Vec<Circle>,
    tutorial: bool,

    custom_text: Option<&'static str>,

    next_level: Option<fn() -> Level>,
}

impl Level {
    pub fn tutorial_level() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into());
        rope.add([0.0, -300.0].into());
        Level {
            init_state: rope,
            gravity: v2![0.0, 1000.0],
            targets: vec![
                Target {
                    zone: Circle {
                        pos: [0.0, 0.0].into(),
                        radius: 50.0,
                    },
                    closed: 250.0,
                },
            ],
            red_zones: vec![],
            tutorial: true,

            custom_text: None,

            next_level: Some(Level::second_level),
        }
    }

    pub fn second_level() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into());
        rope.add([0.0, -300.0].into());
        rope.add([10.0, 300.0].into());
        Level {
            init_state: rope,
            gravity: v2![0.0, 1000.0],
            targets: vec![
                Target {
                    zone: Circle {
                        pos: [0.0, 0.0].into(),
                        radius: 50.0,
                    },
                    closed: 250.0,
                },
            ],
            red_zones: vec![],
            tutorial: false,

            custom_text: Some("you're not limited to two sticks"),

            next_level: Some(Level::third_level),
        }
    }

    pub fn third_level() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into());
        rope.add([0.0, -300.0].into());
        Level {
            init_state: rope,
            gravity: v2![0.0, 1000.0],
            targets: vec![
                Target {
                    zone: Circle {
                        pos: [0.0, 0.0].into(),
                        radius: 50.0,
                    },
                    closed: 250.0,
                },
                Target {
                    zone: Circle {
                        pos: [550.0, 0.0].into(),
                        radius: 100.0,
                    },
                    closed: 100.0,
                },
            ],
            red_zones: vec![],
            tutorial: false,

            custom_text: Some("soft retries with 'r' lead to win more often than you'd think"),

            next_level: Some(Level::fourth_level),
        }
    }

    pub fn fourth_level() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into());
        rope.add([0.0, -300.0].into());
        Level {
            init_state: rope,
            gravity: v2![0.0, 1000.0],
            targets: vec![
                Target {
                    zone: Circle {
                        pos: [-550.0, 0.0].into(),
                        radius: 100.0,
                    },
                    closed: 100.0,
                },
                Target {
                    zone: Circle {
                        pos: [550.0, 0.0].into(),
                        radius: 100.0,
                    },
                    closed: 100.0,
                },
            ],
            red_zones: vec![
                Circle {
                    pos: [-550.0, -500.0].into(),
                    radius: 300.0,
                },
                Circle {
                    pos: [550.0, -500.0].into(),
                    radius: 300.0,
                },
                Circle {
                    pos: [-550.0, 500.0].into(),
                    radius: 300.0,
                },
                Circle {
                    pos: [550.0, 500.0].into(),
                    radius: 300.0,
                },
            ],
            tutorial: false,

            custom_text: Some("you can skip this easy level through the settings ->"),

            next_level: Some(Level::fifth_level),
        }
    }

    pub fn fifth_level() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into());
        rope.add([0.0, -200.0].into());
        Level {
            init_state: rope,
            gravity: v2![0.0, 1000.0],
            targets: vec![
                Target {
                    zone: Circle {
                        pos: [550.0 - 150.0, -500.0 + 150.0].into(),
                        radius: 50.0,
                    },
                    closed: 40.0,
                },
                Target {
                    zone: Circle {
                        pos: [-550.0 + 150.0, 500.0 - 150.0].into(),
                        radius: 50.0,
                    },
                    closed: 40.0,
                },
                Target {
                    zone: Circle {
                        pos: [650.0 - 150.0, 600.0 - 150.0].into(),
                        radius: 50.0,
                    },
                    closed: 40.0,
                },
            ],
            red_zones: vec![
                Circle {
                    pos: [-650.0 + 150.0, -600.0 + 150.0].into(),
                    radius: 90.0,
                }
            ],
            tutorial: false,

            custom_text: None,

            next_level: Some(Level::sixth_level),
        }
    }

    pub fn sixth_level() -> Level {
        let mut rope = Rope::new([0.0, 0.0].into());
        rope.add([0.0, -200.0].into());
        Level {
            init_state: rope,
            gravity: v2![1000.0 / std::f64::consts::SQRT_2, 1000.0 / std::f64::consts::SQRT_2],
            targets: vec![
                Target {
                    zone: Circle {
                        pos: [-500.0 + 150.0, -500.0 + 150.0].into(),
                        radius: 50.0,
                    },
                    closed: 250.0,
                },
            ],
            red_zones: vec![],
            tutorial: false,

            custom_text: Some("watch your step, gravity is weird"),

            next_level: None,
        }
    }
}

impl MainGame {
    pub fn new(level: Level, game: &mut ChaosTheory) -> Self {
        Self {
            rope: level.init_state.clone(),
            sim_status: SimStatus::Setup,
            win_status: WinStatus::NotYet,
            prev_trails: VecDeque::new(),
            trail: VecDeque::new(),
            creating: None,
            touched_targets: vec![0; level.targets.len()].into_boxed_slice(),
            touching_target: None,

            next_level_button: game.button(""),

            menu_shown: false,
            menu_hovered: false,

            sound_button: game.button("").with_size(1.2),
            music_button: game.button("").with_size(1.2),
            skip_button: game.button("").with_size(1.2),
            tutorial_button: game.button("").with_size(1.2),

            tutorial: level.tutorial.then(|| Tutorial::new(game)),

            level,
            next_level: None,
        }
    }
}

const BG_COLOR: &str = "black";
const BG_LINE_COLOR: &str = "#333040";
const TARGET_COLOR: &str = "#183769";
const BONUS_COLOR: &str = "#ffdf00";
const DANGER_COLOR: &str = "#730c05";

fn draw_background(context: &Context<ChaosTheory>, spacing: f64) {
    let size = context.surface().size();
    let half_size = size / 2.0;
    let surface = context.surface().context();

    surface.fill_color(BG_COLOR);
    surface.fill_rect(-size.x / 2.0, -size.y / 2.0, size.x, size.y);

    surface.stroke_color(BG_LINE_COLOR);
    surface.set_line_width(1.0);

    let mut i = half_size.x % spacing - half_size.x;
    while i < size.x {
        surface.line(v2![i, -half_size.y], v2![i, half_size.y]);
        i += spacing;
    }
    i = half_size.y % spacing - half_size.y;
    while i < size.y {
        surface.line(v2![-half_size.x, i], v2![half_size.x, i]);
        i += spacing;
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
        match std::mem::replace(&mut self.sim_status, SimStatus::Setup) {
            SimStatus::Running { setup } | SimStatus::Paused { setup } if soft => {
                self.rope = setup.clone();
                self.rope.jiggle();
                self.prev_trails.push_back(std::mem::take(&mut self.trail));
                if self.prev_trails.len() > 127 {
                    self.prev_trails.pop_front();
                }
                self.sim_status = SimStatus::Running { setup }
            }
            _ => {
                self.rope = self.level.init_state.clone();
                self.trail.clear();
                self.prev_trails.clear();
            }
        }
    }

    fn pause(&mut self) {
        self.sim_status = match std::mem::replace(&mut self.sim_status, SimStatus::Setup) {
            SimStatus::Setup => SimStatus::Setup,
            SimStatus::Running { setup } | SimStatus::Paused { setup } => SimStatus::Paused { setup },
        }
    }
}

impl GameState<ChaosTheory> for MainGame {
    fn on_pushed(&mut self, context: &mut Context<ChaosTheory>) -> StateTransition<ChaosTheory> {
        context.sound_context_mut().sound_mask = context.storage().get_enabled_sounds();
        if context.storage().passed_tutorial {
            self.tutorial = None;
        }
        StateTransition::None
    }


    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<ChaosTheory>,
    ) -> StateTransition<ChaosTheory> {
        if let Some(tutorial) = &mut self.tutorial {
            if !tutorial.on_event(&event, context) {
                return StateTransition::None;
            }
        }
        self.next_level_button.text.pos /= 0.666;
        self.skip_button.text.pos /= 0.666;
        self.sound_button.text.pos /= 0.666;
        self.music_button.text.pos /= 0.666;
        self.tutorial_button.text.pos /= 0.666;
        if self.next_level_button.on_event(&event, context) || self.skip_button.on_event(&event, context) {
            self.next_level = self.level.next_level.map(|f| f());
            return StateTransition::Pop;
        } else if self.sound_button.on_event(&event, context) {
            let data = context.storage().clone();
            context.sound_context_mut().sound_mask.set(0, !data.sounds_enabled);
            context.set_storage(StoredData {
                sounds_enabled: !data.sounds_enabled,
                ..data
            });
            return StateTransition::None;
        } else if self.music_button.on_event(&event, context) {
            let data = context.storage().clone();
            context.sound_context_mut().sound_mask.set(1, !data.music_enabled);
            let bg = &context.game.background;
            if data.music_enabled {
                bg.stop();
            } else {
                bg.play_unique();
            }
            context.set_storage(StoredData {
                music_enabled: !data.music_enabled,
                ..data
            });
            return StateTransition::None;
        } else if self.tutorial_button.on_event(&event, context) {
            let data = context.storage().clone();
            context.set_storage(StoredData {
                passed_tutorial: false,
                ..data
            });
            self.next_level = Some(Level::tutorial_level());
            return StateTransition::Pop;
        }
        fn in_menu_button(pos: V2, size: V2) -> bool {
            let right = size.x / 2.0;
            let top = -size.y / 2.0;
            let x1 = right - 60.0;
            let x2 = right - 20.0;
            let y1 = top + 20.0;
            let y2 = top + 70.0;
            pos.x * 0.666 > x1 && pos.x * 0.666 < x2 && pos.y * 0.666 > y1 && pos.y * 0.666 < y2
        }
        match event {
            MouseDown { pos, .. } if matches!(self.sim_status, SimStatus::Setup) => {
                if (self.rope.tail() - pos).magnitude() < 15.0 {
                    self.creating = Some(pos)
                }
            }
            MouseMove { pos, .. } => {
                if self.creating.is_some() {
                    self.creating = Some(self.constrain(pos))
                } else if in_menu_button(pos, context.surface().size()) {
                    if !self.menu_hovered {
                        self.menu_hovered = true;
                        context.game.hover.play_unique();
                    }
                } else {
                    self.menu_hovered = false;
                }
            }
            MouseUp { pos, button: MouseButton::Left } => {
                if self.creating.is_some() {
                    self.rope.add(self.constrain(pos));
                    self.creating = None;
                } else if in_menu_button(pos, context.surface().size()) {
                    self.menu_shown = !self.menu_shown;
                    if !self.menu_shown {
                        self.music_button.set_text("");
                        self.sound_button.set_text("");
                        self.skip_button.set_text("");
                        self.tutorial_button.set_text("");
                    }
                    context.game.click.play_unique();
                }
            }
            KeyUp { code: 32, .. } => self.sim_status = match std::mem::replace(&mut self.sim_status, SimStatus::Setup) {
                SimStatus::Setup => {
                    if self.creating.is_none() {
                        let setup = self.rope.clone();
                        self.rope.jiggle();
                        SimStatus::Running { setup }
                    } else {
                        SimStatus::Setup
                    }
                }
                SimStatus::Running { setup } => SimStatus::Paused { setup },
                SimStatus::Paused { setup } => SimStatus::Running { setup }
            },
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

        let scale_fix = 0.666;

        let surface = context.surface().context();

        draw_background(context, 100.0 * scale_fix);

        surface.scale(scale_fix, scale_fix).unwrap();

        context.game.background.play_unique();

        let tail = self.rope.tail();

        if matches!(self.sim_status, SimStatus::Running { .. }) {
            let mut delta_time = context.delta_time();

            if delta_time > 0.05 {
                log::warn!("delta time is > 50ms");
                delta_time = 0.0;
            }

            self.rope.simulate(self.level.gravity, delta_time, 15);

            self.trail.push_back(tail);
            if self.trail.len() > 60 * 10 {
                self.trail.pop_front();
            }
        }

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
                    context.game.target_hit.play();
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

        surface.scale(1.0 / scale_fix, 1.0 / scale_fix).unwrap();

        let right = size.x / 2.0;
        let top = -size.y / 2.0;

        if let Some(title) = self.level.custom_text {
            surface.fill_color("white");
            surface.set_font("1.5rem monospace");
            surface.fill_text(title, 0.0, top + context.rem_to_px(1.0)).unwrap();
        }

        if let WinStatus::Won { bonuses } = self.win_status {
            surface.fill_color("white");
            surface.set_font("2.5rem monospace");
            let text =
                if bonuses == 0 {
                    Cow::Borrowed("You win")
                } else {
                    Cow::Owned(format!("You win (+{})", bonuses))
                };
            surface.fill_text(text.as_ref(), 0.0, top + context.rem_to_px(2.5)).unwrap();

            if self.level.next_level.is_none() {
                surface.fill_text("That's all there is for now 🤷", 0.0, top + context.rem_to_px(5.0)).unwrap();
                self.next_level_button.set_text("");
            } else {
                self.next_level_button.set_text("next level");
            }
            self.next_level_button.on_update(context, v2![0.0, top + context.rem_to_px(2.5) + context.rem_to_px(1.6)]);
        }

        surface.fill_color(if self.menu_hovered { HOVER_COLOR } else { BUTTON_COLOR });

        surface.fill_rect(right - 60.0, top + 20.0, 40.0, 7.0);
        surface.fill_rect(right - 60.0, top + 35.0, 40.0, 7.0);
        surface.fill_rect(right - 60.0, top + 50.0, 40.0, 7.0);

        if self.menu_shown {
            let s = context.storage();

            self.music_button.set_text(if s.music_enabled {
                "Music: On"
            } else {
                "Music: Off"
            });
            self.sound_button.set_text(if s.sounds_enabled {
                "Sounds: On"
            } else {
                "Sounds: Off"
            });
            self.skip_button.set_text("Skip level");
            self.tutorial_button.set_text("Replay tutorial");

            let (music_button_width, _) = self.music_button.text.compute_size(context);
            let (sound_button_width, _) = self.sound_button.text.compute_size(context);
            let (skip_button_width, _) = self.skip_button.text.compute_size(context);
            let (tutorial_button_width, _) = self.tutorial_button.text.compute_size(context);

            let top = top + 100.0;
            let right = right - 20.0;
            self.music_button.on_update(context, v2![right - music_button_width / 2.0, top]);
            self.sound_button.on_update(context, v2![right - sound_button_width / 2.0, top + context.rem_to_px(1.3)]);
            self.skip_button.on_update(context, v2![right - skip_button_width / 2.0, top + context.rem_to_px(2.6)]);
            self.tutorial_button.on_update(context, v2![right - tutorial_button_width / 2.0, top + context.rem_to_px(3.9)]);
        }

        if let Some(tutorial) = &mut self.tutorial {
            if tutorial.on_update(context, matches!(self.win_status, WinStatus::Won {..})) {
                self.pause()
            }
        }
        surface.scale(scale_fix, scale_fix).unwrap();

        if let Some(pos) = self.creating {
            let tail = self.rope.tail();

            surface.stroke_color("gray");
            surface.fill_color("white");
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

    fn on_popped(self: Box<Self>, _context: &mut Context<ChaosTheory>) -> StateTransition<ChaosTheory> {
        StateTransition::push(MainGame::new(self.next_level.unwrap_or(self.level), _context.game))
    }
}
