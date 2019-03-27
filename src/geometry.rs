pub mod hit {   
    use cgmath::Vector3;

    pub struct Hit {
        pub is_hit: bool,
        pub t: f32,
        pub point: Vector3<f32>,
        pub norm: Vector3<f32>,
        pub material: usize
    }
}

pub mod ray {
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
}
