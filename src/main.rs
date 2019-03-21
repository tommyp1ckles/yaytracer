extern crate image;
extern crate rand;

use rand::Rng;
use image::{Rgb};
use image::RGB;

use image::png::PNGEncoder;
use std::fs::File;

use std::io;

use cgmath::{
    Vector3,
    InnerSpace
};

const IMG_WIDTH: usize = 200;
const IMG_HEIGHT: usize = 100;

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

struct Ray {
    A: Vector3<f32>,
    B: Vector3<f32>,
}

impl Ray {
    fn new(A: Vector3<f32>, B: Vector3<f32>) -> Ray {
        Ray{
            A: A,
            B: B
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
fn gradient_color(r: &Ray) -> Vector3<f32> {
    //let unit = r.direction().unit_x();
    let unit = unit_vector(r.direction());
    let t = 0.5 * (unit.y + 1.0);
    return (1.0-t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);
}

const T_MAX: f32 = 10000.0;
const T_MIN: f32 = 0.0;

fn trace(r: &Ray, x: usize, y: usize, u: f32, v: f32) -> Vector3<f32> {

    let mut objects: Vec<Box<Visible>> = Vec::new();
    objects.push(Box::new(
        Sphere::new(
            Vector3::new(0.0, 0.0, -1.0),
            0.5
        )
    ));

    /*objects.push(Box::new(
        Sphere::new(
            Vector3::new(0.0, -3.0, -1.0),
            3.0
        )
    ));*/
    
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
        if x == 0 && y == 90 {
            println!("?hit = {}", tmp_hit.is_hit);
        }

        if tmp_hit.is_hit {
            closest = tmp_hit.t;
            hit_exists = true;
            hit = tmp_hit;
        }
    }


    if hit_exists {
        if cfg!(debug = "1") {
            println!("\n\nFor Ray ======> {:?} {:?}", r.A, r.B);
            println!("uv = {} {}", u, v);
            println!("XY => {} {}", x, y);
            println!("xyz = ({}, {}, {})", hit.norm.x, hit.norm.y, hit.norm.z);
        }
        return 0.5 * Vector3::new(
            hit.norm.x+1.0,
            hit.norm.y+1.0,
            hit.norm.z+1.0
        );
    }
    gradient_color(r)
}

fn hit_sphere(center: Vector3<f32>, radius: f32, ray: &Ray) -> (bool, f32) {
    let oc = ray.origin() - center;
    let a = ray.direction().dot(*ray.direction());
    let b = 2.0 * oc.dot(*ray.direction());
    let c = oc.dot(oc) - radius*radius;
    let d = b*b - 4.0*a*c;

    /*if d > 0.0 {
        println!("--- Hit Dump ---");
        println!("ray = {:?} {:?}", ray.A, ray.B);
        println!("oc = {:?}", oc);
        println!("a = {}", a);
        println!("b = {}", b);
        println!("c = {}", c);
        println!("d = {}", d);
    }*/

    (
        d > 0.0,
        (-1.0 * b - d.sqrt()) / (2.0 * a) 
    )
}

struct Hit {
    is_hit: bool,
    t: f32,
    point: Vector3<f32>,
    norm: Vector3<f32>
}

trait Visible {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Hit;
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
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Hit {
        let oc = ray.origin() - self.center;
        let a = ray.direction().dot(*ray.direction());
        let b = 2.0 * oc.dot(*ray.direction());
        let c = oc.dot(oc) - self.radius*self.radius;
        let d = b*b - 4.0 * a*c;

        /*println!("--- Hit Dump ---");
        println!("ray = {:?} {:?}", ray.A, ray.B);
        println!("oc = {:?}", oc);
        println!("a = {}", a);
        println!("b = {}", b);
        println!("c = {}", c);
        println!("d = {}", oc.dot(*ray.direction())*oc.dot(*ray.direction()) - a*c);*/
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

const ANTI_ALIASING_SAMPLE: i32 = 100;

fn main() {
    println!("Time for some raytracing!");
    let mut data: [u8; 3 * IMG_HEIGHT * IMG_WIDTH] = [0; 3 * IMG_HEIGHT * IMG_WIDTH];

    let lower_left = Vector3::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);
    let origin = Vector3::new(0.0, 0.0, 0.0);
    for y in (0..IMG_HEIGHT) {
        for x in (0..IMG_WIDTH) {
            let index = (y * IMG_WIDTH + x) * 3;
            let u: f32 = x as f32 / IMG_WIDTH as f32;
            let v: f32 = ((IMG_HEIGHT - y) as f32) / IMG_HEIGHT as f32;
            
            let mut rng = rand::thread_rng();
            let mut red: f32 = 0.0;
            let mut green: f32 = 0.0;
            let mut blue: f32 = 0.0;
            //let samples: [Vector3<f32>; ANTI_ALIASING_SAMPLE as usize] = 
            //    [Vector3::new(0.0, 0.0, 0.0); ANTI_ALIASING_SAMPLE as usize];
            for sample in 0..ANTI_ALIASING_SAMPLE {
                let u: f32 = (x as f32 + rng.gen::<f32>()) / IMG_WIDTH as f32;
                let v: f32 = ((IMG_HEIGHT - y) as f32 + rng.gen::<f32>()) / IMG_HEIGHT as f32;
                let r = Ray::new(
                    origin,
                    unit_vector(&(lower_left + ((u * horizontal) + (v * vertical))))
                );
                let sample = trace(&r, x, IMG_HEIGHT - y, u, v); 
                red += sample.x;
                green += sample.y;
                blue += sample.z;
            }

            red /= ANTI_ALIASING_SAMPLE as f32;
            blue /= ANTI_ALIASING_SAMPLE as f32;
            green /= ANTI_ALIASING_SAMPLE as f32;
    
            //let color = trace(&r, x, IMG_HEIGHT - y, u, v);
            data[index] = (red * 255.99) as u8;
            data[index+1] = (green * 255.99) as u8;
            data[index+2] = (blue * 255.99) as u8;
        }
    }

    let r = write_image(
        &String::from("out6.png"),
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
