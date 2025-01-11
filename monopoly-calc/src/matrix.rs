use std::error::Error;

use nalgebra::Matrix;

pub enum RenderMatrixCb<'a, T> {
    RowColHd(&'a str),
    ColHd(&'a dyn std::fmt::Display),
    RowHd(&'a dyn std::fmt::Display),
    Cell(&'a T),
    Eol,
}

pub fn render_matrix<T, R, C, S, RH, CH, F>(
    matrix: &Matrix<T, R, C, S>,
    colheaders: Option<CH>,
    rowheaders: Option<RH>,
    rowcolhd: &str,
    transpose: bool,
    cb: F,
) -> Result<(), Box<dyn Error>>
where
    R: nalgebra::Dim,
    C: nalgebra::Dim,
    S: nalgebra::RawStorage<T, R, C>,
    nalgebra::DefaultAllocator: nalgebra::allocator::Allocator<C, R>,
    T: Clone + PartialEq + nalgebra::Scalar,
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: IntoIterator + Clone,
    CH::Item: std::fmt::Display,
    F: FnMut((usize, usize), RenderMatrixCb<T>) -> Result<(), Box<dyn Error>>,
{
    // Transpose matrix if required
    if transpose {
        render_matrix_int(&matrix.transpose(), colheaders, rowheaders, rowcolhd, cb)
    } else {
        render_matrix_int(matrix, colheaders, rowheaders, rowcolhd, cb)
    }
}

pub fn render_matrix_int<T, R, C, S, RH, CH, F>(
    matrix: &Matrix<T, R, C, S>,
    colheaders: Option<CH>,
    rowheaders: Option<RH>,
    rowcolhd: &str,
    mut cb: F,
) -> Result<(), Box<dyn Error>>
where
    R: nalgebra::Dim,
    C: nalgebra::Dim,
    S: nalgebra::RawStorage<T, R, C>,
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: IntoIterator + Clone,
    CH::Item: std::fmt::Display,
    F: FnMut((usize, usize), RenderMatrixCb<T>) -> Result<(), Box<dyn Error>>,
{
    let yoffset = if colheaders.is_some() { 1 } else { 0 };
    let xoffset = if rowheaders.is_some() { 1 } else { 0 };

    // Write header row if specified
    if yoffset > 0 {
        for blank in 0..xoffset {
            cb((blank, 0), RenderMatrixCb::RowColHd(rowcolhd))?;
        }
    }

    if let Some(colheaders) = colheaders {
        for (j, header) in colheaders.clone().into_iter().enumerate() {
            cb((j + xoffset, 0), RenderMatrixCb::ColHd(&header))?;
        }

        cb((xoffset + matrix.ncols(), 0), RenderMatrixCb::Eol)?;
    }

    // Write rows
    let mut rowheaders = rowheaders.map(|rowheaders| rowheaders.into_iter());

    for (i, row) in matrix.row_iter().enumerate() {
        if let Some(headers) = &mut rowheaders {
            cb((0, i + yoffset), RenderMatrixCb::RowHd(&headers.next().unwrap()))?;
        }

        for (j, cell) in row.iter().enumerate() {
            cb((j + xoffset, i + yoffset), RenderMatrixCb::Cell(cell))?;
        }

        cb((xoffset + matrix.ncols(), i + yoffset), RenderMatrixCb::Eol)?;
    }

    Ok(())
}
