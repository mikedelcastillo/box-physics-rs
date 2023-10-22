use bevy::{log::LogPlugin, prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::{Fill, GeometryBuilder, ShapeBundle, ShapePlugin};
use bevy_prototype_lyon::shapes::Circle;

fn main() {
    App::new()
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0))).add_plugins(DefaultPlugins.set(LogPlugin {
        filter: "info,wgpu_core=off,wgpu_hal=off,bevy_input=off,bevy_render=off,bevy_diagnostic=off".into(),
        level: bevy::log::Level::DEBUG,
    }))
    .add_plugins(ShapePlugin).add_systems(Startup, (spawn_camera, spawn_bodies)).run();
}

#[derive(Component, Default)]
pub struct Body {
    pub shape: Shape,
    pub position: Vec2,
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub rotation: f32,
    pub mass: f32,
}

pub enum Shape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

impl Default for Shape {
    fn default() -> Self {
        Self::Circle { radius: 1.0 }
    }
}

pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

pub fn spawn_bodies(commands: Commands) {
    fn spawn_circle(mut commands: Commands, position: Vec2, radius: f32) {
        commands.spawn((
            Body {
                shape: Shape::Circle { radius },
                position,
                ..default()
            },
            ShapeBundle {
                path: GeometryBuilder::build_as(&Circle {
                    radius,
                    ..default()
                }),
                ..default()
            },
            Fill::color(Color::WHITE),
        ));
    }

    spawn_circle(commands, Vec2::ZERO, 25.0);
}
