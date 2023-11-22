use std::thread;
use std::time::{Duration, Instant};
use rand::random;
use raylib::prelude::*;

pub struct Camera {
    pub position: Vector2,

    scale: f32,
    scale_v: Vector2,
}

impl Camera {
    pub fn new(position: Vector2, scale: f32) -> Self {
        Self {
            position,
            scale,
            scale_v: Vector2::one() * scale,
        }
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.scale_v = Vector2::new(scale, scale);
    }

    pub fn invert_v(mut self) -> Self {
        self.scale_v *= Vector2::new(1.0, -1.0);
        self
    }

    pub fn invert_h(mut self) -> Self {
        self.scale_v *= Vector2::new(-1.0, 1.0);
        self
    }

    pub fn project(&self, v: Vector2) -> Vector2 {
        (v * self.scale_v) + self.position
    }

    pub fn scale(&self, v: f32) -> f32 {
        v * self.scale
    }
}

#[derive(Default, Copy, Clone)]
pub struct Ball {
    pub id: usize,
    pub center: Vector2,
    pub radius: f32,
    pub mass: f32,
    pub color: Color,
    pub velocity: Vector2,
    pub freezing: i32,
}

const DAMPING: f32 = 1.0;
const FREEZING_THRESHOLD: f32 = 1e-4;
const GRAVITY: Vector2 = Vector2::new(0.0, -980.0);
const FPS_CAP: f32 = 120.0;


impl Ball {
    pub fn new(id: usize, center: Vector2, radius: f32, color: Color) -> Ball {
        Ball {
            id,
            center,
            radius,
            color,
            mass: radius,
            velocity: Vector2::zero(),
            freezing: 10,
        }
    }

    pub fn draw(&self, cam: &Camera, d: &mut RaylibDrawHandle) {
        let center = cam.project(self.center);
        let radius = cam.scale(self.radius);

        d.draw_circle_v(center, radius, self.color);
    }

    fn apply_collision(&mut self, v: Vector2, other: &mut Ball) {
        // static collision
        let half_d = v / 2.0;
        self.center += half_d;
        other.center -= half_d;

        // dynamic collision
        let normal = v.normalized();
        let tangent = Vector2::new(-normal.y, normal.x);

        let dot_tan_self = self.velocity.dot(tangent);
        let dot_tan_other = other.velocity.dot(tangent);

        let dot_normal_self = self.velocity.dot(normal);
        let dot_normal_other = other.velocity.dot(normal);

        let total_mass = self.mass + other.mass;

        let momentum_self = (dot_normal_self * (self.mass - other.mass) + 2.0 * other.mass * dot_normal_other) / total_mass;
        let momentum_other = (dot_normal_other * (other.mass - self.mass) + 2.0 * self.mass * dot_normal_self) / total_mass;

        self.velocity = tangent * dot_tan_self + normal * momentum_self;
        other.velocity = tangent * dot_tan_other + normal * momentum_other;

        if other.freezing < 0 && other.velocity.length() > FREEZING_THRESHOLD {
            other.freezing = 10
        }
    }

    pub fn update(&mut self, dt: f32, balls: &mut [Ball]) {
        if self.freezing < 0 {
            return;
        }

        self.velocity += GRAVITY * dt;
        self.center += self.velocity * dt;

        for ball in balls {
            if self.id == ball.id {
                continue;
            }
            if let Some(v) = self.collides(ball) {
                self.apply_collision(v, ball);
            }
        }

        self.resolve_bounding(0.0, 0.0, 640.0, 480.0);

        if self.velocity.length() < FREEZING_THRESHOLD {
            self.freezing -= 1;
        }
    }

    fn resolve_bounding(&mut self, left: f32, bottom: f32, right: f32, top: f32) {
        let mid = Vector2::new((right + left) / 2.0, (top + bottom) / 2.0);
        let half_bounding_size = Vector2::new(right - left, top - bottom) / 2.0 - Vector2::one() * self.radius;

        let pos = self.center - mid;

        if pos.x.abs() > half_bounding_size.x {
            self.center.x = half_bounding_size.x * pos.x.signum() + mid.x;
            self.velocity.x *= -1.0 * DAMPING;
        }

        if pos.y.abs() > half_bounding_size.y {
            self.center.y = half_bounding_size.y * pos.y.signum() + mid.y;
            self.velocity.y *= -1.0 * DAMPING;
        }
    }

    fn collides(&self, other: &Ball) -> Option<Vector2> {
        let direction = other.center - self.center;
        let intersection = direction.length() - (other.radius + self.radius);
        if intersection > f32::EPSILON {
            None
        } else {
            Some(direction.normalized() * intersection)
        }
    }
}

struct Clock {
    prev_tick: Instant,
    frame_cap: Option<Duration>,
}

impl Clock {
    pub fn new(frame_cap: Option<Duration>) -> Self {
        Self {
            prev_tick: Instant::now(),
            frame_cap,
        }
    }

    pub fn tick(&mut self) -> f32 {
        if let Some(cap) = self.frame_cap {
            self.tick_capped(cap)
        } else {
            self.tick_uncapped()
        }
    }

    pub fn tick_uncapped(&mut self) -> f32 {
        let now = Instant::now();
        let dt = (now - self.prev_tick).as_micros() as f32 / 1e6;
        self.prev_tick = now;
        dt
    }

    pub fn tick_capped(&mut self, cap: Duration) -> f32 {
        let mut now = Instant::now();
        let mut delta = now - self.prev_tick;

        while delta < cap {
            thread::yield_now(); // let os reschedule some other stuff
            thread::sleep(Duration::from_millis(1));

            now = Instant::now();
            delta = now - self.prev_tick;
        }

        self.prev_tick = now;
        (delta.as_micros() as f32) / 1e6
    }
}

fn main() {
    let cam = Camera::new(Vector2::new(0.0, 480.0), 1.0).invert_v();
    let mut balls = Vec::new();

    for _ in 0..5 {
        fn rand_between(min: f32, max: f32) -> f32 {
            min + (max - min) * random::<f32>()
        }

        let radius = rand_between(20.0, 70.0);
        let center = Vector2::new(rand_between(radius, 640.0 - radius), rand_between(radius, 480.0 - radius));
        let color = Color::new(random(), random(), random(), 255);

        balls.push(Ball::new(balls.len(), center, radius, color));
    }

    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Balls")
        .build();

    let frame_cap = if FPS_CAP > 0.0 { Some(Duration::from_micros((1e6 / FPS_CAP) as u64)) } else { None };
    let mut clock = Clock::new(frame_cap);

    while !rl.window_should_close() {
        let dt = clock.tick();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        for i in 0..balls.len() {
            let mut ball = balls[i];

            ball.update(dt, &mut balls);
            ball.draw(&cam, &mut d);

            balls[i] = ball;
        }

        d.draw_text(format!("FPS: {}", (1.0 / dt) as i32).as_str(), 10, 10, 10, Color::RED);
    }
}
