//mod geometry;
use super::geometry::{
    Ray
};

use super::geometry::Hit;

use cgmath::{
    Vector3,
    InnerSpace
};

mod material_utils {
    extern crate rand;

    use rand::{
        Rng,
        XorShiftRng
    };
    use cgmath::{
        Vector3,
        InnerSpace
    };

    pub fn lambert_unit_vector() -> Vector3<f32> {
        let mut p: Vector3<f32>;
        let mut rng = rand::thread_rng();
        loop {
            p = 2.0 * Vector3::new(
                rng.gen::<f32>(),
                rng.gen::<f32>(),
                rng.gen::<f32>()
            ) - Vector3::new(1.0, 1.0, 1.0);

            let sl = p.x*p.x + p.y*p.y + p.z*p.z;
            if sl < 1.0 {
                break;
            }
        }
        return p;
    }

    pub struct LambertTable {
        table: Vec<Vector3<f32>>,
        size: usize
    }

    impl LambertTable {
        pub fn new(size: usize) -> LambertTable {
            let mut table = vec![Vector3::new(0.0, 0.0, 0.0); size];
            for i in 0..size {
                table[i] = lambert_unit_vector();
            }

            LambertTable{
                table: table,
                size: size
            }
        }

        pub fn get(&self) -> Vector3<f32> {
            let mut rng = rand::thread_rng();
            let index = (rng.gen::<u32>() % self.size as u32) as usize;
            self.table[index]
        }
    }
}

pub trait Material: Send + Sync {
    fn reflect(&self, ray: Ray, hit: Hit) -> (Ray, bool);
}

pub struct Lambertian {}

impl Lambertian {
    pub fn new() -> Lambertian { Lambertian{} }
}

impl Material for Lambertian {
    fn reflect(&self, ray: Ray, hit: Hit) -> (Ray, bool) {
        let target = (hit.point + hit.norm) + material_utils::lambert_unit_vector();
        (
            Ray::new(hit.point, target - hit.point),
            true
        )
    }
}

pub struct Metal {}

impl Metal {
    pub fn new() -> Metal {
        Metal{}
    }
}

impl Material for Metal {
    fn reflect(&self, ray: Ray, hit: Hit) -> (Ray, bool) {
        let r = ray.direction() - 2.0 * (ray.direction().dot(hit.norm)) * hit.norm;
        (
            Ray::new(hit.point, r),
            true
        )
    }
}
