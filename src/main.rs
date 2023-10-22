use std::f32::consts::PI;
use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy::{log::LogPlugin, prelude::*};
use bevy_prototype_lyon::prelude::{Fill, GeometryBuilder, ShapeBundle, ShapePlugin};
use bevy_prototype_lyon::shapes::{Circle, Rectangle, RectangleOrigin};

pub const PHYSICS_DT: f32 = 1.0 / 20.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=off,wgpu_hal=off,bevy_input=off,bevy_render=off,bevy_diagnostic=off".into(),
            level: bevy::log::Level::DEBUG,
        }))
        .add_plugins(ShapePlugin)
        .add_systems(Startup, (spawn_camera, spawn_bodies).chain())
        .add_systems(Update, (apply_velocities, update_transforms).chain()
            .run_if(on_timer(Duration::from_millis((1000.0 * PHYSICS_DT) as u64))))
        .run();
}

#[derive(Component)]
pub struct Body {
    pub shape: Shape,
    pub position: Vec2,
    pub velocity: Vec2,
    pub origin: Vec2,
    pub angular_velocity: f32,
    pub air_friction: f32,
    pub angular_friction: f32,
    pub rotation: f32,
    pub mass: f32,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            shape: Shape::Circle { radius: 1.0 },
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            origin: Vec2::ZERO,
            angular_velocity: 0.0,
            air_friction: 0.95,
            angular_friction: 0.95,
            rotation: 0.0,
            mass: 10.0,
        }
    }
}

pub struct ForceTorque {
    force: Vec2,
    torque: f32,
}

impl Body {
    /// kgm^2
    pub fn moment_of_inertia(&self) -> f32 {
        match &self.shape {
            Shape::Circle { radius } => (self.mass * radius.powi(2)) / 2.0,
            Shape::Rectangle { width, height } => {
                (self.mass / 12.0) * (width.powi(2) + height.powi(2))
            }
        }
    }

    pub fn apply_displacement(&mut self, delta: ForceTorque) {
        let vel = delta.force / self.mass;
        println!("vel {:?}", vel);
        self.velocity += vel;
        self.angular_velocity += delta.torque / self.moment_of_inertia();
    }

    pub fn get_displacement(&self, from: Vec2, force: Vec2) -> ForceTorque {
        let delta = from - self.position;
        let distance = delta.length();
        let torque = delta.perp_dot(force) * distance;
        ForceTorque { force, torque }
    }
}

pub enum Shape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_body(commands: &mut Commands, position: Vec2, shape: Shape) {
    let transform = Transform::from_xyz(position.x, position.y, 0.0);
    let shape_bundle = match &shape {
        Shape::Circle { radius } => ShapeBundle {
            path: GeometryBuilder::build_as(&Circle {
                radius: *radius,
                ..default()
            }),
            transform,
            ..default()
        },
        Shape::Rectangle { width, height } => ShapeBundle {
            path: GeometryBuilder::build_as(&Rectangle {
                extents: Vec2 {
                    x: *width,
                    y: *height,
                },
                origin: RectangleOrigin::Center,
            }),
            transform,
            ..default()
        },
    };
    let mass = match &shape {
        Shape::Circle { radius } => PI * radius.powi(2),
        Shape::Rectangle { width, height } => width * height,
    } / 100.0;
    let mut body = Body {
        shape,
        mass,
        velocity: position / -5.0,
        position,
        ..default()
    };
    let ft = body.get_displacement(Vec2::ZERO, Vec2 { x: 0.0, y: 20.0 });
    match body.shape {
        Shape::Circle { radius } => println!("new circle({})", radius),
        Shape::Rectangle { width, height } => println!("new rectangle({}, {})", width, height),
    }
    println!("mass: {}", body.mass);
    println!("inertia: {}", body.moment_of_inertia());
    println!("{:?} {}", ft.force, ft.torque);
    body.apply_displacement(ft);
    commands.spawn((body, shape_bundle, Fill::color(Color::WHITE)));
}

pub fn spawn_bodies(mut commands: Commands) {
    spawn_body(
        &mut commands,
        Vec2 { x: -100.0, y: 0.0 },
        Shape::Rectangle {
            width: 20.0,
            height: 50.0,
        },
    );
    spawn_body(
        &mut commands,
        Vec2 { x: 100.0, y: 0.0 },
        Shape::Rectangle {
            width: 100.0,
            height: 10.0,
        },
    );
}

pub fn apply_velocities(mut body_query: Query<&mut Body>) {
    for mut body in body_query.iter_mut() {
        body.angular_velocity *= PHYSICS_DT / body.angular_friction;
        body.rotation += body.angular_velocity * PHYSICS_DT;

        // body.velocity = body.velocity * PHYSICS_DT / body.air_friction;
        body.position = body.position + body.velocity * PHYSICS_DT;
    }
}

pub fn update_transforms(mut body_query: Query<(&Body, &mut Transform)>) {
    for (body, mut transform) in body_query.iter_mut() {
        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
        transform.rotate_z(body.rotation);
    }
}
