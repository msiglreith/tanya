#[derive(Clone, tanya::Component)]
struct Foo;

#[derive(Clone, tanya::Component)]
struct Bar;

#[derive(Clone, tanya::Component)]
struct Baz;

fn main() {
    let mut entities = tanya::ecs::Entities::new();
    let e0 = entities.create_entity::<(Foo, Bar)>((&[Foo], &[Bar]));
}
