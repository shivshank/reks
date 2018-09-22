use prelude::*;

pub fn create_world() -> World {
    let mut world = World::new();
    world.create_entity()
        .with(Pos { pos: vec3(0.0, 0.0, 0.0) })
        .with(Vel { vel: vec3(1.0, 0.0, 0.0) })
        .build();
    world.create_entity()
        .with(Pos { pos: vec3(1.0, 0.0, 0.0) })
        .with(Vel { vel: vec3(1.0, 0.0, 1.0) })
        .build();
    world
}
