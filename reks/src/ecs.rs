use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

pub struct World {
    components: HashMap<TypeId, Box<Any>>,
    entities: Vec<Vec<(TypeId, usize)>>,
}

pub struct EntityBuilder<'a> {
	world: &'a mut World,
    new_entity: Vec<(TypeId, usize)>,
}

/// A type required in order to execute a system against the ECS world.
///
/// # Safety
///
/// The trait implementor is responsible for guaranteeing that the user is able to reason about
/// what is actually being accessed by `indices`.
///
/// For example, if `SystemReq` is impl for `Foo` and the user has indices into a store of `Foo`,
/// `fetch` needs to actually use those indices to access a storage of `Foo`s. Otherwise the user
/// cannot actually declare a call to `fetch` safe, even if they verify its requirements (see the
/// method for details).
///
/// A bad impl of SystemReq could, for instance, always access a storage of
/// `Bar` in the world, even though Self is `Foo`. Don't do that.
///
/// The unsafety of the trait is especially important for more complex structs and tuples, as these
/// rely on correct implementations of fetch.
pub unsafe trait SystemReq<'a> {
    fn c_set(store: &mut Vec<TypeId>);

    /// It is critically important that the order of indices corresponds to the order of the tuple,
    /// if the thing that we are fetching is indeed a tuple.
    ///
    /// # Safety
    ///
    /// The caller is responsible for guaranteeing indices that are less than the length of any
    /// accessed underlying storages, otherwise the result is Undefined Behavior.
    ///
    /// An impl of SystemReq should be assumed to potentially do unchecked accessess on the
    /// underlying storage.
    ///
    /// The caller must also ensure borrowing rules at runtime as this method cannot be expected
    /// to borrow check the references it returns (i.e., it may return multiple mutable aliases).
    unsafe fn fetch<'b: 'a>(world: &'b World, indices: &[usize]) -> Self;
}

impl World {
    pub fn new() -> Self {
        World {
            components: HashMap::new(),
            entities: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
		EntityBuilder {
			world: self,
            new_entity: Vec::new(),
		}
	}

    pub unsafe fn execute<'a, 'b, T, F>(
        &'b mut self, 
        mut f: F
    )
        where 'b: 'a,
            T: SystemReq<'a>,
            F: FnMut(T)
    {
        let mut component_set = Vec::new();
        T::c_set(&mut component_set);

        let mut indices = Vec::with_capacity(component_set.len());
        let mut e_c_set = HashSet::new();
        for entity in &self.entities {
            // TODO: Compare building the HashSet for each entity to m*n linear search...
            e_c_set.clear();
            for c in entity {
                e_c_set.insert(c.0);
            }
            // check if this entity has every component that we need
            let mut do_continue = false;
            indices.clear();
            for c in &component_set {
                if !e_c_set.contains(c) {
                    do_continue = true;
                    break;
                }
                for (entity_c_type, entity_c_index) in entity {
                    if entity_c_type == c {
                        indices.push(*entity_c_index);
                        break;
                    }
                }
            }
            if do_continue {
                continue;
            }
            // else: this entity must have everything we need
            // TODO: Leave this unsafe block; `execute` should be a safe method, fix that!
            // (currently the user can violate aliasing rules depending on their SystemReqs)
            // (we need to do a mini borrowck on the components)
            unsafe {
                let data = T::fetch(self, &indices);
                f(data);
            }
        }
    }

    pub fn omg_dont_call_this_print_components<C: ::std::fmt::Debug + 'static>(&self) {
        let storage: &Vec<C> = self.components.get(&TypeId::of::<C>())
            .unwrap()
            .downcast_ref()
            .unwrap();
        for c in storage {
            print!("{:?} ", c);
        }
        println!();
    }
}

impl<'a> EntityBuilder<'a> {
	pub fn with<C: 'static>(mut self, component: C) -> EntityBuilder<'a> {
        {
            // NLL strikes again
            let type_id = TypeId::of::<C>();
            let storage: &mut Vec<C> = self.world.components.entry(type_id)
                .or_insert_with(|| -> Box<Any> { Box::new(Vec::<C>::new()) })
                .downcast_mut()
                .unwrap();
            let index = storage.len();
            storage.push(component);
            self.new_entity.push((type_id, index));
        }
        self
    }

    pub fn build(self) {
        let EntityBuilder { new_entity, world } = self;
        world.entities.push(new_entity);
    }
}

unsafe impl<'a, C: 'static> SystemReq<'a> for &'a C {
    fn c_set(store: &mut Vec<TypeId>) {
        store.push(TypeId::of::<C>());
    }

    unsafe fn fetch<'b: 'a>(world: &'b World, indices: &[usize]) -> &'a C {
        debug_assert_eq!(indices.len(), 1);
        let storage: &Vec<C> = world.components.get(&TypeId::of::<C>())
            .expect("Tried to fetch a component that is not used in any entities")
            .downcast_ref()
            .unwrap();
        // NEEDS REVIEW
        // We're not touching the vec - Rust cannot reason about this pointer at all so we'll just
        // cast it how we like - since world has inherited mutablity and is immutable, there is
        // no safe way for anyone else to already have a &mut C
        &*storage.as_ptr().offset(indices[0] as isize)
    }
}

unsafe impl<'a, C: 'static> SystemReq<'a> for &'a mut C {
    fn c_set(store: &mut Vec<TypeId>) {
        store.push(TypeId::of::<C>());
    }

    unsafe fn fetch<'b: 'a>(world: &'b World, indices: &[usize]) -> &'a mut C {
        debug_assert_eq!(indices.len(), 1);
        let storage: &Vec<C> = world.components.get(&TypeId::of::<C>())
            .expect("Tried to fetch a component that is not used in any entities")
            .downcast_ref()
            .unwrap();
        // NEEDS REVIEW
        // We're not touching the vec - Rust cannot reason about this pointer at all so we'll just
        // cast it how we like - since world has inherited mutablity and is immutable, there is
        // no safe way for anyone else to already have a &mut C... However, someone could have
        // &C, but it is within `fetch`es contract that the user is responsible for managing
        // borrowing of the storage's items.
        &mut *(storage.as_ptr().offset(indices[0] as isize) as *mut C)
    }
}

unsafe impl<'a, A, B> SystemReq<'a> for (A, B)
	where A: SystemReq<'a>,
		B: SystemReq<'a>,
{
    fn c_set(store: &mut Vec<TypeId>) {
        A::c_set(store);
        B::c_set(store);
    }

    unsafe fn fetch<'b: 'a>(world: &'b World, indices: &[usize]) -> (A, B) {
        debug_assert_eq!(indices.len(), 2);
        // We have to do this recursively, since &T has a different TypeId of T
        // (we have no way to access the T inside of A, if A is a &T, so to speak, at least the way
        // that these trait bounds are written)
        (
            A::fetch(world, &indices[0..1]),
            B::fetch(world, &indices[1..2]),
        )
    }
}
