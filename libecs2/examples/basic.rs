use tanya_ecs2::*;

#[derive(Copy, Clone, Debug)]
struct Foo {
    a: usize,
}
impl Component for Foo {}

#[derive(Copy, Clone, Debug)]
struct Bar {}
impl Component for Bar {}

fn main() {
    let mut world = World::new();
    world.define_component::<Foo>();
    world.define_component::<Bar>();
    let mut entities = [Entity::INVALID; 8];
    let foo_data = [Foo { a: 4 }; 8];
    let bar_data = [Bar {}; 8];
    world.create_entities::<(Foo, Bar)>(&mut entities, (&foo_data, &bar_data));

    let foo_data = [Foo { a: 8 }; 64];
    let bar_data = [Bar {}; 64];
    world.create_entities::<(Foo, Bar)>(&mut [Entity::INVALID; 64], (&foo_data, &bar_data));

    for (a, b) in world.query_group::<(Foo, Bar)>() {
        println!("{:?}", (a, b));
    }
}
