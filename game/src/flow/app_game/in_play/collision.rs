use crate::{
    components::{CollidedEvent, Explosion, Invisible, Player, Spaceship, UFO},
    flow::app_game::triggers::{HealthReduceEvent, RemoveUFOEvent},
};
use bevy::prelude::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_collisions);
    }
}

pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollidedEvent>,
    ufo_q: Query<&UFO>,
    spaceship_q: Query<&Player, With<Spaceship>>,
) {
    for collision in collision_events.read() {
        let entity1 = collision.entity1;
        let entity2 = collision.entity2;

        // ufo-spaceship collision
        if let Ok(ufo) = ufo_q.get(entity1) {
            if let Ok(player) = spaceship_q.get(entity2) {
                return handle_ufo_spaceship_collision(
                    commands.reborrow(),
                    player,
                    entity2,
                    ufo,
                    entity1,
                );
            }
        }
        if let Ok(player) = spaceship_q.get(entity1) {
            if let Ok(ufo) = ufo_q.get(entity2) {
                return handle_ufo_spaceship_collision(
                    commands.reborrow(),
                    player,
                    entity1,
                    ufo,
                    entity2,
                );
            }
        }
    }
}

fn handle_ufo_spaceship_collision(
    mut commands: Commands,
    player: &Player,
    player_entity: Entity,
    ufo: &UFO,
    ufo_entity: Entity,
) {
    commands.trigger(HealthReduceEvent::new(player.0));
    commands.entity(player_entity).insert(Invisible::new());
    commands.spawn(Explosion::new(ufo.get_position()));
    commands.trigger(RemoveUFOEvent::clean_up(ufo_entity));
}
