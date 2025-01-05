use std::{error::Error, path::Path};

use clap::Parser;
use cli::Cli;
use console::print_summary;
use csv::{write_jump_csv, write_move_csv, write_prob_csv, write_steady_csv, write_summary_csv};
use monopoly_lib::calc::transmatrix::TransMatrix;
use monopoly_lib::space::SPACES;
use monopoly_lib::strategy::Strategy;
use rust_xlsxwriter::Workbook;
use sheet::{write_jump_sheet, write_move_sheet, write_prob_sheet, write_steady_sheet, write_summary_sheet};

mod cli;
mod console;
mod csv;
mod sheet;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Calc probabilities when paying to get out of jail
    let pay_map = TransMatrix::new(Strategy::PayJail, cli.dp, cli.debug);

    // Calc probabilities when rolling to get out of jail
    let wait_map = TransMatrix::new(Strategy::JailWait, cli.dp, cli.debug);

    // Summarise steady state for pay strategy by board position
    let (pay_space_headings, pay_space_mat) = pay_map.steady_summary(|state| Some(state.position));

    // Summarise steady state for pay strategy by board set
    let (pay_set_headings, pay_set_mat) = pay_map.steady_summary(|state| Some(SPACES[state.position].set()));

    // Summarise steady state for wait strategy by board position
    let (wait_space_headings, wait_space_mat) = wait_map.steady_summary(|state| Some(state.position));

    // Summarise steady state for wait strategy by board set
    let (wait_set_headings, wait_set_mat) = wait_map.steady_summary(|state| Some(SPACES[state.position].set()));

    // -- Spreadsheet Output --

    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    // Write summary by space for pay strategy
    write_summary_sheet(
        &mut workbook,
        &pay_space_headings,
        &pay_space_mat,
        "Probablility by position (pay)",
        |p| SPACES[**p].shortdesc(),
    )?;

    // Write summary by set for pay strategy
    write_summary_sheet(&mut workbook, &pay_set_headings, &pay_set_mat, "Probablility by set (pay)", |p| {
        p.to_string()
    })?;

    // Write summary by space for wait strategy
    write_summary_sheet(
        &mut workbook,
        &wait_space_headings,
        &wait_space_mat,
        "Probablility by position (wait)",
        |p| SPACES[**p].shortdesc(),
    )?;

    // Write summary by set for wait strategy
    write_summary_sheet(&mut workbook, &wait_set_headings, &wait_set_mat, "Probablility by set (wait)", |p| {
        p.to_string()
    })?;

    // Write worksheets for steady states for both strategies
    write_steady_sheet(&mut workbook, "Pay Steady", &pay_map)?;
    write_steady_sheet(&mut workbook, "Wait Steady", &wait_map)?;

    // Write worksheets for combined probabilities for both strategies
    write_prob_sheet(&mut workbook, "Pay Prob Frac", &pay_map, false)?;
    write_prob_sheet(&mut workbook, "Pay Prob Flt", &pay_map, true)?;
    write_prob_sheet(&mut workbook, "Wait Prob Frac", &wait_map, false)?;
    write_prob_sheet(&mut workbook, "Wait Prob Flt", &wait_map, true)?;

    // Write worksheets for smove probabilities for both strategies
    write_move_sheet(&mut workbook, "Pay Moves Frac", &pay_map, false)?;
    write_move_sheet(&mut workbook, "Pay Moves Flt", &pay_map, true)?;
    write_move_sheet(&mut workbook, "Wait Moves Frac", &wait_map, false)?;
    write_move_sheet(&mut workbook, "Wait Moves Flt", &wait_map, true)?;

    // Write worksheets for jump probabilities (same for both strategies)
    write_jump_sheet(&mut workbook, "Jumps Frac", &wait_map, false)?;
    write_jump_sheet(&mut workbook, "Jumps Flt", &wait_map, true)?;

    // Save workbook
    workbook.save("probabilities.xlsx")?;

    // -- CSV Output --

    // Write summary by space for pay strategy
    write_summary_csv(Path::new("csv/pay_space.csv"), "Space", &pay_space_headings, &pay_space_mat, |p| {
        SPACES[**p].shortdesc()
    })?;

    // Write summary by set for pay strategy
    write_summary_csv(Path::new("csv/pay_set.csv"), "Set", &pay_set_headings, &pay_set_mat, |p| p.to_string())?;

    // Write summary by space for wait strategy
    write_summary_csv(Path::new("csv/wait_space.csv"), "Space", &wait_space_headings, &wait_space_mat, |p| {
        SPACES[**p].shortdesc()
    })?;

    // Write summary by set for wait strategy
    write_summary_csv(Path::new("csv/wait_set.csv"), "Set", &wait_set_headings, &wait_set_mat, |p| p.to_string())?;

    // Write csv for pay strategy steady state
    write_steady_csv(Path::new("csv/pay_steady.csv"), &pay_map)?;

    // Write csv for move probabilities for pay strategy
    write_move_csv(Path::new("csv/pay_move_frac.csv"), &pay_map, false)?;
    write_move_csv(Path::new("csv/pay_move_flt.csv"), &pay_map, true)?;

    // Write csv for combined probabilities for pay strategy
    write_prob_csv(Path::new("csv/pay_frac.csv"), &pay_map, false)?;
    write_prob_csv(Path::new("csv/pay_flt.csv"), &pay_map, true)?;

    // Write csv for wait strategy steady state
    write_steady_csv(Path::new("csv/wait_steady.csv"), &wait_map)?;

    // Write csv for move probabilities for wait strategy
    write_move_csv(Path::new("csv/wait_move_frac.csv"), &wait_map, false)?;
    write_move_csv(Path::new("csv/wait_move_flt.csv"), &wait_map, true)?;

    // Write csv for combined probabilities for wait strategy
    write_prob_csv(Path::new("csv/wait_frac.csv"), &wait_map, false)?;
    write_prob_csv(Path::new("csv/wait_flt.csv"), &wait_map, true)?;

    // Write csv for jump probabilities (same for both strategies)
    write_jump_csv(Path::new("csv/jump_frac.csv"), &wait_map, false)?;
    write_jump_csv(Path::new("csv/jump_flt.csv"), &wait_map, true)?;

    // -- Console output --

    // Write summary by space for pay strategy
    print_summary(pay_space_headings, pay_space_mat, "Probablility by position (pay)", |p| {
        SPACES[*p].shortdesc()
    });

    // Write summary by set for pay strategy
    print_summary(pay_set_headings, pay_set_mat, "Probablility by set (pay)", |p| *p);

    // Write summary by space for wait strategy
    print_summary(wait_space_headings, wait_space_mat, "Probablility by position (wait)", |p| {
        SPACES[*p].shortdesc()
    });

    // Write summary by set for wait strategy
    print_summary(wait_set_headings, wait_set_mat, "Probablility by set (wait)", |p| *p);

    Ok(())
}
