# Bevy Gym

This is playground as I figure out how I can use tch-rs inside Bevy directly.

While python wonderful and all, its the libaries that lets its rule all of machine learning and I want to use rust.

This is based mainly on tch-rs examples and of course OpenAI Gym

Because tch-rs is not thread safe, we have to limit bevy in how it can access tch-rs
Will be using bevy non_send resources and exclusive_system [thread_local_system](https://github.com/bevyengine/bevy/blob/main/examples/ecs/ecs_guide.rs)

> Note: This is my second attempt at a gym using bevy.  My first attempt to use tch-rs had me controlling bevy from outside though schedual runner and poking it with a stick to update.  It was not pretty and adding completity from making the environments and models enterchangable made it a dumpster fire, so far this is going far better.
