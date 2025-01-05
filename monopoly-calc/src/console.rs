use nalgebra::DMatrix;

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
