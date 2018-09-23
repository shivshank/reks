extern crate common;

use common::prelude::*;

use common::specs;
use specs::prelude::*;
use specs::World;

fn main() {
    let mut world = create_world();

    let dt = DeltaTime(5.0);
    world.add_resource(dt);
    IntegrateVelocity.run_now(&mut world.res);
}

struct IntegrateVelocity;

impl<'a> System<'a> for IntegrateVelocity {
    type SystemData = (
        Read<'a, DeltaTime>,
        WriteStorage<'a, Pos>,
        ReadStorage<'a, Vel>,
    );

    fn run(&mut self, (
            dt,
            mut pos,
            vel,
        ): Self::SystemData
    ) {
        let DeltaTime(dt) = *dt;

        for (Pos { pos }, Vel { vel }) in (&mut pos, &vel).join() {
            integrate_velocity(pos, vel, dt);
        }
    }
}

fn create_world() -> World {
    let mut world = World::new();

    world.register::<Pos>();
    world.register::<Vel>();

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
