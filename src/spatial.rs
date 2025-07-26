//! Spatial data types and traits for multi-dimensional indexing.

/// Core trait for any type that can be used in a KD-tree.
/// Provides dimensional access for tree algorithms (splitting, traversal, etc.)
pub trait Point {
    /// Get the value for a specific dimension.
    fn get_dimension(&self, dim: usize) -> f64;

    /// Get the total number of dimensions.
    fn dimensions(&self) -> usize;
}

/// Extended trait for spatial range queries (bounding box searches, etc.)
/// Separates spatial operations from basic KD-tree operations.
pub trait SpatialPoint: Point {
    /// Check if this point/region is fully within the query region.
    fn is_within(&self, query: &Self) -> bool;

    /// Check if this point/region overlaps with the query region.
    fn overlaps(&self, query: &Self) -> bool;
}

/// 4-dimensional bounding box for spatial indexing.
/// Represents a rectangular region in 2D space with min/max coordinates.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "tantivy", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundingBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BoundingBox {
    /// Create a new bounding box from min/max coordinates.
    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        BoundingBox {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }
}

impl Point for BoundingBox {
    /// Get value for dimension (0=xmin, 1=ymin, 2=xmax, 3=ymax)
    fn get_dimension(&self, dim: usize) -> f64 {
        match dim {
            0 => self.xmin,
            1 => self.ymin,
            2 => self.xmax,
            3 => self.ymax,
            _ => panic!("Invalid dimension: {}", dim),
        }
    }

    fn dimensions(&self) -> usize {
        4
    }
}

impl SpatialPoint for BoundingBox {
    /// Check if this bounding box is fully within the query box
    fn is_within(&self, query: &Self) -> bool {
        self.xmin >= query.xmin
            && self.xmax <= query.xmax
            && self.ymin >= query.ymin
            && self.ymax <= query.ymax
    }

    /// Check if this bounding box overlaps with the query box
    fn overlaps(&self, query: &Self) -> bool {
        !(self.xmax < query.xmin
            || self.xmin > query.xmax
            || self.ymax < query.ymin
            || self.ymin > query.ymax)
    }
}
