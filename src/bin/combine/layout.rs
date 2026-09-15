//! Where each material's cell sits in a combined image, and the UV region that
//! addresses it.

use crate::CombineError;

/// A material's rectangle in UV space, `v = 0` at the top edge of the image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvRegion {
    pub u_min: f32,
    pub u_max: f32,
    pub v_min: f32,
    pub v_max: f32,
}

/// A grid of equal square cells, filled left to right and then top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub columns: u32,
    pub rows: u32,
    pub cell: u32,
}

impl Layout {
    /// Places `count` cells of `cell` pixels within `limit` pixels per side.
    ///
    /// A single row when `count × cell` fits in `limit`; otherwise as many
    /// columns as fit, and as many rows as the remainder needs. The result
    /// depends on nothing but the three arguments, so the same inputs can be
    /// regenerated without remapping the uv.
    ///
    /// Errors with `NoMaterials` when `count` is zero, and with `TooLarge`
    /// when a single cell or the resulting grid height exceeds `limit`.
    pub fn place(count: usize, cell: u32, limit: u32) -> Result<Layout, CombineError> {
        if count == 0 {
            return Err(CombineError::NoMaterials);
        }
        let too_large = CombineError::TooLarge { count, cell, limit };
        if cell == 0 || cell > limit {
            return Err(too_large);
        }
        let count = count as u64;
        let columns = if count * u64::from(cell) <= u64::from(limit) {
            count
        } else {
            u64::from(limit / cell)
        };
        let rows = count.div_ceil(columns);
        if rows * u64::from(cell) > u64::from(limit) {
            return Err(too_large);
        }
        Ok(Layout {
            columns: columns as u32,
            rows: rows as u32,
            cell,
        })
    }

    /// Width of the combined image, in pixels.
    pub fn width(&self) -> u32 {
        self.columns * self.cell
    }

    /// Height of the combined image, in pixels.
    pub fn height(&self) -> u32 {
        self.rows * self.cell
    }

    /// Top-left pixel of cell `index`. `index` must be below the cell count
    /// the layout was placed for.
    pub fn origin(&self, index: usize) -> (u32, u32) {
        let (column, row) = self.column_row(index);
        (column * self.cell, row * self.cell)
    }

    /// UV region of cell `index`. Neighbouring regions share their edge value
    /// exactly, bit for bit.
    pub fn region(&self, index: usize) -> UvRegion {
        let (column, row) = self.column_row(index);
        UvRegion {
            u_min: fraction(column, self.columns),
            u_max: fraction(column + 1, self.columns),
            v_min: fraction(row, self.rows),
            v_max: fraction(row + 1, self.rows),
        }
    }

    fn column_row(&self, index: usize) -> (u32, u32) {
        let index = index as u32;
        (index % self.columns, index / self.columns)
    }
}

fn fraction(step: u32, steps: u32) -> f32 {
    step as f32 / steps as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    // The common case from the brief: four materials side by side, each owning a quarter of u.
    #[test]
    fn four_materials_fit_in_one_row() {
        let layout = Layout::place(4, 2048, 8192).unwrap();
        assert_eq!((layout.columns, layout.rows), (4, 1));
        assert_eq!((layout.width(), layout.height()), (8192, 2048));
        assert_eq!(
            layout.region(0),
            UvRegion {
                u_min: 0.0,
                u_max: 0.25,
                v_min: 0.0,
                v_max: 1.0
            }
        );
        assert_eq!(layout.origin(3), (6144, 0));
    }

    // A row exactly at the limit is still a row; the grid only starts past it.
    #[test]
    fn one_material_past_the_limit_becomes_a_grid() {
        let layout = Layout::place(5, 2048, 8192).unwrap();
        assert_eq!((layout.columns, layout.rows), (4, 2));
        assert_eq!(layout.origin(4), (0, 2048));
        assert_eq!(
            layout.region(4),
            UvRegion {
                u_min: 0.0,
                u_max: 0.25,
                v_min: 0.5,
                v_max: 1.0
            }
        );
    }

    // A partly filled last row keeps the full grid height, so v stays proportional.
    #[test]
    fn partial_last_row_keeps_full_height() {
        let layout = Layout::place(3, 4096, 8192).unwrap();
        assert_eq!((layout.columns, layout.rows), (2, 2));
        assert_eq!(layout.height(), 8192);
    }

    // Pins the size limit: a cell wider than it, or more rows than fit, is refused.
    #[test]
    fn oversized_layouts_are_refused() {
        assert!(matches!(
            Layout::place(1, 16384, 8192),
            Err(CombineError::TooLarge { .. })
        ));
        assert!(matches!(
            Layout::place(17, 2048, 8192),
            Err(CombineError::TooLarge { .. })
        ));
        assert!(Layout::place(16, 2048, 8192).is_ok());
    }

    // An empty material list has no layout rather than a divide by zero.
    #[test]
    fn no_materials_is_an_error() {
        assert!(matches!(
            Layout::place(0, 2048, 8192),
            Err(CombineError::NoMaterials)
        ));
    }

    // Adjacent regions must meet without a gap or overlap, even for thirds that f32 cannot hold exactly.
    #[test]
    fn neighbouring_regions_share_their_edge_exactly() {
        let layout = Layout::place(3, 1024, 8192).unwrap();
        for index in 0..2 {
            assert_eq!(
                layout.region(index).u_max.to_bits(),
                layout.region(index + 1).u_min.to_bits()
            );
        }
        assert_eq!(layout.region(2).u_max, 1.0);
    }
}
