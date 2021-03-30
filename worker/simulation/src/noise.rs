pub use perlin::PerlinNoise;

mod perlin {
    use crate::{indices::WorldPosition, prelude::Axial};

    use rand::{prelude::SliceRandom, rngs::SmallRng, SeedableRng};

    pub struct PerlinNoise {
        seed: u64,
        permutations: Box<[u32; 512]>,
    }

    impl PerlinNoise {
        pub fn seed(&self) -> u64 {
            self.seed
        }

        pub fn new(seed: impl Into<Option<u64>>) -> Self {
            let seed = seed.into().unwrap_or(0xdeadbeef);
            let mut res = Self {
                seed,
                permutations: Box::new([0; 512]),
            };
            res.reseed(seed);
            res
        }

        pub fn reseed(&mut self, seed: u64) {
            self.seed = seed;
            for i in 0..256u32 {
                self.permutations[i as usize] = i;
            }
            let mut rng = SmallRng::seed_from_u64(seed);

            self.permutations[0..256].shuffle(&mut rng);

            for i in 0..256 {
                self.permutations[i + 256] = self.permutations[i];
            }
        }

        pub fn axial_perlin(&self, pos: Axial, room_size: f32) -> f32 {
            let [x, y] = pos.to_pixel_pointy(1.0);

            let [x, y] = [x / room_size, y / room_size];

            self.perlin(x, y, 0.0)
        }

        pub fn world_perlin(&self, pos: WorldPosition, room_size: f32) -> f32 {
            let WorldPosition { room, pos } = pos;

            let [_, _, z] = pos.hex_axial_to_cube();
            let z = z as f32;

            let [x, y] = pos.to_pixel_pointy(4.0);
            let [rx, ry] = room.to_pixel_pointy(room_size * 8.0);

            let [x, y] = [rx + x, ry + y];

            self.perlin(x, y, z)
        }

        pub fn perlin(&self, x: f32, y: f32, z: f32) -> f32 {
            let x0 = x as u32 & 255;
            let y0 = y as u32 & 255;
            let z0 = z as u32 & 255;

            let x = x.fract();
            let y = y.fract();
            let z = z.fract();

            let u = fade(x);
            let v = fade(y);
            let w = fade(z);

            let a = self.permutations[x0 as usize] + y0;
            let aa = self.permutations[a as usize] + z0;
            let ab = self.permutations[a as usize + 1] + z0;
            let b = self.permutations[x0 as usize + 1] + y0;
            let ba = self.permutations[b as usize] + z0;
            let bb = self.permutations[b as usize + 1] + z0;

            interpolate(
                interpolate(
                    interpolate(
                        grad(self.permutations[aa as usize], x, y, z),
                        grad(self.permutations[ba as usize], x - 1.0, y, z),
                        u,
                    ),
                    interpolate(
                        grad(self.permutations[ab as usize], x, y - 1.0, z),
                        grad(self.permutations[bb as usize], x - 1.0, y - 1.0, z),
                        u,
                    ),
                    v,
                ),
                interpolate(
                    interpolate(
                        grad(self.permutations[aa as usize + 1], x, y, z - 1.0),
                        grad(self.permutations[ba as usize + 1], x - 1.0, y, z - 1.0),
                        u,
                    ),
                    interpolate(
                        grad(self.permutations[ab as usize + 1], x, y - 1.0, z - 1.0),
                        grad(
                            self.permutations[bb as usize + 1],
                            x - 1.0,
                            y - 1.0,
                            z - 1.0,
                        ),
                        u,
                    ),
                    v,
                ),
                w,
            )
        }
    }

    fn grad(hash: u32, x: f32, y: f32, z: f32) -> f32 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 {
            y
        } else if h == 12 || h == 14 {
            x
        } else {
            z
        };

        let a = if h & 1 == 0 { u } else { -u };
        let b = if h & 2 == 0 { v } else { -v };

        a + b
    }

    fn interpolate(a0: f32, a1: f32, w: f32) -> f32 {
        (a1 - a0) * w + a0
    }

    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }
}
