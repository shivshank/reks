# Reks

A Rust Entity Component System (ECS) that will eventually be good.

Right now it's terrible (I think; haven't profiled yet! that's next) although the API has some
nice features that I intend to preserve as development continues, namely:

 - Components are plain structs; you do not need to implement any traits
 - You do not need to register anything, the ECS will figure it out
    - (Maybe I'll add more APIs to do things manually if it can improve perf a lot)
 - Systems are just plain functions
    - (this could change, if it's a major perf blocker, although I don't think it will be)

Specs ECS is a huge inspiration - without it this wouldn't exist.
