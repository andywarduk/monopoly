use std::{error::Error, path::Path};

use monopoly_lib::calc::transmatrix::TransMatrix;
use monopoly_lib::space::SPACES;
use monopoly_lib::strategy::Strategy;
use nalgebra::{DMatrix, Matrix};
use rust_xlsxwriter::Workbook;
use std::fs::File;
use std::io::Write;

const DEBUG: bool = true;
const ACCURACY_DP: u8 = 8;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    let pay_map = TransMatrix::new(Strategy::PayJail, ACCURACY_DP, DEBUG);

    let wait_map = TransMatrix::new(Strategy::JailWait, ACCURACY_DP, DEBUG);

    write_steady_sheet(&mut workbook, "Pay Steady", &pay_map)?;
    write_steady_sheet(&mut workbook, "Wait Steady", &wait_map)?;

    write_prob_sheet(&mut workbook, "Pay Prob Frac", &pay_map, false)?;
    write_prob_sheet(&mut workbook, "Pay Prob Flt", &pay_map, true)?;
    write_prob_sheet(&mut workbook, "Wait Prob Frac", &wait_map, false)?;
    write_prob_sheet(&mut workbook, "Wait Prob Flt", &wait_map, true)?;
    write_move_sheet(&mut workbook, "Pay Moves Frac", &pay_map, false)?;
    write_move_sheet(&mut workbook, "Pay Moves Flt", &pay_map, true)?;
    write_move_sheet(&mut workbook, "Wait Moves Frac", &wait_map, false)?;
    write_move_sheet(&mut workbook, "Wait Moves Flt", &wait_map, true)?;

    write_jump_sheet(&mut workbook, "Jumps Frac", &wait_map, false)?;
    write_jump_sheet(&mut workbook, "Jumps Flt", &wait_map, true)?;

    write_steady_csv(Path::new("csv/pay_steady.csv"), &pay_map)?;
    write_move_csv(Path::new("csv/pay_move_frac.csv"), &pay_map, false)?;
    write_move_csv(Path::new("csv/pay_move_flt.csv"), &pay_map, true)?;
    write_prob_csv(Path::new("csv/pay_frac.csv"), &pay_map, false)?;
    write_prob_csv(Path::new("csv/pay_flt.csv"), &pay_map, true)?;

    write_steady_csv(Path::new("csv/wait_steady.csv"), &wait_map)?;
    write_move_csv(Path::new("csv/wait_move_frac.csv"), &wait_map, false)?;
    write_move_csv(Path::new("csv/wait_move_flt.csv"), &wait_map, true)?;
    write_prob_csv(Path::new("csv/wait_frac.csv"), &wait_map, false)?;
    write_prob_csv(Path::new("csv/wait_flt.csv"), &wait_map, true)?;

    write_jump_csv(Path::new("csv/jump_frac.csv"), &wait_map, false)?;
    write_jump_csv(Path::new("csv/jump_flt.csv"), &wait_map, true)?;

    workbook.save("probabilities.xlsx")?;

    print_summary(
        pay_map.steady_summary(|state| Some(state.position)),
        "Probablility by position (pay)",
        |p| SPACES[*p].shortdesc(),
    );

    print_summary(
        pay_map.steady_summary(|state| Some(SPACES[state.position].set())),
        "Probablility by set (pay)",
        |p| p.clone(),
    );

    print_summary(
        wait_map.steady_summary(|state| Some(state.position)),
        "Probablility by position (wait)",
        |p| SPACES[*p].shortdesc(),
    );

    print_summary(
        wait_map.steady_summary(|state| Some(SPACES[state.position].set())),
        "Probablility by set (wait)",
        |p| p.clone(),
    );

    Ok(())
}

pub fn write_prob_csv(file: &Path, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_csv(file, mat.combinedmat(), Some(mat.states().keys()), Some(mat.states().keys()), |p| {
        if float { p.as_f64().to_string() } else { p.to_string() }
    })?;

    Ok(())
}

pub fn write_move_csv(file: &Path, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_csv(file, mat.movemat(), Some(mat.states().keys()), Some(mat.states().keys()), |p| {
        if float { p.as_f64().to_string() } else { p.to_string() }
    })?;

    Ok(())
}

pub fn write_jump_csv(file: &Path, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    let iter = SPACES.iter().map(|s| s.shortdesc());

    write_matrix_csv(file, mat.jumpmat(), Some(iter.clone()), Some(iter), |p| {
        if float { p.as_f64().to_string() } else { p.to_string() }
    })?;

    Ok(())
}

pub fn write_steady_csv(file: &Path, mat: &TransMatrix) -> Result<(), Box<dyn Error>> {
    write_matrix_csv(file, mat.steady(), Some(mat.states().keys()), None::<Vec<bool>>, |p| p.to_string())?;

    Ok(())
}

pub fn write_matrix_csv<T, R, C, S, RH, CH, F>(
    file: &Path,
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
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    };

    let mut file = File::create(file)?;

    // Write header row
    if let Some(colheaders) = colheaders {
        for header in colheaders.clone().into_iter() {
            write!(file, ",{}", header)?;
        }

        writeln!(file)?;
    }

    // Write state transition probability rows
    let mut rowheaders = rowheaders.map(|rowheaders| rowheaders.into_iter());

    for row in matrix.row_iter() {
        if let Some(headers) = &mut rowheaders {
            write!(file, "{},", headers.next().unwrap())?;
        }

        writeln!(file, "{}", row.iter().map(&format).collect::<Vec<String>>().join(","))?;
    }

    Ok(())
}

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

fn print_summary<T, F, D>((items, mat): (Vec<T>, DMatrix<f64>), desc: &str, trans: F)
where
    F: Fn(&T) -> D,
    D: std::fmt::Display,
{
    println!("---- {desc} ----");

    for (item, prob) in items.iter().zip(mat.iter()) {
        println!("{:>15}: {:<10.8} ({:>7.4}%)", trans(item), prob, prob * 100.0);
    }
}
