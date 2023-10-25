use std::time::Duration;

use bevy::prelude::Entity;
use bevy::time::common_conditions::on_timer;
use bevy::window::PrimaryWindow;
use bevy::{log::LogPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use rand::random;

pub const PHYSICS_ITERS: u8 = 4;
pub const PHYSICS_DT: f32 = 1.0 / 60.0;
pub const PHYSICS_ITER_DT: f32 = PHYSICS_DT / (PHYSICS_ITERS as f32);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=off,wgpu_hal=off,bevy_input=off,bevy_render=off,bevy_diagnostic=off".into(),
            level: bevy::log::Level::DEBUG,
        }))
        .add_plugins(WorldInspectorPlugin::new())
        .register_type::<Point>()
        .register_type::<Constraint>()
        .add_systems(Startup, (spawn_entities).chain())
        .add_systems(Update, (apply_velocities, compute_boundaries, compute_constraints, update_positions).chain()
            .run_if(on_timer(Duration::from_millis((1000.0 * PHYSICS_DT) as u64))))
        .add_systems(Update, (debug_points, debug_constraints))
        .run();
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Point {
    pub position: Vec2,
    pub velocity: Vec2,
    pub friction: f32,
    pub radius: f32,
    pub mass: f32,
}

impl Point {
    pub fn future_position(&self) -> Vec2 {
        self.position + self.velocity
    }
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Constraint {
    pub point_a: Entity,
    pub point_b: Entity,
    pub length: f32,
    pub strength: f32,
}

pub fn make_point(position: Vec2) -> impl Bundle {
    Point {
        position,
        velocity: Vec2::new(random::<f32>() - 0.5, random::<f32>() - 0.5).normalize(),
        friction: 1.0,
        radius: 10.0,
        mass: 10.0,
    }
}

pub fn make_constraint(point_a: Entity, point_b: Entity, length: f32) -> impl Bundle {
    Constraint {
        point_a,
        point_b,
        length,
        strength: 1.0,
    }
}

pub fn spawn_entities(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let a = commands.spawn(make_point(Vec2::new(0.0, 0.0))).id();
    let b = commands.spawn(make_point(Vec2::new(100.0, 0.0))).id();
    let c = commands.spawn(make_point(Vec2::new(100.0, 100.0))).id();
    let d = commands.spawn(make_point(Vec2::new(0.0, 100.0))).id();
    commands.spawn(make_constraint(a, b, 100.0));
    commands.spawn(make_constraint(b, c, 100.0));
    commands.spawn(make_constraint(c, d, 100.0));
    commands.spawn(make_constraint(d, a, 100.0));
    commands.spawn(make_constraint(a, c, 100.0 * 2.0_f32.sqrt()));
    commands.spawn(make_constraint(b, d, 100.0 * 2.0_f32.sqrt()));
    let e = commands.spawn(make_point(Vec2::new(0.0, -100.0))).id();
    commands.spawn(make_constraint(a, e, 100.0));


}

macro_rules! constraint_points {
    ($constraint:ident, $point_query:ident, $point_a:ident, $point_b:ident, $then:block) => {
        let result = $point_query.get_many([$constraint.point_a, $constraint.point_b]);
        if let Ok([$point_a, $point_b]) = result $then
    };
}

macro_rules! constraint_points_mut {
    ($constraint:ident, $point_query:ident, $point_a:ident, $point_b:ident, $then:block) => {
        let result = $point_query.get_many_mut([$constraint.point_a, $constraint.point_b]);
        if let Ok([mut $point_a, mut $point_b]) = result $then
    };
}

pub fn apply_velocities(mut point_query: Query<&mut Point>) {
    for mut point in point_query.iter_mut() {
        point.position = point.position + point.velocity;
    }
}

pub fn compute_boundaries(
    mut point_query: Query<&mut Point>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single();
    if let Ok(window) = window {
        for mut point in point_query.iter_mut() {
            let width2 = window.width() / 2.0 - point.radius;
            let height2 = window.height() / 2.0 - point.radius;
            let r = -1.0;
            let mut pos_effect = point.position;
            let mut bounce_x = false;
            let mut bounce_y = false;
            if point.position.x < -width2 {
                pos_effect.x = -width2;
                bounce_x = true;
            }
            if point.position.y < -height2 {
                pos_effect.y = -height2;
                bounce_y = true;
            }
            if point.position.x > width2 {
                pos_effect.x = width2;
                bounce_x = true;
            }
            if point.position.y > height2 {
                pos_effect.y = height2;
                bounce_y = true;
            }
            if bounce_x || bounce_y {
                point.position = pos_effect;
                if bounce_x {
                    point.velocity.x *= r;
                }
                if bounce_y {
                    point.velocity.y *= r;
                }
            }
        }
    }
}

pub fn compute_constraints(
    mut point_query: Query<&mut Point>,
    constraint_query: Query<&Constraint>,
) {
    for _ in 0..PHYSICS_ITERS {
        for constraint in constraint_query.iter() {
            constraint_points_mut!(constraint, point_query, point_a, point_b, {
                let pos_a = point_a.future_position();
                let pos_b = point_b.future_position();
                let delta = pos_b - pos_a;
                let distance = pos_a.distance(pos_b);
                let diff = (constraint.length - distance) / distance * constraint.strength * 2.0;
                let offset = delta * diff * 0.5;
                let effect_a = (1.0 / point_a.mass) / ((1.0 / point_a.mass) + (1.0 / point_b.mass));
                let effect_b = 1.0 - effect_a;

                point_a.velocity -= offset * effect_a;
                point_b.velocity += offset * effect_b;
            });
        }
    }
}

pub fn update_positions(mut point_query: Query<&mut Point>) {
    for mut point in point_query.iter_mut() {
        let velocity = point.velocity * point.friction;
        point.position += velocity;
        point.velocity = velocity;
    }
}

pub fn debug_points(mut gizmos: Gizmos, point_query: Query<&Point>) {
    for point in point_query.iter() {
        gizmos.circle_2d(point.position, point.radius, Color::WHITE);
    }
}

pub fn debug_constraints(
    mut gizmos: Gizmos,
    point_query: Query<&Point>,
    constraint_query: Query<&Constraint>,
) {
    for constraint in constraint_query.iter() {
        constraint_points!(constraint, point_query, point_a, point_b, {
            gizmos.line_2d(point_a.position, point_b.position, Color::RED);
        });
    }
}
