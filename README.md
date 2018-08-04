# Tanya

### nightly-only library

A job based multithreaded execution library tailored towards game development.

The job system allows async execution (`futures`) of small tasks on top of the `rayon`s threadpool.
On the top-level we split a main loop iteration into smaller `Frame`s. Each frame consists of a bunch of smaller async jobs.
With this split we can achieve dependencies across multiple timesteps:
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

## TODO
- [ ] API design iterations
- [ ] Complexer example
- [ ] Profiler integration
- [ ] Frame-Frame dependencies
