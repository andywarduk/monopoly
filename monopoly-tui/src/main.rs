use std::{
    error::Error,
    io::{self, stdout},
    time::Duration,
};

use clap::Parser;
use cli::Cli;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, poll},
    execute, queue,
    style::{self, Color, Colors},
    terminal::{self, ClearType},
    tty::IsTty,
};

use monopoly_lib::{movereason::MoveReason, sim::Board, space::SPACES};
use monopoly_lib::{space::Space, strategy::Strategy};
use num_traits::{FromPrimitive, Num, NumCast};
use numformat::NumFormat;

mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    // Check we've got a terminal
    if !stdout().is_tty() {
        Err("stdout is not a tty")?;
    }

    // Parse command line arguments
    let cli = Cli::parse();

    // Get stdout
    let mut stdout = stdout();

    // Enter alternate screen and clear it
    execute!(stdout, terminal::EnterAlternateScreen, terminal::Clear(ClearType::All),)?;

    // Enable raw mode
    terminal::enable_raw_mode()?;

    // Play the game
    game_loop(
        &mut stdout,
        if cli.wait {
            Strategy::JailWait
        } else {
            Strategy::PayJail
        },
    )?;

    // Reset the terminal
    execute!(stdout, style::ResetColor, cursor::Show, terminal::LeaveAlternateScreen)?;

    // Disable raw mode
    terminal::disable_raw_mode()?;

    Ok(())
}

#[derive(Default)]
struct State {
    terminate: bool,
    paused: bool,
    dirty_draw: bool,
    split_jail: bool,
}

fn game_loop<W>(w: &mut W, strategy: Strategy) -> io::Result<()>
where
    W: io::Write,
{
    // Create state
    let mut state = State::default();

    // Create the board, cards pulled in order
    let mut board = Board::new(strategy, false);

    // Play the game
    while !state.terminate {
        if !state.dirty_draw {
            // Clear and redraw the screen
            initialise_screen(w, &board, &state)?;
            state.dirty_draw = true;
        }

        if state.paused {
            // Paused - block waiting for events
            let event = event::read()?;

            process_event(event, &mut state)?;
        } else {
            // Do 1000 moves
            for _ in 0..1000 {
                board.turn();
            }

            // Draw the board
            draw(w, &board, state.split_jail)?;

            // Flush output
            w.flush()?;

            // Auto pause?
            if board.turns() % 100_000_000 == 0 {
                state.paused = true;
            }

            // Poll event queue
            while let Some(event) = poll_event()? {
                process_event(event, &mut state)?;
            }
        };
    }

    Ok(())
}

fn poll_event() -> std::io::Result<Option<Event>> {
    if poll(Duration::from_secs(0))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

fn process_event(event: Event, state: &mut State) -> io::Result<()> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            modifiers: _,
            state: _,
        }) => {
            // Keypress
            match c {
                'q' => state.terminate = true,
                'p' => state.paused = !state.paused,
                'j' => {
                    state.split_jail = !state.split_jail;
                    state.dirty_draw = false;
                }
                _ => (),
            }
        }
        Event::Resize(_, _) => {
            // Terminal resized
            state.dirty_draw = false;
        }
        _ => (), // Ignored
    }

    Ok(())
}

fn initialise_screen<W>(w: &mut W, board: &Board, state: &State) -> io::Result<()>
where
    W: io::Write,
{
    // Clear the screen
    execute!(w, terminal::Clear(ClearType::All))?;

    // Draw instructions
    draw_instructions(w)?;

    // Draw the board
    draw(w, board, state.split_jail)?;

    // Flush output
    w.flush()?;

    Ok(())
}

fn draw_instructions<W>(w: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    let mut draw_instruction_line = |y: &mut u16, line| -> io::Result<()> {
        queue!(w, cursor::MoveTo(6, *y + 13), style::Print(format!("{:^55}", line)),)?;

        *y += 1;

        Ok(())
    };

    let mut y = 0;

    draw_instruction_line(&mut y, "MONOPOLY")?;
    y += 1;
    draw_instruction_line(&mut y, "Calculates the probability of landing")?;
    draw_instruction_line(&mut y, "on each space by simulating moves")?;
    y += 1;
    draw_instruction_line(&mut y, "Press 'q' to exit")?;
    draw_instruction_line(&mut y, "Press 'p' to toggle pause")?;
    draw_instruction_line(&mut y, "Press 'j' to toggle Just Visiting")?;

    Ok(())
}

