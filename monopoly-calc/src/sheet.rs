use std::error::Error;

use monopoly_lib::{calc::transmatrix::TransMatrix, space::SPACES};
use nalgebra::Matrix;
use num_traits::NumCast;
use rust_xlsxwriter::Workbook;

pub fn write_prob_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_sheet(book, name, mat.combinedmat(), Some(mat.states().keys()), Some(mat.states().keys()), |p| {
        if float { p.as_f64().to_string() } else { p.to_string() }
    })?;

    Ok(())
}

pub fn write_move_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_sheet(book, name, mat.movemat(), Some(mat.states().keys()), Some(mat.states().keys()), |p| {
        if float { p.as_f64().to_string() } else { p.to_string() }
    })?;

    Ok(())
}

pub fn write_jump_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    let iter = SPACES.iter().map(|s| s.shortdesc());

    write_matrix_sheet(book, name, mat.jumpmat(), Some(iter.clone()), Some(iter), |p| {
        if float { p.as_f64().to_string() } else { p.to_string() }
    })?;

    Ok(())
}

pub fn write_steady_sheet(book: &mut Workbook, name: &str, mat: &TransMatrix) -> Result<(), Box<dyn Error>> {
    write_matrix_sheet(book, name, mat.steady(), Some(mat.states().keys()), None::<Vec<bool>>, |p| p.to_string())?;

    Ok(())
}

pub fn write_matrix_sheet<T, R, C, S, RH, CH, F>(
    book: &mut Workbook,
    name: &str,
    matrix: &Matrix<T, R, C, S>,
    colheaders: Option<CH>,
    rowheaders: Option<RH>,
    format: F,
) -> Result<(), Box<dyn Error>>
where
    T: nalgebra::Scalar,
    R: nalgebra::Dim,
    C: nalgebra::Dim,
    S: nalgebra::storage::Storage<T, R, C>,
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: IntoIterator + Clone,
    CH::Item: std::fmt::Display,
    F: Fn(&T) -> String,
{
    let sheet = book.add_worksheet().set_name(name)?;

    // Work out x and y offsets for data
    let xoffset: usize = if rowheaders.is_some() {
        sheet.set_repeat_rows(0, 0)?;
        1
    } else {
        0
    };

    let yoffset: usize = if colheaders.is_some() {
        sheet.set_repeat_columns(0, 0)?;
        1
    } else {
        0
    };

    sheet.set_freeze_panes(yoffset as u32, xoffset as u16)?;

    // Write column headers
    if let Some(colheaders) = colheaders {
        for (x, header) in colheaders.into_iter().enumerate() {
            sheet.write_string(0, (x + xoffset) as u16, header.to_string())?;
        }
    }

    // Write row headers
    if let Some(rowheaders) = rowheaders {
        for (y, header) in rowheaders.into_iter().enumerate() {
            sheet.write_string((y + yoffset) as u32, 0, header.to_string())?;
        }
    }

    // Write data rows
    for (y, row) in matrix.row_iter().enumerate() {
        for (x, prob) in row.iter().enumerate() {
            sheet.write_string((y + yoffset) as u32, (x + xoffset) as u16, format(prob))?;
        }
    }

    Ok(())
}

pub fn write_summary_sheet<T, R, C, S, H, F>(
    book: &mut Workbook,
    rowheaders: H,
    matrix: &Matrix<T, R, C, S>,
    name: &str,
    format: F,
) -> Result<(), Box<dyn Error>>
where
    T: nalgebra::Scalar + NumCast,
    R: nalgebra::Dim,
    C: nalgebra::Dim,
    S: nalgebra::storage::Storage<T, R, C>,
    H: IntoIterator + Clone,
    H::Item: std::fmt::Display,
    F: Fn(&H::Item) -> String,
{
    let sheet = book.add_worksheet().set_name(name)?;

    // Write header row
    sheet.write_string(0, 1, "Probability".to_string())?;
    sheet.write_string(0, 2, "Percentage".to_string())?;

    // Write headers
    for (y, header) in rowheaders.into_iter().enumerate() {
        sheet.write_string((y + 1) as u32, 0, format(&header))?;
    }

    for (y, value) in matrix.iter().enumerate() {
        let value = value.to_f64().unwrap();
        sheet.write_number((y + 1) as u32, 1, value)?;
        sheet.write_number((y + 1) as u32, 2, value * 100.0)?;
    }

    sheet.set_column_width(1, 20)?;
    sheet.set_column_width(2, 20)?;

    Ok(())
}
