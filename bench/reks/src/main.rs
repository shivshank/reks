extern crate common;

use common::prelude::*;

use common::create_world;

fn main() {
    let mut world = create_world();

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
