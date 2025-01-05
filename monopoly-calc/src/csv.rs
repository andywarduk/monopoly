use std::io::Write;
use std::{error::Error, fs::File, path::Path};

use monopoly_lib::{calc::transmatrix::TransMatrix, space::SPACES};
use nalgebra::Matrix;
use num_traits::NumCast;

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
        for (i, header) in colheaders.clone().into_iter().enumerate() {
            if i > 0 || rowheaders.is_some() {
                write!(file, ",")?;
            }

            write!(file, "{}", header)?;
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

pub fn write_summary_csv<T, R, C, S, RH, CH, F>(
    file: &Path,
    colheader: CH,
    rowheaders: RH,
    matrix: &Matrix<T, R, C, S>,
    format: F,
) -> Result<(), Box<dyn Error>>
where
    T: nalgebra::Scalar + std::fmt::Display + NumCast,
    R: nalgebra::Dim,
    C: nalgebra::Dim,
    S: nalgebra::storage::Storage<T, R, C>,
    RH: IntoIterator + Clone,
    RH::Item: std::fmt::Display,
    CH: std::fmt::Display,
    F: Fn(&RH::Item) -> String,
{
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    };

    let mut file = File::create(file)?;

    // Write header row
    writeln!(file, "{colheader},Probability,Percentage")?;

    // Write values
    let mut headers = rowheaders.into_iter();

    for value in matrix {
        // Write item name
        writeln!(file, "{},{},{}", format(&headers.next().unwrap()), value, value.to_f64().unwrap() * 100.0)?;
    }

    Ok(())
}
