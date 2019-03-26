extern crate image;
extern crate rand;

use rand::{
    Rng,
    XorShiftRng
};

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
            //println!("p = {:?}", p);
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

const IMG_WIDTH: usize = 600;
const IMG_HEIGHT: usize = 300;

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

#[derive(Copy, Clone)]
struct Ray {
    A: Vector3<f32>,
    B: Vector3<f32>,
}

impl Ray {
    fn new(A: Vector3<f32>, B: Vector3<f32>) -> Ray {
        Ray{
            A: A,
            B: B,
        }
    }

    fn origin(&self) -> &Vector3<f32> { &self.A }
    fn direction(&self) -> &Vector3<f32> { &self.B }
    fn point(&self, t: f32) -> Vector3<f32> {
        self.A + (t * self.B)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_ray_point() {
        let a = Vector3::new(0.0, 0.0, 0.0);
        let b = Vector3::new(1.0, 1.0, 1.0);
        let r = Ray::new(a, b);
        let p = r.point(1.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 1.0);
        assert_eq!(p.z, 1.0);
        Vector3::new(1.0, 2.0, 3.0).dot(Vector3::new(3.0, 4.0, 1.0));
    }

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
    //let unit = r.direction().unit_x();
    let unit = unit_vector(r.direction());
    let t = 0.5 * (unit.y + 1.0);
    return (1.0-t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);
}

// allocates a new variable to the stack and prints its address.
macro_rules! print_next_stack_ptr {
    () => (
        let dummy: i32 = 0x0;
        println!("next_stack_ptr: {:p}", &dummy);
    )
}


const T_MAX: f32 = 10000.0;
const T_MIN: f32 = 0.001;

fn trace(r: Ray, depth: i32) -> Vector3<f32> {
    //print_next_stack_ptr!();

    if depth >= 100 {
        return gradient_color(r);
    }

    let mut objects: Vec<Box<Visible>> = Vec::new();
    objects.push(Box::new(
        Sphere::new(
            Vector3::new(0.0, 0.0, -1.0),
            0.5
        )
    ));
    
    objects.push(Box::new(
        Sphere::new(
            Vector3::new(0.0, -100.5, -1.0),
            100.0
        )
    ));

    let mut hit = Hit{
        is_hit: false,
        t: 0.0,
        point: Vector3::new(0.0, 0.0, 0.0),
        norm: Vector3::new(0.0, 0.0, 0.0)
    };
    let mut closest = T_MAX; 
    let mut hit_exists = false;
    for object in objects.iter() {
        let tmp_hit = object.hit(r, T_MIN, closest);
        if tmp_hit.is_hit {
            closest = tmp_hit.t;
            hit_exists = true;
            hit = tmp_hit;
        }
    }

    if hit_exists {
        let target = (hit.point + hit.norm) + material_utils::lambert_unit_vector();
        //let target = (hit.point + hit.norm) + lambert;
        // redirect the ray, we reuse for easy tracing.
        //r.A = hit.point;
        //r.B = target - hit.point;
        let new_ray = Ray::new(
            hit.point,
            target - hit.point
        );
        return 0.5 * trace(new_ray, depth+1);
    }
    gradient_color(r)
}

struct Hit {
    is_hit: bool,
    t: f32,
    point: Vector3<f32>,
    norm: Vector3<f32>,
    //material: Material
}

//trait Material {
//    fn reflect(ray: Ray, point: Vector3<f32>)
//}

trait Visible {
    fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> Hit;
}

struct Sphere {
    center: Vector3<f32>,
    radius: f32,
}

impl Sphere {
    fn new(center: Vector3<f32>, radius: f32) -> Sphere {
        Sphere{
            center: center,
            radius: radius
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

            // todo: write a macro for this.
            if root_a < t_max && root_a > t_min {
                let p = ray.point(root_a);
                return Hit{
                    is_hit: true,
                    t: root_a,
                    point: p,
                    norm: (p - self.center) / self.radius
                };
            }

            if root_b < t_max && root_b > t_min {
                let p = ray.point(root_b);
                return Hit{
                    is_hit: true,
                    t: root_b,
                    point: p,
                    norm: (p - self.center) / self.radius
                };
            }
        }

        Hit{
            is_hit: false,
            t: 0.0,
            point: Vector3::new(0.0, 0.0, 0.0),
            norm: Vector3::new(0.0, 0.0, 0.0)
        }
    }
}

const ANTI_ALIASING_SAMPLE: i32 = 256;

fn print_stack_size() {
    let dummy: i32 = 0x0;
    unsafe {
        println!("print_stack_size: {:p}", &dummy);
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

    println!("Precomputing lambert values");
    let lambert = LambertTable::new(1024);
    println!("Tracing rays!");

    // Note: Have to allocate data to heap in order to not overflow the stack during runtime.
    // TODO: Investigate if it it's faster to allocate data to heap but then iterate over a 
    // fixed size buffer which gets written to data.
    //let mut data: [u8; 3 * IMG_HEIGHT * IMG_WIDTH] = [0; 3 * IMG_HEIGHT * IMG_WIDTH];
    let mut data = vec![0; 3 * IMG_HEIGHT * IMG_WIDTH];

    let lower_left = Vector3::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);
    let origin = Vector3::new(0.0, 0.0, 0.0);
    //for y in (IMG_HEIGHT-1..IMG_HEIGHT) {
    //    for x in (IMG_WIDTH-1..IMG_WIDTH) {
    for y in (0..IMG_HEIGHT) {
        for x in (0..IMG_WIDTH) {
            let index = (y * IMG_WIDTH + x) * 3;
            let u: f32 = x as f32 / IMG_WIDTH as f32;
            let v: f32 = ((IMG_HEIGHT - y) as f32) / IMG_HEIGHT as f32;
            
            let mut rng = rand::thread_rng();
            //let mut red: f32 = 0.0;
            //let mut green: f32 = 0.0;
            //let mut blue: f32 = 0.0;
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

                // TODO: Having the ray be mutable and borrowed is a dumb idea.
                let sample = trace(r, 0);
                //red += sample.x;
                //green += sample.y;
                //blue += sample.z;

                color += sample;
            }

            //println!("XY =>")
            //println!("RGB => {}, {}, {}", red, green, blue);

            color /= ANTI_ALIASING_SAMPLE as f32;
            color = gamma(color, 2.0);
            //red /= ANTI_ALIASING_SAMPLE as f32;
            //blue /= ANTI_ALIASING_SAMPLE as f32;
            //green /= ANTI_ALIASING_SAMPLE as f32;
            
            data[index] = (color.x * 255.99) as u8;
            data[index+1] = (color.y * 255.99) as u8;
            data[index+2] = (color.z * 255.99) as u8;
        }
    }

    println!("This was just a debug run");
    //process::exit(0x0100);

    let r = write_image(
        &String::from("ch7.png"),
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
