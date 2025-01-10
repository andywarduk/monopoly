use std::cmp::max;

use monopoly_lib::calc::transmatrix::TransMatrix;
use nalgebra::DMatrix;

use crate::matrix::{RenderMatrixCb, render_matrix};

pub fn print_summary<T>(items: Vec<T>, mat: DMatrix<f64>, desc: &str)
where
    T: std::fmt::Display,
{
    println!("-------- {desc} --------");

    print_matrix(&mat, None::<Vec<bool>>, Some(items.iter()), "", true);
}

pub fn print_steady(mat: &TransMatrix, desc: &str) {
    println!("-------- {desc} --------");

    print_matrix(mat.steady(), None::<Vec<bool>>, Some(mat.states().keys()), "", true);
}

// Generic matrix print functions

pub fn print_matrix<T, RH, CH>(
    matrix: &DMatrix<T>,
    colheaders: Option<CH>,
    rowheaders: Option<RH>,
    rowcolhd: &str,
    transpose: bool,
) where
    T: nalgebra::Scalar + std::fmt::Display,
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: IntoIterator + Clone,
    CH::Item: std::fmt::Display,
{
    // Get column lengths
    let lencount = if transpose { matrix.nrows() } else { matrix.ncols() };
    let mut max_lens = vec![0; lencount + 1];

    render_matrix(
        matrix,
        colheaders.clone(),
        rowheaders.clone(),
        rowcolhd,
        transpose,
        |(i, _j), value| {
            match value {
                RenderMatrixCb::RowColHd(d) => {
                    max_lens[i] = max(max_lens[i], d.len());
                }
                RenderMatrixCb::ColHd(d) | RenderMatrixCb::RowHd(d) => {
                    max_lens[i] = max(max_lens[i], format!("{d}").len());
                }
                RenderMatrixCb::Cell(d) => {
                    max_lens[i] = max(max_lens[i], format!("{d}").len());
                }
                RenderMatrixCb::Eol => (),
            }
            Ok(())
        },
    )
    .unwrap();

    // Print
    render_matrix(matrix, colheaders, rowheaders, rowcolhd, transpose, |(i, _j), value| {
        match value {
            RenderMatrixCb::RowColHd(d) => print_matrix_write(i, max_lens[i], d),
            RenderMatrixCb::ColHd(d) | RenderMatrixCb::RowHd(d) => print_matrix_write(i, max_lens[i], d),
            RenderMatrixCb::Cell(d) => print_matrix_write(i, max_lens[i], d),
            RenderMatrixCb::Eol => println!(),
        }
        Ok(())
    })
    .unwrap();
}

fn print_matrix_write(i: usize, length: usize, display: impl std::fmt::Display) {
    if i > 0 {
        print!(" ");
    }

    print!("{display:<length$}");
}
