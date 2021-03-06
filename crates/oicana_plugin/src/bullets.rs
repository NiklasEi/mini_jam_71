use crate::enemies::{Enemy, Tameable};
use crate::{AppState, STAGE};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::f32::consts::PI;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_update(STAGE, AppState::InGame, update_bullets.system())
            .on_state_exit(STAGE, AppState::InGame, break_down_bullets.system());
    }
}

pub struct Bullet {
    pub damage: i32,
    pub speed: f32,
    pub target: Entity,
}

fn update_bullets(
    commands: &mut Commands,
    mut bullet_query: Query<(Entity, &Bullet, &mut Transform)>,
    mut enemy_query: Query<(&mut Enemy, &Transform), Without<Tameable>>,
    time: Res<Time>,
) {
    let delta = time.delta().as_secs_f32();
    for (entity, bullet, mut transform) in bullet_query.iter_mut() {
        let target = enemy_query.get_mut(bullet.target);
        if let Ok((mut target, target_transform)) = target {
            let distance = target_transform.translation - transform.translation;
            if distance.length() < bullet.speed * delta {
                target.health -= bullet.damage;
                commands.despawn(entity);
            } else {
                let movement = distance.normalize() * bullet.speed * delta;
                transform.translation += movement;
            }
        } else {
            commands.despawn(entity);
        }
    }
}

pub fn spawn_bullet(
    commands: &mut Commands,
    bullet: Bullet,
    translation: Vec3,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let mut builder = PathBuilder::new();
    builder.arc(Vec2::new(0.001, 0.001), Vec2::new(3.0, 3.0), 2. * PI, 0.0);
    let path = builder.build();
    commands
        .spawn(GeometryBuilder::build_as(
            &path,
            materials.add(ColorMaterial::color(Color::BLACK)),
            TessellationMode::Fill(FillOptions::default()),
            Transform::from_translation(translation),
        ))
        .with(bullet);
}

fn break_down_bullets(commands: &mut Commands, bullets_query: Query<Entity, With<Bullet>>) {
    for entity in bullets_query.iter() {
        commands.despawn(entity);
    }
}
