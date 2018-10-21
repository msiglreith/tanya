# Tanya (nightly)

Everything here is early and barely useful or finished.

## Features
### `libjobs`
A job based multithreaded execution library.

The job system allows async execution (`futures`) of small tasks on top of `rayon`s threadpool.
On the top-level we split a main loop iteration into smaller `Frame`s. Each frame consists of a bunch of smaller async jobs.
With this split we can achieve dependencies across multiple timesteps (pipelining!):
*
```
loop {
    jobs.block_on(prev_game_update);
    let game_update = {
        let mut frame = FrameBuilder::new(&mut jobs);
        {
            // record updates
        }
        frame.dispatch()
    };

    jobs.block_on(prev_build_render_tree);
    let build_render_tree = {
        let mut frame = FrameBuilder::new(&mut jobs);
        {
            // record game world -> render tree
        }
        frame.dispatch()
    };

    jobs.block_on(prev_render);
    let render = {
        let mut frame = FrameBuilder::new(&mut jobs);
        {
            // record rendering
        }
        frame.dispatch()
    };
}
```

### `libecs`

Entity component system inspired by Unity's ECS approach. Entities are stored in groups depending on their components, trading off memory vs cache locality.

### `libash-vma`
Vulkan Memory ALlocator wrapper on top of `ash`. Will be used with the upcoming `ash` based renderer.
