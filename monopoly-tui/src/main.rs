use std::{
    error::Error,
    io::{self, stdout},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, poll},
    execute, queue,
    style::{self, Color, Colors},
    terminal::{self, ClearType},
    tty::IsTty,
};

use monopoly_lib::{Board, MoveReason, Space};
use num_traits::{FromPrimitive, Num, NumCast};
use numformat::NumFormat;

fn main() -> Result<(), Box<dyn Error>> {
    // Check we've got a terminal
    if !stdout().is_tty() {
        Err("stdout is not a tty")?;
    }

    // Get stdout
    let mut stdout = stdout();

    // Enter alternate screen and clear it
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
    )?;

    // Enable raw mode
    terminal::enable_raw_mode()?;

    // Draw instructions
    draw_instructions(&mut stdout)?;

    // Create the board
    let mut board = Board::default();

    // Play the game
    draw_board(&board, &mut stdout)?;

    let mut terminate = false;
    let mut paused = false;

    while !terminate {
        let c = if paused {
            Some(read_char()?)
        } else {
            for _ in 0..1000 {
                board.turn();
            }
            draw_board(&board, &mut stdout)?;

            if board.turns() % 100_000_000 == 0 {
                paused = true;
            }

            poll_char()?
        };

        match c {
            Some('q') => terminate = true,
            Some('p') => paused = !paused,
            _ => (),
        }
    }

    // Reset the terminal
    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    // Disable raw mode
    terminal::disable_raw_mode()?;

    Ok(())
}

fn poll_char() -> std::io::Result<Option<char>> {
    while poll(Duration::from_secs(0))? {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            modifiers: _,
            state: _,
        })) = event::read()
        {
            return Ok(Some(c));
        }
    }

    Ok(None)
}

fn read_char() -> std::io::Result<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            modifiers: _,
            state: _,
        })) = event::read()
        {
            return Ok(c);
        }
    }
}

fn draw_instructions<W>(w: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    let mut draw_instruction_line = |y, line| -> io::Result<()> {
        queue!(
            w,
            cursor::MoveTo(6, y as u16 + 13),
            style::Print(format!("{:^55}", line)),
        )
    };

    draw_instruction_line(0, "MONOPOLY")?;
    draw_instruction_line(2, "Calculates the probability of landing")?;
    draw_instruction_line(3, "on each square by simulating moves")?;
    draw_instruction_line(5, "Press 'q' to exit")?;
    draw_instruction_line(6, "Press 'p' to toggle pause")?;

    w.flush()?;

    Ok(())
}

