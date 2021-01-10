use crate::enemies::{
    build_circle_path, build_quadratic_path, build_triangle_path, Enemy, EnemyColor, EnemyForm,
    Tameable,
};
use crate::map::{Coordinate, Map, Tile};
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{point, FillOptions, LineJoin, PathBuilder, StrokeOptions};
use std::f32::consts::PI;
use rand::random;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(PuzzleIdFactory::default())
            .add_resource(PickSource {
                cursor_offset: Vec2::new(17., -19.),
                ..Default::default()
            })
            .add_resource(CurrentPiece {
                entity: None,
                piece: None,
            })
            .add_resource(Puzzles { new_towers: vec![] })
            .add_startup_system(set_tower_puzzles.system())
            .add_system(pick_up_piece.system()) /*.add_system(show_cursor.system())*/
            .add_system(update_picked_up_piece.system())
            .add_system(update_puzzle_slots.system());
    }
}

#[derive(Default)]
struct PuzzleIdFactory {
    next_id: usize,
}

impl PuzzleIdFactory {
    pub fn get_next_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id - 1
    }
}

#[derive(Clone)]
pub struct PuzzleSlot {
    piece: Piece,
    filled: bool,
    puzzle_id: usize,
}

pub struct CurrentPiece {
    pub entity: Option<Entity>,
    pub piece: Option<Piece>,
}

#[derive(Default)]
pub struct PickSource {
    pub cursor_events: EventReader<CursorMoved>,
    pub last_cursor_pos: Vec2,
    pub cursor_offset: Vec2,
}

pub struct Puzzles {
    new_towers: Vec<Puzzle>,
}

pub struct Puzzle {
    id: usize,
    coordinate: Coordinate,
    pieces: [Piece; 4],
    filled: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    color: EnemyColor,
    form: EnemyForm,
}

struct ToFill;

fn set_tower_puzzles(
    commands: &mut Commands,
    mut puzzle_ids: ResMut<PuzzleIdFactory>,
    mut puzzles: ResMut<Puzzles>,
    map: Res<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut new_tower_positions: Vec<Coordinate> = vec![];
    for (row_index, row) in map.tiles.iter().enumerate() {
        for (column_index, tile) in row.iter().enumerate() {
            if tile == &Tile::TowerPlot {
                new_tower_positions.push(Coordinate {
                    x: column_index as f32 * map.tile_size,
                    y: row_index as f32 * map.tile_size,
                })
            }
        }
    }

    for coordinate in new_tower_positions {
        let id = puzzle_ids.get_next_id();
        let puzzle = Puzzle {
            coordinate: coordinate.clone(),
            filled: 0,
            id,
            pieces: [
                Piece {
                    color: random(),
                    form: random(),
                },
                Piece {
                    color: random(),
                    form: random(),
                },
                Piece {
                    color: random(),
                    form: random(),
                },
                Piece {
                    color: random(),
                    form: random(),
                },
            ],
        };
        for (index, piece) in puzzle.pieces.iter().enumerate() {
            let path = match piece.form {
                EnemyForm::Circle => build_circle_path(),
                EnemyForm::Triangle => build_triangle_path(),
                EnemyForm::Quadratic => build_quadratic_path(),
            };
            let coordinate = match index {
                0 => Coordinate {
                    x: coordinate.x - 16.,
                    y: coordinate.y - 16.,
                },
                1 => Coordinate {
                    x: coordinate.x + 16.,
                    y: coordinate.y - 16.,
                },
                2 => Coordinate {
                    x: coordinate.x + 16.,
                    y: coordinate.y + 16.,
                },
                _ => Coordinate {
                    x: coordinate.x - 16.,
                    y: coordinate.y + 16.,
                },
            };

            commands
                .spawn(
                    path.stroke(
                        materials.add(piece.color.to_color().into()),
                        &mut meshes,
                        Vec3::new(coordinate.x, coordinate.y, 0.),
                        &StrokeOptions::default()
                            .with_line_width(2.)
                            .with_line_join(LineJoin::Round),
                    ),
                )
                .with(PuzzleSlot {
                    piece: piece.clone(),
                    filled: false,
                    puzzle_id: id,
                });
        }

        puzzles.new_towers.push(puzzle);
    }
}

