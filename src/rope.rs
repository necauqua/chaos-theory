use nalgebra::Vector2;
use ld_game_engine::util::Mut;

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pos: Vector2<f64>,
    prev_pos: Vector2<f64>,
    pub locked: bool,
}

impl Point {
    pub fn new(pos: Vector2<f64>) -> Self {
        Self {
            pos,
            prev_pos: pos,
            locked: false,
        }
    }

    pub fn locked(pos: Vector2<f64>) -> Self {
        let mut p = Self::new(pos);
        p.locked = true;
        p
    }

    pub fn pos(&self) -> Vector2<f64> {
        self.pos
    }

    fn step(&mut self, gravity: Vector2<f64>) {
        if self.locked {
            return
        }
        let prev_pos = self.pos;
        self.pos += self.pos - self.prev_pos;
        self.pos += gravity;
        self.prev_pos = prev_pos;
    }
}

#[derive(Debug)]
pub struct Stick {
    pub point_a: Mut<Point>,
    pub point_b: Mut<Point>,
    pub length: f64,
}

impl Rope {
    pub fn new(root: Vector2<f64>) -> Rope {
        Rope {
            root,
            sticks: Vec::new()
        }
    }

    pub fn add(&mut self, point: Vector2<f64>) {
        let point_a = self.tail().unwrap_or_else(|| Mut::new(Point::locked(self.root)));
        let length = (point - point_a.borrow().pos).magnitude();
        self.sticks.push(Stick {
            point_a,
            point_b: Mut::new(Point::new(point)),
            length
        })
    }

    pub fn tail(&self) -> Option<Mut<Point>> {
        self.sticks.last().map(|s| s.point_b.clone())
    }

    pub fn simulate(&mut self, gravity: Vector2<f64>, num_iterations: u32) {
        for stick in &mut self.sticks {
            stick.point_a.borrow_mut().step(gravity);
            stick.point_b.borrow_mut().step(gravity);
        }
        for _ in 0..num_iterations {
            for stick in &mut self.sticks {
                let mut point_a = stick.point_a.borrow_mut();
                let mut point_b = stick.point_b.borrow_mut();
                let center = (point_a.pos + point_b.pos) / 2.0;
                let direction = (point_a.pos - point_b.pos).normalize();
                if !point_a.locked {
                    point_a.pos = center + direction * stick.length / 2.0
                }
                if !point_b.locked {
                    point_b.pos = center - direction * stick.length / 2.0
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Rope {
    pub root: Vector2<f64>,
    pub sticks: Vec<Stick>,
}
