use bevy_ecs::prelude::*;
use cgmath::*;
use rayon::prelude::*;

#[derive(Copy, Clone)]
struct Position(Vector3<f32>);

#[derive(Copy, Clone)]
struct Rotation(Vector3<f32>);

#[derive(Copy, Clone)]
struct Velocity(Vector3<f32>);

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();

        world.spawn_batch((0..1000).map(|_| {
            (
                Matrix4::<f32>::from_angle_x(Rad(1.2)),
                Position(Vector3::unit_x()),
                Rotation(Vector3::unit_x()),
                Velocity(Vector3::unit_x()),
            )
        }));

        Self(world)
    }

    pub fn run(&mut self) {
        self.0
            .query_batched_mut::<(&mut Position, &mut Matrix4<f32>)>(64)
            .par_bridge()
            .for_each(|batch| {
                for (mut pos, mut mat) in batch {
                    for _ in 0..100 {
                        *mat = mat.invert().unwrap();
                    }

                    pos.0 = mat.transform_vector(pos.0);
                }
            });
    }
}
