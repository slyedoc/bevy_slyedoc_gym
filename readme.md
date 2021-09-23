# Bevy Gym

- [Bevy Gym](#bevy-gym)
  - [Getting started](#getting-started)
  - [Overview](#overview)
  - [Models](#models)
  - [Environments](#environments)
    - [Classical](#classical)
    - [Games](#games)
    - [Action Space and Observation Space](#action-space-and-observation-space)
  - [Debugging](#debugging)
  - [Resources](#resources)

For now this is playground as I figure out how I can use tch-rs inside Bevy directly. While python wonderful and all, its the libraries that lets its rule all of machine learning, but I want to use rust.

Ideally with bevy it should be possible to create your own dynamic environments, edit and play them yourself.  Depending on your environments, run them in simulation without rendering anything.

## Getting started

This is not a lib yet, so pull down he repo and run 'cargo run -- --help' to see command line args.

```text
FLAGS:
        --help          Prints help information
    -h, --human         (enable user inputs)
    -s, --simulation    (run in console, no rendering)
    -V, --version       Prints version information

OPTIONS:
    -e, --environment <environment>    [default: acrobot] [possible values: acrobot,
                                     cartpole, mountaincar, pendulum, flappy]

```

I use cargo watch for a fast development cycle, example command:

```bash
cargo watch --clear -x "run --release -- -e flappy -h"
```

## Overview

The gym is broken up over 2 types of plugins: Environments and Models (TODO: need better word for this, agent maybe)  They currently use resources to pass information back and forth, (this design will need to change to allow more than one agent in an environment, events maybe).

Each "Step", is basically a loop over 3 phases controlled by system execution order in bevy:

- update_state: (preformed by Env)  Update resource EnvState with current state of environment
- update_action: (preformed by Model) Update resource EnvAction, and preform any training unique to that model
- take_action: (preformed by Env) Use EnvAction to preform action in Env

**Note**: Tch-rs is not thread safe, we have to limit bevy in how it can access tch-rs
Will be using bevy non_send resources and [exclusive_system](https://github.com/bevyengine/bevy/blob/main/examples/ecs/ecs_guide.rs)

## Models

- Policy Gradient(pg) - from [tch-rs](https://github.com/LaurentMazare/tch-rs/blob/master/examples/reinforcement-learning/policy_gradient.rs)
- Neat - in progress
- PPO - in progress

## Environments

Below are the current environments, still work in progress.  Will try to mark the models currently working with each.

### Classical

- Acrobot (human)
- Cartpole (human, pg)
- Mountain Car - in progress, forked bevy_rapier and added support for polyline debug rendering for the ground, still need to work on the car
- Pendulum (human)

### Games

- Flappy Bird (human) - in progress

### Action Space and Observation Space

Currently each environment sets a resource to let the model know what its action space is.

Its doesn't really work that well because each model then need to handle that and its really only option is to panic.

It looks to a common pain point with other gyms, looks to by why almost every example is hard coded to its example.  Will keep thinking about this.

## Debugging

While tch-rs works out of the box, if you want to attach a debugger it takes a bit more setup.

See [libtorch setup](https://github.com/LaurentMazare/tch-rs#libtorch-manual-install), that will lead you to [pytorch](https://pytorch.org/get-started/locally/)

Download whatever version you need, for myself I wanted cuda 11.1 support so:
    Stable (1.9.0) > Linux > LibTorch > C++/Java > CUDA 11.1

If your new to setting up CUDA, off to google with you, your going to be gone a while, good luck!

You will need to set LIBTORCH env var otherwise tch-rs will download a precompiled version, this is my setup.

```bash
export LD_LIBRARY_PATH=/usr/local/cuda/lib64:/usr/local/cuda/extras/CUPTI/lib64
export LIBTORCH=/{YOUR-PATH}/libtorch-cxx11-abi-shared-with-deps-1.9.0+cu111/libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
export TORCH_CUDA_VERSION="cu111"
```

## Resources

- [Bevy](https://github.com/bevyengine/bevy)
- [Torch - A GPU-Ready Tensor Library](https://github.com/pytorch/pytorch#from-source)
- [NVidia Cuda 11.1](https://developer.nvidia.com/cuda-11.1.0-download-archive?target_os=Linux&target_arch=x86_64&target_distro=Ubuntu&target_version=2004&target_type=deblocal)
