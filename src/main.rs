extern crate image;
extern crate rand;
extern crate threadpool;

mod geometry;
use geometry::Hit;
use geometry::{
    Ray,
    Sphere,
    Triangle,
    Visible
};

use threadpool::ThreadPool;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

mod materials;

use materials::{
    Material,
    Lambertian,
    Metal
};

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

use std::time::Instant;
//use material_utils::LambertTable;

const ANTI_ALIASING_SAMPLE: i32 = 128;

const IMG_WIDTH: usize = 1920;
const IMG_HEIGHT: usize = 1080;
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

fn gamma(color: Vector3<f32>, n: f32) -> Vector3<f32> {
    let i: f32 = 1.0 / n;
    Vector3::new(
        f32::powf(color.x, i),
        f32::powf(color.y, i),
        f32::powf(color.z, i)
    )
}

use std::sync::{Mutex, Arc};

struct World {
    objects: Arc<Vec<Box<Visible>>>,
    materials: Arc<Vec<Box<Material>>>
}

use std::sync::RwLock;

#[derive(Debug, Copy, Clone)]
struct PixelMessage {
    color: Vector3<f32>,
    index: usize
}

fn main() {
    let start = Instant::now();
    println!("Time for some raytracing!");

    // Initialize world.
    let mut materials: Vec<Box<Material>> = Vec::new();
    materials.push(Box::new(
        Lambertian::new(),
    ));
    materials.push(Box::new(
        Metal::new(),
    ));
    materials.push(Box::new(
        Lambertian::new(),
    ));

    let mut objects: Vec<Box<Visible>> = Vec::new();
    objects.push(Box::new(
        Sphere::new(
            Vector3::new(0.0, 0.0, -1.0),
            0.5,
            0
        )
    ));

    objects.push(Box::new(
        Sphere::new(
            Vector3::new(-1.0, 0.0, -1.0),
            0.5,
            1
        )
    ));

    objects.push(Box::new(
        Sphere::new(
            Vector3::new(1.0, 0.0, -1.0),
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

    let objects = Arc::new(objects);
    let materials = Arc::new(materials);
    /*objects.push(Box::new(
        Triangle::new(
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(-1.0, 0.0, -1.0),
            Vector3::new(0.0, -1.0, -1.0),
            2
        )
    ));*/

    /*let world: World = World{
        materials: materials,
        objects: objects
    };*/

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

    let n_workers = 8;
    let pool = ThreadPool::new(n_workers);
    let (tx, rx): (Sender<PixelMessage>, Receiver<PixelMessage>) = mpsc::channel();
    for y in (0..IMG_HEIGHT) {
        for x in (0..IMG_WIDTH) {
            //bar.inc(1);
            let index = (y * IMG_WIDTH + x) * 3;
            let u: f32 = x as f32 / IMG_WIDTH as f32;
            let v: f32 = ((IMG_HEIGHT - y) as f32) / IMG_HEIGHT as f32;
            //let world = World{
            //    materials: materials.clone(),
            //    objects: objects.clone()
            //};
            //let materials_clone = materials.clone();
            //let materials_clone = materials.clone();
            let world = World{
                materials: materials.clone(),
                objects: objects.clone()
            };
            let tx = tx.clone();
            pool.execute(move|| {
                //println!("executing ray thread #{}", x + y * IMG_WIDTH);
                //let materials_clone = Arc::try_unwrap(materials_clone);
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
                tx.send(PixelMessage{
                    color: color,
                    index: index
                }).expect("channel will be there waiting for the pool");

            })
        }
    }

    for _ in 0..(IMG_HEIGHT*IMG_WIDTH) {
        let pm = rx.recv().unwrap();
        data[pm.index] = (pm.color.x * 255.99) as u8;
        data[pm.index+1] = (pm.color.y * 255.99) as u8;
        data[pm.index+2] = (pm.color.z * 255.99) as u8;
        //println!("Just did -> {}", pm.index);
    }

    //bar.finish();
    //process::exit(0x0100);

    let r = write_image(
        &String::from("output.png"),
        &data,
        IMG_WIDTH as u32,
        IMG_HEIGHT as u32   
    );

    /*match r {
        Ok(v) => println!("Ok, file written",),
        Err(e) => println!("Error writing file: {}", e)
    }*/

    //let v = Vector3::new(1.0, 2.0, 3.0);
    println!("Raytracing took: {}", start.elapsed().as_secs());
}
