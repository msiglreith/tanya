#[derive(Clone, tanya::Component)]
struct Foo;

#[derive(Clone, tanya::Component)]
struct Bar;

#[derive(Clone, tanya::Component)]
struct Baz;

fn main() {
    let mut entities = tanya::ecs::Entities::new();
    let mut e0 = [tanya::ecs::Entity::INVALID];
    entities.create_entities::<(Foo, Bar)>(&mut e0, (&[Foo], &[Bar]));
}
