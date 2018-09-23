extern crate common;
extern crate reks;

use common::prelude::*;

use reks::World;

fn main() {
    let mut world = create_world();

    let dt = DeltaTime(5.0);
    unsafe {
        world.execute(|(Pos { pos } , Vel { vel }): (&mut Pos, &Vel)| {
            integrate_velocity(pos, vel, dt.0);
        });
    }
}

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
