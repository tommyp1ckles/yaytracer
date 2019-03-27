extern crate image;
extern crate rand;

mod geometry;
use geometry::hit::Hit;
use geometry::ray::Ray;

use indicatif::{ProgressBar, ProgressStyle};

use rand::{
    Rng,
    XorShiftRng
};
use std::cmp::min;
use std::thread;
use std::time::Duration;

use image::{Rgb};
use image::RGB;

use image::png::PNGEncoder;
use std::fs::File;

use std::io;
use std::process;

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

use material_utils::LambertTable;

const ANTI_ALIASING_SAMPLE: i32 = 64;

const IMG_WIDTH: usize = 400;
const IMG_HEIGHT: usize = 200;
const T_MAX: f32 = 10000.0;
const T_MIN: f32 = 0.001;
const MAX_RECURSION_SIZE: i32 = 100;

fn write_image(filename: &String, data: &[u8], width: u32, height: u32) -> io::Result<()> {
    let mut file = File::create(filename).unwrap();
    let enc = PNGEncoder::new(file);
    enc.encode(
        &data,
        width,
        height,
        RGB(8)
    )
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_unit_vector() {
        let v = unit_vector(&Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(v.x, 0.57735026);
    }
}

fn unit_vector(v: &Vector3<f32>) -> Vector3<f32> {
    let euclidian_length = (v.x*v.x + v.y*v.y + v.z*v.z).sqrt();
    Vector3::new(
        v.x / euclidian_length,
        v.y / euclidian_length,
        v.z / euclidian_length
    )
}

// this generates a gradient based on a traced ray.
fn gradient_color(r: Ray) -> Vector3<f32> {
    let unit = unit_vector(r.direction());
    let t = 0.5 * (unit.y + 1.0);
    return (1.0-t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);
}

struct World {
    objects: Vec<Box<Visible>>,
    materials: Vec<Box<Material>>
}

fn trace(r: Ray, world: &World, depth: i32) -> Vector3<f32> {
    if depth >= MAX_RECURSION_SIZE {
        return gradient_color(r);
    }

    let mut hit = Hit{
        is_hit: false,
        t: 0.0,
        point: Vector3::new(0.0, 0.0, 0.0),
        norm: Vector3::new(0.0, 0.0, 0.0),
        material: 0
    };
    let mut closest = T_MAX; 
    let mut hit_exists = false;
    for object in world.objects.iter() {
        let tmp_hit = object.hit(r, T_MIN, closest);
        if tmp_hit.is_hit {
            closest = tmp_hit.t;
            hit_exists = true;
            hit = tmp_hit;
        }
    }

    if hit_exists {
        let (new_ray, was_reflected) = world.materials[hit.material].reflect(r, hit);
        return 0.5 * trace(new_ray, world, depth+1);
    }
    gradient_color(r)
}

trait Material {
    fn reflect(&self, ray: Ray, hit: Hit) -> (Ray, bool);
}

struct Lambertian {}

impl Lambertian {
    fn new() -> Lambertian { Lambertian{} }
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

struct Metal {}

impl Metal {
    fn new() -> Metal {
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

trait Visible {
    fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> Hit;
}

struct Sphere {
    center: Vector3<f32>,
    radius: f32,
    material: usize
}

impl Sphere {
    fn new(center: Vector3<f32>, radius: f32, material: usize) -> Sphere {
        Sphere{
            center: center,
            radius: radius,
            material: material
        }
    }
}

impl Visible for Sphere {
    fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> Hit {
        let oc = ray.origin() - self.center;
        let a = ray.direction().dot(*ray.direction());
        let b = 2.0 * oc.dot(*ray.direction());
        let c = oc.dot(oc) - self.radius*self.radius;
        let d = b*b - 4.0 * a*c;
        if d > 0.0 {
            let root_a = (-1.0 * b - d.sqrt())/(2.0*a);
            let root_b = (-1.0 * b + d.sqrt())/(2.0*a);

            if root_a < t_max && root_a > t_min {
                let p = ray.point(root_a);
                return Hit{
                    is_hit: true,
                    t: root_a,
                    point: p,
                    norm: (p - self.center) / self.radius,
                    material: self.material
                };
            }

            if root_b < t_max && root_b > t_min {
                let p = ray.point(root_b);
                return Hit{
                    is_hit: true,
                    t: root_b,
                    point: p,
                    norm: (p - self.center) / self.radius,
                    material: self.material
                };
            }
        }

        Hit{
            is_hit: false,
            t: 0.0,
            point: Vector3::new(0.0, 0.0, 0.0),
            norm: Vector3::new(0.0, 0.0, 0.0),
            material: self.material
        }
    }
}

fn gamma(color: Vector3<f32>, n: f32) -> Vector3<f32> {
    let i: f32 = 1.0 / n;
    Vector3::new(
        f32::powf(color.x, i),
        f32::powf(color.y, i),
        f32::powf(color.z, i)
    )
}

fn main() {
    println!("Time for some raytracing!");

    // Initialize world.
    let mut materials: Vec<Box<Material>> = Vec::new();
    materials.push(Box::new(
        Lambertian::new(),
    ));
    materials.push(Box::new(
        Metal::new(),
    ));

    let mut objects: Vec<Box<Visible>> = Vec::new();
    objects.push(Box::new(
        Sphere::new(
            Vector3::new(0.0, 0.0, -1.0),
            0.5,
            1
        )
    ));
    
    objects.push(Box::new(
        Sphere::new(
            Vector3::new(0.0, -100.5, -1.0),
            100.0,
            0
        )
    ));

    let world = World{
        materials: materials,
        objects: objects
    };

    // Note: Have to allocate data to heap in order to not overflow the stack during runtime.
    let mut data = vec![0; 3 * IMG_HEIGHT * IMG_WIDTH];

    let lower_left = Vector3::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);
    let origin = Vector3::new(0.0, 0.0, 0.0);

    let bar = ProgressBar::new((IMG_HEIGHT * IMG_WIDTH) as u64);
    bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:100.cyan/blue}] {pos:>7}/{len:7} Rays ({eta})")
        .progress_chars("#-"));

    for y in (0..IMG_HEIGHT) {
        for x in (0..IMG_WIDTH) {
            bar.inc(1);

            let index = (y * IMG_WIDTH + x) * 3;
            let u: f32 = x as f32 / IMG_WIDTH as f32;
            let v: f32 = ((IMG_HEIGHT - y) as f32) / IMG_HEIGHT as f32;
            
            let mut rng = rand::thread_rng();
            let mut color = Vector3::new(0.0, 0.0, 0.0);
            // TODO: Better to just divide the pixel, random leads to strange re
            for sample in 0..ANTI_ALIASING_SAMPLE {
                let mut rng = rand::thread_rng();
                let u: f32 = (x as f32 + rng.gen::<f32>()) / IMG_WIDTH as f32;
                let v: f32 = ((IMG_HEIGHT - y) as f32 + rng.gen::<f32>()) / IMG_HEIGHT as f32;

                let mut r = Ray::new(
                    origin,
                    unit_vector(&(lower_left + ((u * horizontal) + (v * vertical))))
                );

                // should be fine for concurrency since we're not passing
                // mutable references.
                let sample = trace(r, &world, 0);

                color += sample;
            }

            color /= ANTI_ALIASING_SAMPLE as f32;
            color = gamma(color, 2.0);
            
            data[index] = (color.x * 255.99) as u8;
            data[index+1] = (color.y * 255.99) as u8;
            data[index+2] = (color.z * 255.99) as u8;
        }
    }
    bar.finish();
    //process::exit(0x0100);

    let r = write_image(
        &String::from("output.png"),
        &data,
        IMG_WIDTH as u32,
        IMG_HEIGHT as u32   
    );

    match r {
        Ok(v) => println!("Ok, file written",),
        Err(e) => println!("Error writing file: {}", e)
    }

    let v = Vector3::new(1.0, 2.0, 3.0);
}
