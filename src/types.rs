use euclid;
use imgref::ImgVec;
use rgb::RGBA8;
use std::ops::Add;

pub const INFINITY_SIZE: Size = Size::new(f32::INFINITY, f32::INFINITY);
pub const ZERO_SIZE: Size = Size::new(0.0, 0.0);

pub type PhyRect<T = i32> = euclid::Rect<T, PhysicUnit>;
pub type PhyPoint<T = i32> = euclid::Point2D<T, PhysicUnit>;
pub type PhySize<T = i32> = euclid::Size2D<T, PhysicUnit>;
pub type PhyVector<T = i32> = euclid::Vector2D<T, PhysicUnit>;

pub type Rect<T = f32> = euclid::Rect<T, LogicUnit>;
pub type Point<T = f32> = euclid::Point2D<T, LogicUnit>;
pub type Size<T = f32> = euclid::Size2D<T, LogicUnit>;
pub type Transform<T = f32> = euclid::Transform2D<T, LogicUnit, LogicUnit>;
pub type Vector<T = f32> = euclid::Vector2D<T, LogicUnit>;
pub type Angle<T = f32> = euclid::Angle<T>;
pub type Box2D<T = f32> = euclid::Box2D<T, LogicUnit>;

pub type Canvas = ImgVec<RGBA8>;
pub type Img = ImgVec<RGBA8>;

/// The tag for device unit system to prevent mixing values from different
/// system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicUnit;

/// The tag for logic unit system to prevent mixing values from different
/// system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicUnit;

/// Return the four corners of a rectangle: [left-top, right-top,
/// right-bottom, left-bottom]
pub fn rect_corners<T, U>(rect: &euclid::Rect<T, U>) -> [euclid::Point2D<T, U>; 4]
where
    T: Copy + Add<Output = T>,
{
    use euclid::Point2D;

    [
        rect.min(),
        Point2D::new(rect.max_x(), rect.min_y()),
        rect.max(),
        Point2D::new(rect.min_x(), rect.max_y()),
    ]
}

trait Draw {
    // fill(self) on Canvas
    fn fill(&self, canvas: &mut Canvas, bg: RGBA8);
    fn padding_left(&self, canvas: &mut Canvas, stride: usize);
}

impl Draw for PhyRect {
    fn fill(&self, canvas: &mut Canvas, bg: RGBA8) {
        let offset_y = self.min_y() as usize;
        let offset_x = self.min_x() as usize;
        let height = self.height() as usize;
        let width = self.width() as usize;

        for y in 0..height {
            for x in 0..width {
                let y = y + offset_y;
                let x = x + offset_x;

                canvas.as_mut()[x][y] = bg;
            }
        }
    }

    fn padding_left(&self, canvas: &mut Canvas, stride: usize) {
        let offset_y = self.min_y() as usize;
        let offset_x = self.min_x() as usize;
        let height = self.height() as usize;
        let width = self.width() as usize;

        for y in 0..height {
            for x in 0..width {
                let y = y + offset_y;
                let x = x + offset_x;

                // canvas.as_mut()[x][y] = bg;
            }
        }
    }
}

//trait Draw {
//    fn fill(&self, phy_rect: &PhyRect, buf: &mut Img);
//}
//
//impl Draw for PhyRect {
//    fn fill(&self, phy_rect: &PhyRect, buf: &mut Img) {
//        let pw = phy_rect.width() as usize;
//
//        let offset_y = self.min_y() as usize;
//        let offset_x = self.min_x() as usize;
//        let height = self.height() as usize;
//        let width = self.width() as usize;
//
//        for y in (0..height).step_by(4) {
//            for x in (0..width).step_by(4) {
//                let y = y + offset_y;
//                let x = x + offset_x;
//
//                buf.as_mut()[x][y] = RGBA8::new(11, 22, 33, 255);
//            }
//        }
//    }
//}
