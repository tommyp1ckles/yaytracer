pub struct Hit {
    pub is_hit: bool,
    pub t: f32,
    pub point: Vector3<f32>,
    pub norm: Vector3<f32>,
    pub material: usize
}

//pub mod ray {
use cgmath::{
    Vector3,
    InnerSpace
};

#[derive(Copy, Clone)]
pub struct Ray {
    A: Vector3<f32>,
    B: Vector3<f32>,
}

impl Ray {
    pub fn new(A: Vector3<f32>, B: Vector3<f32>) -> Ray {
        Ray{
            A: A,
            B: B,
        }
    }

    pub fn origin(&self) -> &Vector3<f32> { &self.A }
    pub fn direction(&self) -> &Vector3<f32> { &self.B }
    pub fn point(&self, t: f32) -> Vector3<f32> {
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
}

pub trait Visible {
    fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> Hit;
}


pub struct Sphere {
    center: Vector3<f32>,
    radius: f32,
    material: usize
}

impl Sphere {
    pub fn new(center: Vector3<f32>, radius: f32, material: usize) -> Sphere {
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

pub struct Triangle {
    v0: Vector3<f32>,
    v1: Vector3<f32>,
    v2: Vector3<f32>,
    material: usize
}

const EPSILON: f32 = 0.000001;

impl Triangle {
    pub fn new(v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>, material: usize) -> Triangle {
        Triangle{
            v0: v0,
            v1: v1,
            v2: v2,
            material: material
        }
    }
}

impl Visible for Triangle {
    // Inputs:
    // O
    // D
    // V0
    // V1
    // V2
    // Outputs:
    // t, u, v
    fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> Hit {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let pvec = ray.direction().cross(edge2);
        let det = edge1.dot(pvec);
        if det < EPSILON {
            return Hit{
                is_hit: false,
                t: 0.0,
                point: Vector3::new(0.0, 0.0, 0.0),
                norm: Vector3::new(0.0, 0.0, 0.0),
                material: self.material              
            };
        }

        let tvec = ray.origin() - self.v0;
        let mut u = tvec.dot(pvec);

        if u < 0.0 || u > det {
            return Hit{
                is_hit: false,
                t: 0.0,
                point: Vector3::new(0.0, 0.0, 0.0),
                norm: Vector3::new(0.0, 0.0, 0.0),
                material: self.material              
            }
        }

        let qvec = tvec.cross(edge1);

        let mut v = ray.direction().dot(qvec);

        if v < 0.0 || (u+v) > det {
            return Hit{
                is_hit: false,
                t: 0.0,
                point: Vector3::new(0.0, 0.0, 0.0),
                norm: Vector3::new(0.0, 0.0, 0.0),
                material: self.material              
            }           
        }

        let mut t = edge2.dot(qvec);
        let inv_det = 1.0 / det;

        t *= inv_det;
        u *= inv_det;
        v *= inv_det;

        // T(u, v) = (1 - u - v)*V0 + u*V1 + uV2

        Hit{
            is_hit: false,
            t: t,
            // TODO: !!!!!!!!!!!!!!!
            // TODO: !!!!!!!!!!!!!!!
            point: ray.origin() + t*ray.direction(),
            norm: Vector3::new(0.0, 0.0, 0.0),
            material: self.material                  
        }
    }
}