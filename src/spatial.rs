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

    /// Return a new bounding box with the specified dimension set to a new value.
    /// Used for bounds calculation in SVG rendering.
    pub fn with_dimension(&self, dim: usize, value: f64) -> Self {
        match dim {
            0 => BoundingBox::new(value, self.ymin, self.xmax, self.ymax),
            1 => BoundingBox::new(self.xmin, value, self.xmax, self.ymax),
            2 => BoundingBox::new(self.xmin, self.ymin, value, self.ymax),
            3 => BoundingBox::new(self.xmin, self.ymin, self.xmax, value),
            _ => panic!("Invalid dimension: {}", dim),
        }
    }

    /// Compute union of two bounding boxes (enclosing box).
    /// Used for calculating overall bounds that contain multiple boxes.
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            xmin: self.xmin.min(other.xmin),
            ymin: self.ymin.min(other.ymin),
            xmax: self.xmax.max(other.xmax),
            ymax: self.ymax.max(other.ymax),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_point_trait() {
        let bbox = BoundingBox::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(bbox.get_dimension(0), 1.0); // xmin
        assert_eq!(bbox.get_dimension(1), 2.0); // ymin
        assert_eq!(bbox.get_dimension(2), 3.0); // xmax
        assert_eq!(bbox.get_dimension(3), 4.0); // ymax
        assert_eq!(bbox.dimensions(), 4);
    }

    #[test]
    fn test_bounding_box_spatial_operations() {
        let bbox1 = BoundingBox::new(1.0, 1.0, 3.0, 3.0);
        let bbox2 = BoundingBox::new(2.0, 2.0, 4.0, 4.0); // Overlaps bbox1
        let bbox3 = BoundingBox::new(0.0, 0.0, 5.0, 5.0); // Contains bbox1
        let bbox4 = BoundingBox::new(10.0, 10.0, 12.0, 12.0); // No overlap

        // Test overlaps
        assert!(bbox1.overlaps(&bbox2));
        assert!(bbox1.overlaps(&bbox3));
        assert!(!bbox1.overlaps(&bbox4));

        // Test is_within
        assert!(bbox1.is_within(&bbox3));
        assert!(!bbox1.is_within(&bbox2));
        assert!(!bbox1.is_within(&bbox4));
    }
}