fn draw<W>(w: &mut W, board: &Board, split_jail: bool) -> io::Result<()>
where
    W: io::Write,
{
    draw_board(w, board, split_jail)?;
    draw_stats(w, board, split_jail)?;

    Ok(())
}

const XPAD: u16 = 1;
const YPAD: u16 = 1;
const XSPACE: u16 = 6;
const YSPACE: u16 = 3;

fn draw_board<W>(w: &mut W, board: &Board, split_jail: bool) -> io::Result<()>
where
    W: io::Write,
{
    // Reset, clear the screen, hide cursor
    queue!(w, style::ResetColor, cursor::Hide, cursor::MoveTo(1, 1))?;

    // Draw the board
    let mut draw_space_int = |x, y, desc, arrivals, space: &Space| -> io::Result<()> {
        let pct = percent(arrivals, board.moves());

        let pct_str = if pct < 10.0 {
            format!("{:.2}%", pct)
        } else if pct < 100.0 {
            format!("{:.1}%", pct)
        } else {
            format!("{:.0}% ", pct)
        };

        let (fgcol, bgcol) = match space {
            Space::Property(set, _) => match *set {
                0 => (Color::White, Color::DarkMagenta),
                1 => (Color::Black, Color::Blue),
                2 => (Color::Black, Color::Magenta),
                3 => (Color::White, Color::DarkYellow),
                4 => (Color::Black, Color::Red),
                5 => (Color::Black, Color::Yellow),
                6 => (Color::White, Color::DarkGreen),
                7 => (Color::White, Color::DarkBlue),
                _ => panic!("invalid set"),
            },
            _ => (Color::Black, Color::Green),
        };

        queue!(
            w,
            style::SetColors(Colors::new(fgcol, bgcol)),
            cursor::MoveTo(x, y),
            style::Print(format!("{:^5}", desc)),
            cursor::MoveTo(x, y + 1),
            style::Print(pct_str),
            style::ResetColor
        )?;

        Ok(())
    };

    let mut draw_space = |x, y, elem| -> io::Result<()> {
        let desc = space_desc(elem);

        draw_space_int(x, y, desc, board.arrivals_on(elem), &SPACES[elem])
    };

    // Top row
    for i in 0..10 {
        let elem = i as usize;
        draw_space(XPAD + (i * XSPACE), YPAD, elem)?;
    }

    // Right column (excluding jail)
    for i in 1..10 {
        let elem = (i + 10) as usize;
        draw_space(XPAD + (10 * XSPACE), YPAD + (i * YSPACE), elem)?;
    }

    // Bottom row
    for i in 0..10 {
        let elem = (29 - i) as usize;
        draw_space(XPAD + ((i + 1) * XSPACE), YPAD + (10 * YSPACE), elem)?;
    }

    // Left column (excluding go to jail)
    for i in 0..9 {
        let elem = (39 - i) as usize;
        draw_space(XPAD, YPAD + ((i + 1) * YSPACE), elem)?;
    }

    let visit_elem = Space::find(Space::Visit);
    let g2j_elem = Space::find(Space::GoToJail);

    let jail = board.arrivals_on(g2j_elem);
    let visit = board.arrivals_on(visit_elem);

    if split_jail {
        // Split jail & Just visiting
        draw_space_int(
            XPAD + (10 * XSPACE),
            YPAD,
            "VISIT".to_string(),
            visit,
            &SPACES[visit_elem],
        )?;
        draw_space_int(
            XPAD + (9 * XSPACE),
            YPAD + YSPACE,
            "JAIL".to_string(),
            jail,
            &SPACES[visit_elem],
        )?;
    } else {
        // Combined jail
        draw_space_int(
            XPAD + (10 * XSPACE),
            YPAD,
            "JAIL".to_string(),
            visit + jail,
            &SPACES[visit_elem],
        )?;
    }

    // Draw go to jail
    draw_space_int(XPAD, YPAD + (10 * YSPACE), "G2J".to_string(), 0, &SPACES[30])?;

    Ok(())
}