fn draw_board<W>(board: &Board, w: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    let xpad = 1;
    let ypad = 1;
    let xspace = 6;
    let yspace = 3;

    // Reset, clear the screen, hide cursor
    queue!(w, style::ResetColor, cursor::Hide, cursor::MoveTo(1, 1))?;

    // Draw the board
    queue!(w, style::SetColors(Colors::new(Color::Black, Color::Green)))?;

    let mut draw_square = |x, y, elem| -> io::Result<()> {
        let desc = space_desc(board, elem);

        let pct = percent(board.arrivals_on(elem), board.moves());

        let pct_str = if pct < 10.0 {
            format!("{:.2}%", pct)
        } else if pct < 100.0 {
            format!("{:.1}%", pct)
        } else {
            format!("{:.0}% ", pct)
        };

        queue!(
            w,
            cursor::MoveTo(x, y),
            style::Print(format!("{:^5}", desc)),
            cursor::MoveTo(x, y + 1),
            style::Print(pct_str)
        )?;

        Ok(())
    };

    // Top row
    for i in 0..10 {
        let elem = i as usize;
        draw_square(xpad + (i * xspace), ypad, elem)?;
    }

    // Right column
    for i in 0..10 {
        let elem = (i + 10) as usize;
        draw_square(xpad + (10 * xspace), ypad + (i * yspace), elem)?;
    }

    // Bottom row
    for i in 0..10 {
        let elem = (29 - i) as usize;
        draw_square(xpad + ((i + 1) * xspace), ypad + (10 * yspace), elem)?;
    }

    // Left column
    for i in 0..10 {
        let elem = (39 - i) as usize;
        draw_square(xpad, ypad + ((i + 1) * yspace), elem)?;
    }

    queue!(w, style::ResetColor)?;

    // Draw stats
    let draw_stat = |w: &mut W, y, desc: &str, value: u64| -> io::Result<()> {
        queue!(
            w,
            cursor::MoveTo(6 + (11 * xspace), y + ypad),
            style::Print(format!("{desc:16} : {:>16}", value.num_format())),
        )?;

        Ok(())
    };

    let draw_stat_pct = |w: &mut W, y, desc: &str, value: u64, total, dp| -> io::Result<()> {
        let pct = percent(value, total);

        queue!(
            w,
            cursor::MoveTo(6 + (11 * xspace), y + ypad),
            style::Print(format!(
                "{desc:16} : {:>16}  ({:.dp$}%)  ",
                value.num_format(),
                pct,
            )),
        )?;

        Ok(())
    };

    let blank_line = |w: &mut W, y| -> io::Result<()> {
        queue!(
            w,
            cursor::MoveTo(6 + (11 * xspace), y + ypad),
            style::Print(format!("{:48}", "")),
        )?;

        Ok(())
    };

    draw_stat(w, 0, "Turns", board.turns())?;
    draw_stat(w, 1, "Moves", board.moves())?;
    draw_stat_pct(w, 2, "Doubles", board.doubles_elem(0), board.turns(), 1)?;
    draw_stat_pct(
        w,
        3,
        "Double doubles",
        board.doubles_elem(1),
        board.turns(),
        1,
    )?;
    draw_stat_pct(
        w,
        4,
        "Triple doubles",
        board.doubles_elem(2),
        board.turns(),
        1,
    )?;

    let mut sorted = board
        .arrivals()
        .iter()
        .enumerate()
        .map(|(i, a)| (a, i))
        .collect::<Vec<_>>();

    sorted.sort();

    let mut y = 6;
    for (a, elem) in sorted.into_iter().rev().take(10) {
        let desc = space_desc(board, elem);
        draw_stat_pct(w, y, desc.as_str(), *a, board.moves(), 2)?;
        y += 1;

        queue!(w, style::SetAttribute(style::Attribute::Dim),)?;

        if *board.space(elem) == Space::Jail {
            // Special for jail - show % for just visiting
            let visiting =
                board.arrivals_on(elem) - board.arrival_reasons(elem).iter().sum::<u64>();

            draw_stat_pct(w, y, &format!("  {}", "Visiting"), visiting, *a, 2)?;
            y += 1;
        }

        for (reason, count) in board.arrival_reasons(elem).iter().enumerate() {
            if *count != 0 {
                draw_stat_pct(
                    w,
                    y,
                    &format!("  {}", MoveReason::from_usize(reason).unwrap()),
                    *count,
                    *a,
                    2,
                )?;
                y += 1;
            }
        }

        queue!(w, style::SetAttribute(style::Attribute::Reset),)?;
    }

    blank_line(w, y)?;

    w.flush()?;

    Ok(())
}

fn percent<I: Num + NumCast>(value: I, total: I) -> f64 {
    let total = total.to_f64().unwrap();

    if total == 0.0 {
        0.0
    } else {
        (value.to_f64().unwrap() * 100.0) / total
    }
}

fn space_desc(board: &Board, elem: usize) -> String {
    match board.space(elem) {
        Space::Go => "GO".to_string(),
        Space::Jail => "JAIL".to_string(),
        Space::FreeParking => "FREE".to_string(),
        Space::GoToJail => "GO2J".to_string(),
        Space::Property(set, n) => {
            format!("{}{}", (set + b'A') as char, n + 1)
        }
        Space::Rail(n) => format!("RAIL{}", n + 1),
        Space::Utility(n) => format!("UTIL{}", n + 1),
        Space::CommunityChest(n) => format!("COMM{}", n + 1),
        Space::Chance(n) => format!("CHNC{}", n + 1),
        Space::Tax(n) => format!("TAX{}", n + 1),
    }
}
