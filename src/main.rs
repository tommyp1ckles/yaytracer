extern crate image;
extern crate rand;

use rand::Rng;
use image::{Rgb};
use image::RGB;

use image::png::PNGEncoder;
use std::fs::File;

use std::io;

use cgmath::Vector3;

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
        let a = Vector3::new(1.0, 1.0, 1.0);
        let b = Vector3::new(5.0, 5.0, 5.0);
        let r = Ray::new(a, b);
        assert_eq!(1, 1);
    }
}

//fn color(r: &Ray) -> color::

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

    for y in (0..IMG_HEIGHT) {
        for x in (0..IMG_WIDTH) {
            let pixel_index = (y * IMG_WIDTH + x);
            let index = pixel_index * 3;
            let r: f32 = (x as f32) / (IMG_WIDTH as f32);
            let g: f32 = (IMG_HEIGHT as f32 - y as f32) / (IMG_HEIGHT as f32);
            let b: f32 = 0.2;
            // set red
            data[index] = (255.99 * r) as u8;
            data[index+1] = (255.99 * g) as u8;
            data[index+2] = (255.99 * b) as u8;
 
        }
    }

    let r = write_image(
        &String::from("out.png"),
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