fn draw_stats<W>(w: &mut W, board: &Board, split_jail: bool) -> io::Result<()>
where
    W: io::Write,
{
    // Draw stats
    let draw_stat = |w: &mut W, y: &mut u16, desc: &str, value: u64| -> io::Result<()> {
        queue!(
            w,
            cursor::MoveTo(4 + (11 * XSPACE), *y + YPAD),
            style::Print(format!("{desc:20.20} : {:>16}", value.num_format())),
        )?;

        *y += 1;

        Ok(())
    };

    let draw_stat_pct = |w: &mut W, y: &mut u16, desc: &str, value: u64, total, dp| -> io::Result<()> {
        let pct = percent(value, total);

        draw_stat(w, y, desc, value)?;
        queue!(w, style::Print(format!("  ({:.dp$}%)  ", pct,)),)?;

        Ok(())
    };

    let blank_line = |w: &mut W, y: &mut u16| -> io::Result<()> {
        queue!(
            w,
            cursor::MoveTo(4 + (11 * XSPACE), *y + YPAD),
            style::Print(format!("{:52}", "")),
        )?;

        *y += 1;

        Ok(())
    };

    let mut y = 0;

    let turns = board.turns();
    let moves = board.moves();
    let doubles = board.doubles();
    let doubles_tot = doubles[0] + (2 * doubles[1]) + (3 * doubles[2]);
    let double_turns = doubles[0];
    let triple_turns = doubles[1] + doubles[2];
    let single_turns = board.turns() - (double_turns + triple_turns);

    draw_stat(w, &mut y, "Turns taken", turns)?;
    draw_stat_pct(w, &mut y, " Single move turns", single_turns, turns, 2)?;
    draw_stat_pct(w, &mut y, " Double move turns", double_turns, turns, 2)?;
    draw_stat_pct(w, &mut y, " Triple move turns", triple_turns, turns, 2)?;
    draw_stat_pct(w, &mut y, " Double double turns", doubles[1], turns, 2)?;
    draw_stat_pct(w, &mut y, " Triple double turns", doubles[2], turns, 2)?;
    draw_stat(w, &mut y, "Moves", moves)?;
    draw_stat_pct(w, &mut y, " Moves from double", doubles_tot, moves, 2)?;

    blank_line(w, &mut y)?;

    let g2j = Space::find(Space::GoToJail);
    let visit = Space::find(Space::Visit);

    let mut sorted = if split_jail {
        board
            .arrivals()
            .iter()
            .enumerate()
            .map(|(i, a)| match SPACES[i] {
                Space::Visit => (*a, i, 2),                            // Just visiting
                Space::GoToJail => (*a, Space::find(Space::Visit), 1), // Jail
                _ => (*a, i, 0),
            })
            .collect::<Vec<_>>()
    } else {
        let mut vec = board
            .arrivals()
            .iter()
            .enumerate()
            .map(|(i, a)| (*a, i, 0))
            .collect::<Vec<_>>();
        vec[visit].0 += vec[g2j].0;
        vec.swap_remove(g2j);
        vec
    };

    sorted.sort();

    for (a, elem, sub) in sorted.into_iter().rev().take(10) {
        let desc = space_desc(elem);
        draw_stat_pct(w, &mut y, desc.as_str(), a, board.moves(), 2)?;

        queue!(w, style::SetAttribute(style::Attribute::Dim),)?;

        let arrivals_elem = if SPACES[elem] == Space::Visit {
            match sub {
                0 => {
                    // Combined jail
                    draw_stat_pct(w, &mut y, &format!("  {}", "Visiting"), board.arrivals_on(elem), a, 2)?;
                    g2j
                }
                1 => g2j,  // Jail (split)
                2 => elem, // Just visiting (split)
                _ => panic!("Invalid sub"),
            }
        } else {
            elem
        };

        for (reason, count) in board.arrival_reasons_on(arrivals_elem).iter().enumerate() {
            if *count != 0 {
                draw_stat_pct(
                    w,
                    &mut y,
                    &format!("  {}", MoveReason::from_usize(reason).unwrap()),
                    *count,
                    a,
                    2,
                )?;
            }
        }

        queue!(w, style::SetAttribute(style::Attribute::Reset),)?;
    }

    blank_line(w, &mut y)?;

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

fn space_desc(elem: usize) -> String {
    match SPACES[elem] {
        Space::Go => "GO".to_string(),
        Space::Visit => "JAIL".to_string(),
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
