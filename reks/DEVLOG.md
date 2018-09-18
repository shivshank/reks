# 0.1.0

The first iteration of this ECS will be designed for quick development iteration, with no
regard to performance. The first goal is to create an MVP single-threaded API that compiles,
and each iteration will provide something we can benchmark against.

This implementation is absolutely terrible, and I love it.

Every entity is a list of type ids and indices, and all components are stored in a HashMap
of TypeIds to boxed Anys, which are actually Vec<C>, where C is any component.

Already this API is very nice, though, because you don't have to register components or
implement any traits.

I'll list out optimizations that come to mind; please note that I am most likely only going
to spend time listing out non-obvious optimizations that are likely to be forgotten if I don't
write them down.

Ideas for minor optimizations:

- Switch from slices to pointers and lengths for `fetch`ing components (`SystemReq`)
- Store HashSets with Vecs for each entity's indices.
- Compare HashSet to binary search and linear search for determining the c_set is a subset of
  an entity's components.
- Use a speedy hash map.
