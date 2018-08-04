#![feature(async_await, await_macro, pin, nll, futures_api)]

#[macro_use]
extern crate tanya;
extern crate futures;

use std::time::Instant;
use tanya::prelude::*;

fn main() {
    let mut world = World::new();
    world.add_resource::<Vec<u32>>(vec![0, 2, 3, 5]);
    world.add_resource::<u32>(4);

    let mut job_system = JobSystem::new(rayon::ThreadPoolBuilder::new().build().unwrap());

    job_system.scope(|mut jobs| {
        let mut i = 0;

        loop {
            let start = Instant::now();

            let update = {
                let mut frame = FrameBuilder::new(&mut jobs);
                {
                    let game_world = frame.access(&mut world);

                    let elem = game_world.query::<Vec<u32>>();

                    // Frame dependency graph:
                    // * User: B -> C, B -> D
                    // * Data: A -> B
                    spawn_job!(frame, |mut elem| {
                        println!("pre y: { }", elem[0]);
                        for _ in 0..200_000 {}
                        elem[0] = i;
                    });

                    let r = spawn_job!(frame, |ref elem| {
                        // let r = await!(y);
                        println!("post y: {:?}", elem[0]);
                    });

                    let r0 = r.clone();
                    let r1 = r;

                    spawn_job!(frame, || {
                        let r = await!(r0);
                        println!("x0 {:?}", r);
                    });

                    spawn_job!(frame, || {
                        let r = await!(r1);
                        println!("x1 {:?}", r);
                        for _ in 0..199_000 {}
                        println!("x1 end");
                    });
                }

                frame.dispatch()
            };

            jobs.block_on(update);
            println!("{:?}", start.elapsed());

            i += 1;
        }
    });
}
