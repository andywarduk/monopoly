use monopoly_lib::calc::transmatrix::TransMatrix;
use nalgebra::{DMatrix, Dyn, Matrix};

pub fn print_summary<T, F, D>(items: Vec<T>, mat: DMatrix<f64>, desc: &str, trans: F)
where
    F: Fn(&T) -> D,
    D: std::fmt::Display,
{
    println!("-------- {desc} --------");

    for (item, prob) in items.iter().zip(mat.iter()) {
        println!("{:>15}: {:<10.8} ({:>7.4}%)", trans(item), prob, prob * 100.0);
    }
}

pub fn print_steady(mat: &TransMatrix, desc: &str) {
    println!("-------- {desc} --------");

    let steady = mat.steady();
    let conv = steady
        .clone()
        .reshape_generic(Dyn(steady.column_iter().count()), Dyn(steady.row_iter().count()));

    print_matrix(&conv, None::<Vec<bool>>, Some(mat.states().keys()), |p| p.to_string());
}

pub fn print_matrix<T, R, C, S, RH, CH, F>(matrix: &Matrix<T, R, C, S>, colheaders: Option<CH>, rowheaders: Option<RH>, format: F)
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
    // Write header row
    if let Some(colheaders) = colheaders {
        for (i, header) in colheaders.clone().into_iter().enumerate() {
            if i > 0 || rowheaders.is_some() {
                print!(",");
            }

            print!("{}", header);
        }

        println!();
    }

    // Write state transition probability rows
    let mut rowheaders = rowheaders.map(|rowheaders| rowheaders.into_iter());

    for row in matrix.row_iter() {
        if let Some(headers) = &mut rowheaders {
            print!("{},", headers.next().unwrap());
        }

        println!("{}", row.iter().map(&format).collect::<Vec<String>>().join(","));
    }
}
