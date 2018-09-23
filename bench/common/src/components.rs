use cgmath::Vector3;

#[cfg(feature="specs_impls")]
use specs::prelude::*;

/// Generates Component impl if specs feature is active
///
/// Makes all fields public.
macro_rules! def_components {
    (
        $(
            $(
                #[$m:meta]
            )*
            pub struct $name:ident {
                $(
                    $(
                        #[$field_meta:meta]
                    )*
                    $field:ident: $kind:ty
                ),*,
            }
        )+
    ) => {
        $(
            $(
                #[$m]
            )*
            pub struct $name {
                $(
                    $(
                        #[$field_meta]
                    )*
                    pub $field: $kind
                ),*,
            }

            #[cfg(feature="specs_impls")]
            impl Component for $name {
                type Storage = VecStorage<Self>;
            }
        )+
    }
}

def_components! {
    #[derive(Debug)]
    pub struct Pos {
        pos: Vector3<f32>,
    }

    #[derive(Debug)]
    pub struct Vel {
        vel: Vector3<f32>,
    }
}
