use crate::indices::WorldPosition;

pub fn world_perlin(pos: WorldPosition, room_size: f32) -> f32 {
    let WorldPosition { room, pos } = pos;

    let [x, y] = pos.to_pixel_pointy(1.0);
    let [rx, ry] = room.to_pixel_pointy(room_size);

    let [x, y] = [rx + x, ry + y];

    perlin(x, y)
}

pub fn perlin(x: f32, y: f32) -> f32 {
    use self::perlin::*;

    let x0 = x as i32;
    let x1 = x0 + 1;
    let y0 = y as i32;
    let y1 = y0 + 1;

    let sx = x - x0 as f32;
    let sy = y - y0 as f32;

    let n0 = dot_grid_gradient(x0, y0, x, y);
    let n1 = dot_grid_gradient(x1, y0, x, y);
    let ix0 = interpolate(n0, n1, sx);

    let n0 = dot_grid_gradient(x0, y1, x, y);
    let n1 = dot_grid_gradient(x1, y1, x, y);
    let ix1 = interpolate(n0, n1, sx);

    interpolate(ix0, ix1, sy)
}

mod perlin {

    pub fn dot_grid_gradient(ix: i32, iy: i32, x: f32, y: f32) -> f32 {
        let [gx, gy] = random_gradient(ix, iy);

        let dx = x - ix as f32;
        let dy = y - iy as f32;

        dx * gx + dy * gy
    }

    pub fn random_gradient(ix: i32, iy: i32) -> [f32; 2] {
        let random = 2920.0
            * (ix as f32 * 21942.0 + iy as f32 * 171324.0 + 8912.0).sin()
            * (ix as f32 * 23157.0 * iy as f32 * 217832.0 + 9758.0).cos();
        [random.cos(), random.sin()]
    }

    pub fn interpolate(a0: f32, a1: f32, w: f32) -> f32 {
        (a1 - a0) * (3.0 - w * 2.0) * w * w + a0
    }
}
