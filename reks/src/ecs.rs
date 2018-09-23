use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

pub struct World {
    /// Stores a map from TypeId T to boxed Vec<U> where TypeId(U) = T.
    components: HashMap<TypeId, Box<Any>>,
    /// Implmenets a sort of manual borrowchk, similar to ref cell (but clunkier!).
    ///
    /// Used to verify the safety of calls to `fetch` in execute.
    comp_chk: CompChk,
    entities: Vec<Vec<(TypeId, usize)>>,
}

pub struct EntityBuilder<'a> {
	world: &'a mut World,
    new_entity: Vec<(TypeId, usize)>,
}

pub struct CompChk {
    chk: HashMap<TypeId, i32>,
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
    fn c_set(store: &mut Vec<TypeId>, chk: &mut CompChk);

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
            comp_chk: CompChk::new(),
            entities: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
		EntityBuilder {
			world: self,
            new_entity: Vec::new(),
		}
	}

    pub fn execute<'a, 'b, T, F>(
        &'b mut self,
        mut f: F,
    )
        where 'b: 'a,
            T: SystemReq<'a>,
            F: FnMut(T)
    {
        let mut component_set = Vec::new();
        self.comp_chk.return_all();
        T::c_set(&mut component_set, &mut self.comp_chk);

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
            self.world.comp_chk.register::<C>();
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
    fn c_set(store: &mut Vec<TypeId>, chk: &mut CompChk) {
        let t = TypeId::of::<C>();
        chk.borrow(t);
        store.push(t);
    }

    unsafe fn fetch<'b: 'a>(world: &'b World, indices: &[usize]) -> &'a C {
        debug_assert_eq!(indices.len(), 1);
        let storage: &Vec<C> = world.components.get(&TypeId::of::<C>())
            .expect("Tried to fetch a component that is not used in any entities")
            .downcast_ref()
            .unwrap();
        // NEEDS REVIEW
        // `fetch`es contract says that the user is responsible for managing borrowing of the
        // storage's items; we are not doing anything weird with type punning or alignment (yet...)
        // so I think this should be OK
        &*storage.as_ptr().offset(indices[0] as isize)
    }
}

unsafe impl<'a, C: 'static> SystemReq<'a> for &'a mut C {
    fn c_set(store: &mut Vec<TypeId>, chk: &mut CompChk) {
        let t = TypeId::of::<C>();
        chk.borrow_mut(t);
        store.push(t);
    }

    unsafe fn fetch<'b: 'a>(world: &'b World, indices: &[usize]) -> &'a mut C {
        debug_assert_eq!(indices.len(), 1);
        let storage: &Vec<C> = world.components.get(&TypeId::of::<C>())
            .expect("Tried to fetch a component that is not used in any entities")
            .downcast_ref()
            .unwrap();
        // NEEDS REVIEW
        // `fetch`es contract says that the user is responsible for managing borrowing of the
        // storage's items; we are not doing anything weird with type punning or alignment (yet...)
        // so I think this should be OK
        //
        // Since the Vec (RawVec) is really just a raw const ptr (Unique<_>) deep down, I think
        // rust doesn't make any aliasing assumptions wrt mutablility/immutability, so casting
        // as mutable should be OK -- and the caller verifies that this reference is singly
        // aliased, and hence is safe to use as a &mut
        &mut *(storage.as_ptr().offset(indices[0] as isize) as *mut C)
    }
}

macro_rules! impl_SystemReq_for {
    (
        $arg_length:expr, $( $t:ident ; $slice:expr ),+
    ) => {
        // The trailing comma is important, specifically for the case of 1 item tuple (A, )
        unsafe impl<'a, $($t),+> SystemReq<'a> for ($($t),+ , )
            where
                $(
                    $t: SystemReq<'a>
                ),+
        {
            fn c_set(store: &mut Vec<TypeId>, chk: &mut CompChk) {
                $(
                    $t::c_set(store, chk);
                )+
            }

            unsafe fn fetch<'b: 'a>(world: &'b World, indices: &[usize]) -> ($($t),+ , ) {
                debug_assert_eq!(indices.len(), $arg_length);
                (
                    $(
                        $t::fetch(world, &indices[$slice])
                    ),+
                    ,
                )
            }
        }
    }
}

/*unsafe impl<'a, A, B> SystemReq<'a> for (A, B)
	where A: SystemReq<'a>,
		B: SystemReq<'a>,
{
    fn c_set(store: &mut Vec<TypeId>, chk: &mut CompChk) {
        A::c_set(store, chk);
        B::c_set(store, chk);
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
}*/

impl_SystemReq_for!(1, A ; 0..1);
impl_SystemReq_for!(2, A ; 0..1, B ; 1..2);
impl_SystemReq_for!(3, A ; 0..1, B ; 1..2, C ; 2..3);

impl CompChk {
    fn new() -> Self {
        CompChk {
            chk: HashMap::new(),
        }
    }

    fn borrow(&mut self, t: TypeId) {
        let val = self.chk.get_mut(&t)
            .expect("Tried to fetch a component that is not used in any entities");
        if *val < 0 {
            panic!(
                "System attempted to fetch a component immutability while it is already
                 mutably borrowed"
            );
        } else if *val == i32::max_value() {
            panic!(
                "Seriously, mate, what are you doing? You dun overflowed the borrow counter!"
            );
        }
        *val += 1;
    }

    fn borrow_mut(&mut self, t: TypeId) {
        let val = self.chk.get_mut(&t)
            .expect("Tried to fetch a component that is not used in any entities");
        if *val != 0 {
            panic!("System attempted to fetch a component mutably while it is already borrowed")
        }
        *val = -1;
    }

    fn return_all(&mut self) {
        for (_, v) in &mut self.chk {
            *v = 0;
        }
    }

    fn register<C: 'static>(&mut self) {
        self.chk.insert(TypeId::of::<C>(), 0);
    }
}
