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

//
fn write_image(filename: &String, data: &[u8], width: u32, height: u32) -> io::Result<()> {
    let mut file = File::create(filename).unwrap();
    let enc = PNGEncoder::new(file);
    /*let mut data: [u8; 3 * IMG_SIZE * IMG_SIZE]  = [0; 3 * IMG_SIZE * IMG_SIZE];
    let mut rng = rand::thread_rng();
    for i in 0..(3 * IMG_SIZE * IMG_SIZE) {
        let n1: u8 = rng.gen();
        data[i] = n1;
    }*/
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
        Vector3::new(1.0, 2.0, 3.0).dot(Vector3::new(3.0, 4.0, 1.0))
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

fn test(v: Vector3<f32>) {
    v.dot(v);
}

fn trace(r: &Ray) -> Vector3<f32> {
    if hit_sphere(Vector3::new(0.0, 0.0, -1.0), 0.5, r) {
        return Vector3::new(1.0, 0.0, 0.0)
    }
    gradient_color(r)
}

fn hit_sphere(center: Vector3<f32>, radius: f32, ray: &Ray) -> bool {
    let oc = ray.origin() - center;
    let a = ray.direction().dot(*ray.direction());
    let b = 2.0 * oc.dot(*ray.direction());
    let c = oc.dot(oc) - (radius*radius);
    let d = b*b - 4.0*a*c;
    (d > 0.0)
}

fn main() {
    println!("Time for some raytracing!");
    //let color = Rgb{
    //    data: [0, 0, 0]
    //};
    //println!("Color: {:?}", color);

    //write_image(&String::from("some_file.png"));

    let mut data: [u8; 3 * IMG_HEIGHT * IMG_WIDTH] = [0; 3 * IMG_HEIGHT * IMG_WIDTH];
    /*let mut rng = rand::thread_rng();
    for i in 0..(3 * IMG_HEIGHT * IMG_WIDTH) {
        data[i] = rng.gen();
    }*/

    let lower_left = Vector3::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);
    let origin = Vector3::new(0.0, 0.0, 0.0);
    for y in (0..IMG_HEIGHT) {
        for x in (0..IMG_WIDTH) {
            let index = (y * IMG_WIDTH + x) * 3;
            let u: f32 = x as f32 / IMG_WIDTH as f32;
            let v: f32 = (IMG_HEIGHT as f32 - y as f32) / IMG_HEIGHT as f32;

            let r = Ray::new(
                origin,
                lower_left + u * horizontal + v * vertical
            );

            let color = trace(&r);
            data[index] = (color.x * 255.99) as u8;
            data[index+1] = (color.y * 255.99) as u8;
            data[index+2] = (color.z * 255.99) as u8;
        }
    }

    let r = write_image(
        &String::from("out3.png"),
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
