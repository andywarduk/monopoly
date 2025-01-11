use std::io::Write;
use std::{error::Error, fs::File, path::Path};

use monopoly_lib::{calc::transmatrix::TransMatrix, space::SPACES};
use nalgebra::DMatrix;

use crate::matrix::render_matrix;

pub fn write_prob_csv(file: &Path, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_csv(
        file,
        mat.combinedmat(),
        Some(mat.states().keys()),
        Some(mat.states().keys()),
        "From \\ To",
        false,
        |p| {
            if float { p.as_f64().to_string() } else { p.to_string() }
        },
    )?;

    Ok(())
}

pub fn write_move_csv(file: &Path, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_csv(
        file,
        mat.movemat(),
        Some(mat.states().keys()),
        Some(mat.states().keys()),
        "From \\ To",
        false,
        |p| {
            if float { p.as_f64().to_string() } else { p.to_string() }
        },
    )?;

    Ok(())
}

pub fn write_jump_csv(file: &Path, mat: &TransMatrix, float: bool) -> Result<(), Box<dyn Error>> {
    write_matrix_csv(
        file,
        mat.jumpmat(),
        Some(SPACES.iter()),
        Some(SPACES.iter()),
        "From \\ To",
        false,
        |p| {
            if float { p.as_f64().to_string() } else { p.to_string() }
        },
    )?;

    Ok(())
}

pub fn write_steady_csv(file: &Path, mat: &TransMatrix) -> Result<(), Box<dyn Error>> {
    write_matrix_csv(
        file,
        mat.steady(),
        None::<Vec<bool>>,
        Some(mat.states().keys()),
        "",
        true,
        |p| p.to_string(),
    )?;

    Ok(())
}

pub fn write_summary_csv<RH>(
    file: &Path,
    matrix: &DMatrix<f64>,
    colheader: &str,
    rowheaders: RH,
    rowcolhd: &str,
) -> Result<(), Box<dyn Error>>
where
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
{
    write_matrix_csv(file, matrix, Some([colheader]), Some(rowheaders), rowcolhd, true, |p| {
        p.to_string()
    })?;

    Ok(())
}

// Generic matrix to csv functions

pub fn write_matrix_csv<T, RH, CH, F>(
    file: &Path,
    matrix: &DMatrix<T>,
    colheaders: Option<CH>,
    rowheaders: Option<RH>,
    rowcolhd: &str,
    transpose: bool,
    format: F,
) -> Result<(), Box<dyn Error>>
where
    T: nalgebra::Scalar,
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: IntoIterator + Clone,
    CH::Item: std::fmt::Display,
    F: Fn(&T) -> String,
{
    // Create parent directories
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    };

    // Open the output file
    let mut file = File::create(file)?;

    // Write matrix to output file
    render_matrix(matrix, colheaders, rowheaders, rowcolhd, transpose, |(i, _j), cb| {
        match cb {
            crate::matrix::RenderMatrixCb::RowColHd(string) => write_matrix_csv_write(&mut file, i, string)?,
            crate::matrix::RenderMatrixCb::ColHd(display) => write_matrix_csv_write(&mut file, i, display)?,
            crate::matrix::RenderMatrixCb::RowHd(display) => write_matrix_csv_write(&mut file, i, display)?,
            crate::matrix::RenderMatrixCb::Cell(value) => write_matrix_csv_write(&mut file, i, format(value))?,
            crate::matrix::RenderMatrixCb::Eol => writeln!(file)?,
        }

        Ok(())
    })?;

    Ok(())
}

fn write_matrix_csv_write(file: &mut File, i: usize, display: impl std::fmt::Display) -> Result<(), Box<dyn Error>> {
    if i > 0 {
        write!(file, ",")?;
    }

    write!(file, "{}", display)?;

    Ok(())
}