fn update_puzzle_slots(
    commands: &mut Commands,
    mut puzzles: ResMut<Puzzles>,
    query: Query<(Entity, &Transform, &PuzzleSlot), With<ToFill>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, transform, slot) in query.iter() {
        commands.despawn(entity);
        let path = match slot.piece.form {
            EnemyForm::Circle => build_circle_path(),
            EnemyForm::Triangle => build_triangle_path(),
            EnemyForm::Quadratic => build_quadratic_path(),
        };
        commands
            .spawn(path.fill(
                materials.add(slot.piece.color.to_color().into()),
                &mut meshes,
                transform.translation,
                &FillOptions::default(),
            ))
            .with(PuzzleSlot {
                filled: true,
                ..slot.clone()
            });
        let puzzle = puzzles
            .new_towers
            .iter_mut()
            .find(|puzzle| puzzle.id == slot.puzzle_id)
            .unwrap();
        puzzle.filled += 1;
        if puzzle.filled == 4 {
            println!("completed the puzzle!");
        }
    }
}

fn pick_up_piece(
    commands: &mut Commands,
    cursor: Res<Events<CursorMoved>>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut tamable_query: Query<(Entity, &mut Transform, &Enemy), With<Tameable>>,
    mut puzzle_query: Query<(Entity, &Transform, &mut PuzzleSlot)>,
    mut currently_picked: ResMut<CurrentPiece>,
    mut pick_source: ResMut<PickSource>,
) {
    let cursor_position = pick_source.cursor_events.latest(&cursor);
    let cursor_position = if let Some(cursor_position) = cursor_position {
        cursor_position.position - pick_source.cursor_offset
    } else {
        pick_source.last_cursor_pos
    };
    pick_source.last_cursor_pos = cursor_position;
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        if currently_picked.entity.is_none() {
            for (entity, transform, enemy) in tamable_query.iter_mut() {
                if Vec2::new(
                    transform.translation.x - cursor_position.x,
                    transform.translation.y - cursor_position.y,
                )
                .length()
                    < 12.
                {
                    currently_picked.entity = Some(entity);
                    currently_picked.piece = Some(Piece {
                        form: enemy.form.clone(),
                        color: enemy.color.clone(),
                    });
                    return;
                }
            }
        } else {
            // we have a piece, place it in a puzzle or let it go
            let mut found_slot: bool = false;
            for (entity, transform, mut slot) in puzzle_query.iter_mut() {
                if slot.filled
                    || Vec2::new(
                        transform.translation.x - cursor_position.x,
                        transform.translation.y - cursor_position.y,
                    )
                    .length()
                        > 12.
                {
                    continue;
                }
                found_slot = true;
                if &slot.piece == currently_picked.piece.as_ref().unwrap() {
                    let (_, mut tamable_transform, _) = tamable_query
                        .get_mut(currently_picked.entity.unwrap())
                        .unwrap();
                    tamable_transform.translation = transform.translation;
                    commands.insert_one(entity, ToFill);
                    commands.despawn(currently_picked.entity.unwrap());
                    slot.filled = true;
                    currently_picked.entity = None;
                    currently_picked.piece = None;
                    return;
                }
            }
            if !found_slot {
                // go free my friend
                currently_picked.entity = None;
                currently_picked.piece = None;
            }
        }
    }
}

fn show_cursor(
    commands: &mut Commands,
    pick_source: Res<PickSource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut builder = PathBuilder::new();
    builder.arc(point(0.000001, 0.000001), 3., 3., 2. * PI, 0.1);
    let path = builder.build();
    commands.spawn(path.fill(
        materials.add(Color::BLACK.into()),
        &mut meshes,
        Vec3::new(
            pick_source.last_cursor_pos.x,
            pick_source.last_cursor_pos.y,
            10.,
        ),
        &FillOptions::default(),
    ));
}

fn update_picked_up_piece(
    pick_source: Res<PickSource>,
    currently_picked_up: Res<CurrentPiece>,
    mut enemy_query: Query<(&mut Transform), With<Tameable>>,
) {
    if currently_picked_up.entity.is_none() {
        return;
    }
    if let Ok((mut transform)) = enemy_query.get_mut(currently_picked_up.entity.unwrap()) {
        transform.translation = Vec3::new(
            pick_source.last_cursor_pos.x,
            pick_source.last_cursor_pos.y,
            0.,
        );
    }
}
