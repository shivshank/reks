extern crate cgmath;

use cgmath::{Vector3, vec3};

mod ecs;

use ecs::World;

#[derive(Debug)]
struct Pos {
    pos: Vector3<f32>,
}

#[derive(Debug)]
struct Vel {
    vel: Vector3<f32>,
}

fn main() {
    let mut world = World::new();
    world.create_entity()
        .with(Pos { pos: vec3(0.0, 0.0, 0.0) })
        .with(Vel { vel: vec3(1.0, 0.0, 0.0) })
        .build();
    world.create_entity()
        .with(Pos { pos: vec3(1.0, 0.0, 0.0) })
        .with(Vel { vel: vec3(1.0, 0.0, 1.0) })
        .build();

    println!("Before execution:");
    world.omg_dont_call_this_print_components::<Pos>();
    world.omg_dont_call_this_print_components::<Vel>();

    let dt = 5.0;
    unsafe {
        world.execute(|(Pos { pos } , Vel { vel }): (&mut Pos, &Vel)| {
            *pos += vel * dt;
        });
    }

    println!("After execution:");
    world.omg_dont_call_this_print_components::<Pos>();
    world.omg_dont_call_this_print_components::<Vel>();
}
