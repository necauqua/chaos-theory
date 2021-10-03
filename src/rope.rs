use ld_game_engine::util::Mut;
use crate::Pos;

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pos: Pos,
    prev_pos: Pos,
    pub locked: bool,
}

impl Point {
    pub fn new(pos: Pos) -> Self {
        Self {
            pos,
            prev_pos: pos,
            locked: false,
        }
    }

    pub fn locked(pos: Pos) -> Self {
        let mut p = Self::new(pos);
        p.locked = true;
        p
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    fn step(&mut self, accel_with_delta_time_sq: Pos) {
        if self.locked {
            return
        }
        let prev_pos = self.pos;
        self.pos += (self.pos - self.prev_pos) + accel_with_delta_time_sq;
        self.prev_pos = prev_pos;
    }
}

#[derive(Debug)]
pub struct Constraint {
    pub point_a: Mut<Point>,
    pub point_b: Mut<Point>,
    pub length: f64,
}

impl Clone for Constraint {
    fn clone(&self) -> Self {
        Constraint {
            point_a: Mut::new(*self.point_a.borrow()),
            point_b: Mut::new(*self.point_b.borrow()),
            length: self.length
        }
    }
}

impl Constraint {

    fn relax(&mut self) {
        let mut point_a = self.point_a.borrow_mut();
        let mut point_b = self.point_b.borrow_mut();
        let diff = point_a.pos - point_b.pos;
        let direction = diff.normalize();
        let delta_d = diff.magnitude() - self.length;
        if !point_a.locked {
            point_a.pos -= direction * delta_d / 2.0
        }
        if !point_b.locked {
            point_b.pos += direction * delta_d / 2.0
        }
    }
}

#[derive(Debug)]
pub struct Rope {
    pub root: Pos,
    pub constraints: Vec<Constraint>,
    pub gravity: Pos,
}

impl Rope {
    pub fn new(root: Pos, gravity: Pos) -> Rope {
        Rope {
            root,
            constraints: Vec::new(),
            gravity,
        }
    }

    pub fn add(&mut self, point: Pos) {
        let point_a = self.constraints.last().map(|s| s.point_b.clone()).unwrap_or_else(|| Mut::new(Point::locked(self.root)));
        let length = (point - point_a.borrow().pos).magnitude();
        let point_b = Mut::new(Point::new(point));
        self.constraints.push(Constraint {
            point_a,
            point_b,
            length
        })
    }

    pub fn tail(&self) -> Pos {
        self.constraints.last().map(|s| s.point_b.borrow().pos).unwrap_or(self.root)
    }

    pub fn jiggle(&mut self) {
        for constraint in &self.constraints {
            let x = js_sys::Math::random() - 0.5;
            let y = js_sys::Math::random() - 0.5;
            constraint.point_b.borrow_mut().pos += Pos::from([x, y]);
        }
    }

    pub fn simulate(&mut self, delta_time: f64, num_iterations: u32) {
        for constraint in &mut self.constraints {
            let accel = self.gravity * (delta_time * delta_time);
            constraint.point_a.borrow_mut().step(accel);
            constraint.point_b.borrow_mut().step(accel);
        }
        for _ in 0..num_iterations {
            self.constraints.iter_mut().for_each(Constraint::relax);
        }
    }
}

impl Clone for Rope {
    fn clone(&self) -> Self {
        let mut new_rope = Rope::new(self.root, self.gravity);
        for constraint in &self.constraints {
            new_rope.add(constraint.point_b.borrow().pos);
        }
        new_rope
    }
}
