use std::time::Duration;

use bevy::prelude::Entity;
use bevy::time::common_conditions::on_timer;
use bevy::{log::LogPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

pub const PHYSICS_DT: f32 = 1.0 / 20.0;

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
        .add_systems(Update, (apply_velocities).chain()
            .run_if(on_timer(Duration::from_millis((1000.0 * PHYSICS_DT) as u64))))
        .add_systems(Update, (debug_points, debug_constraints))
        .run();
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Point {
    pub position: Vec2,
    pub velocity: Vec2,
    pub prev_position: Vec2,
    pub radius: f32,
    pub mass: f32,
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
        velocity: Vec2::ZERO,
        prev_position: position,
        radius: 1.0,
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
}

pub fn apply_velocities(mut point_query: Query<&mut Point>) {
    for mut point in point_query.iter_mut() {
        point.prev_position = point.position;
        point.position = point.position + point.velocity;
    }
}

pub fn debug_points(mut gizmos: Gizmos, point_query: Query<&Point>) {
    for point in point_query.iter() {
        gizmos.circle_2d(point.position, point.radius, Color::WHITE);
    }
}

macro_rules! constraint_points {
    ($constraint:ident, $point_query:ident, $point_a:ident, $point_b:ident, $then:block) => {
        let point_a = $point_query.get($constraint.point_a);
        if let Ok($point_a) = point_a {
            let point_b = $point_query.get($constraint.point_b);
            if let Ok($point_b) = point_b $then
        }
    };
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
