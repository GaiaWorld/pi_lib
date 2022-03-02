use pi_ecs::prelude::*;
use cgmath::*;

#[derive(Copy, Clone)]
struct Transform(Matrix4<f32>);

#[derive(Copy, Clone)]
struct Position(Vector3<f32>);

#[derive(Copy, Clone)]
struct Rotation(Vector3<f32>);

#[derive(Copy, Clone)]
struct Velocity(Vector3<f32>);

pub struct Benchmark(World);

pub struct Node;

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::new();
		world.new_archetype::<Node>()
			.add::<Transform>()
			.add::<Position>()
			.add::<Rotation>()
			.add::<Velocity>();
		
		for i in 0..10000 {
			world.spawn::<Node>()
				.insert(Transform(Matrix4::from_scale(1.0))) 
				.insert(Position(Vector3::unit_x())) 
				.insert(Rotation(Vector3::unit_x())) 
				.insert(Velocity(Vector3::unit_x()));
		}

        Self(world)
    }

    pub fn run(&mut self) {
		let mut query = self.0.query::<(&Velocity, &mut Position)>();
        for (velocity, mut position) in query.iter_mut(&mut self.0) {
            position.0 += velocity.0;
        }
    }
}

#[test]
fn tt() {
	let mut world = World::new();
	let i = world.spawn().insert_bundle((
		Transform(Matrix4::from_scale(1.0)),
		Position(Vector3::unit_x()),
		Rotation(Vector3::unit_x()),
		Velocity(Vector3::unit_x()),
	)).id();
	world.spawn_batch((0..10_000).map(|_| {
		(
			Transform(Matrix4::from_scale(1.0)),
			Position(Vector3::unit_x()),
			Rotation(Vector3::unit_x()),
			Velocity(Vector3::unit_x()),
		)
	}));
	
	// let mut query = world.query::<(&mut Velocity, &mut Position)>();
	let mut query = world.query::<(&Velocity, &Position)>();
	let r = query.get(&world, i);
	let r = query.get(&world, i);
	// for (velocity, mut position) in query.iter_mut(&mut world) {
	// 	position.0 += velocity.0;
	// }
}
