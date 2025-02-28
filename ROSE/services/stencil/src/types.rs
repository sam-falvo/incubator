/// A single coordinate component.
pub type Unit = isize;

/// A coordinate.
pub type Point = (Unit, Unit);

/// A rectangle in ((left, top), (right, bottom)) format.
pub type Rect = (Point, Point);

/// A dimension component.
pub type Dimension = isize;

/// A pair of dimensions indicating a width and a height.
pub type Dimensions = (Dimension, Dimension);

