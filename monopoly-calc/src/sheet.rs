use std::error::Error;

use monopoly_lib::{
    calc::{probability::Probability, transmatrix::TransMatrix},
    space::SPACES,
};
use nalgebra::DMatrix;
use num_traits::NumCast;
use rust_xlsxwriter::{IntoExcelData, Workbook};

use crate::matrix::{RenderMatrixCb, render_matrix};

pub fn write_prob_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_prob_sheet(
        book,
        name,
        mat.combinedmat(),
        Some(mat.states().keys()),
        Some(mat.states().keys()),
        false,
        float,
    )
}

pub fn write_move_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_prob_sheet(
        book,
        name,
        mat.movemat(),
        Some(mat.states().keys()),
        Some(mat.states().keys()),
        false,
        float,
    )
}

pub fn write_jump_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    let iter = SPACES.iter().map(|s| s.shortdesc());

    write_matrix_prob_sheet(book, name, mat.jumpmat(), Some(iter.clone()), Some(iter), false, float)
}

pub fn write_steady_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix) -> Result<(), Box<dyn Error>> {
    write_matrix_sheet(
        book,
        name,
        mat.steady(),
        None::<Vec<bool>>,
        Some(mat.states().keys()),
        true,
        |p| *p,
    )
}

pub fn write_summary_sheet<T, H>(
    book: &mut Workbook,
    rowheaders: H,
    matrix: &DMatrix<T>,
    name: &str,
) -> Result<(), Box<dyn Error>>
where
    T: nalgebra::Scalar + NumCast,
    H: IntoIterator + Clone,
    H::Item: std::fmt::Display,
{
    write_matrix_sheet(book, name, matrix, Some(["Probability"]), Some(rowheaders), true, |p| {
        p.to_f64()
    })?;

    Ok(())
}

// Generic matrix to spreadsheet functions

pub fn write_matrix_prob_sheet<RH, CH>(
    book: &mut Workbook,
    name: &str,
    matrix: &DMatrix<Probability>,
    colheaders: Option<CH>,
    rowheaders: Option<RH>,
    transpose: bool,
    float: bool,
) -> Result<(), Box<dyn Error>>
where
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: IntoIterator + Clone,
    CH::Item: std::fmt::Display,
{
    if float {
        write_matrix_sheet(book, name, matrix, colheaders, rowheaders, transpose, |p| p.as_f64())?;
    } else {
        write_matrix_sheet(book, name, matrix, colheaders, rowheaders, transpose, |p| p.to_string())?;
    }

    Ok(())
}

pub fn write_matrix_sheet<T, RH, CH, F, FR>(
    book: &mut Workbook,
    name: &str,
    matrix: &DMatrix<T>,
    colheaders: Option<CH>,
    rowheaders: Option<RH>,
    transpose: bool,
    format: F,
) -> Result<(), Box<dyn Error>>
where
    T: nalgebra::Scalar,
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: IntoIterator + Clone,
    CH::Item: std::fmt::Display,
    F: Fn(&T) -> FR,
    FR: IntoExcelData,
{
    let sheet = book.add_worksheet().set_name(name)?;

    let rowheadercnt = if rowheaders.is_some() { 1 } else { 0 };
    let colheadercnt = if colheaders.is_some() { 1 } else { 0 };

    sheet.set_freeze_panes(colheadercnt as u32, rowheadercnt as u16)?;

    // Draw the matrix
    render_matrix(
        matrix,
        colheaders,
        rowheaders,
        "",
        transpose,
        |(i, j), value| match value {
            RenderMatrixCb::RowColHd(string) => {
                sheet.write(j as u32, i as u16, string)?;
                Ok(())
            }
            RenderMatrixCb::ColHd(display) | RenderMatrixCb::RowHd(display) => {
                sheet.write(j as u32, i as u16, format!("{}", display))?;
                Ok(())
            }
            RenderMatrixCb::Cell(prob) => {
                sheet.write(j as u32, i as u16, format(prob))?;
                Ok(())
            }
            RenderMatrixCb::Eol => Ok(()),
        },
    )?;

    Ok(())
}
