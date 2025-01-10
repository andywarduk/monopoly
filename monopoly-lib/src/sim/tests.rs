use super::*;

#[test]
fn test_jail_rolls_fail() {
    let mut board = Board::new(Strategy::JailWait, false);

    let g2j = Space::find(Space::GoToJail);
    let visit = Space::find(Space::Visit);

    // Position 2 spaces before go to jail
    board.position = g2j - 2;

    // Roll double 2 - go to jail
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[g2j], 1);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 2);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[g2j], 2);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 3);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[g2j], 3);

    // Roll to get out (fail) - should now be on just visiting
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrival_reason[visit][MoveReason::ExitJail as usize], 1);
    assert_eq!(board.arrivals[g2j], 3);
    assert_eq!(board.arrivals[visit], 1);

    // Roll to move - should now be moved
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, visit + 3);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrival_reason[visit][MoveReason::ExitJail as usize], 1);
    assert_eq!(board.arrivals[g2j], 3);
    assert_eq!(board.arrivals[visit], 1);
    assert_eq!(board.arrivals[visit + 3], 1);

    // Check counts
    assert_eq!(board.turns, 5);
    assert_eq!(board.moves, 5);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(
        board
            .arrival_reason
            .iter()
            .map(|reasons| reasons.iter().sum::<u64>())
            .sum::<u64>(),
        4
    );
}

#[test]
fn test_jail_rolls_succ1() {
    let mut board = Board::new(Strategy::JailWait, false);

    let g2j = Space::find(Space::GoToJail);
    let visit = Space::find(Space::Visit);

    // Position 2 spaces before go to jail
    board.position = g2j - 2;

    // Roll double 2
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[g2j], 1);

    // Roll to get out (success)
    board.turn_with_dice(|_board, doubles| {
        // Should only get one move
        assert_eq!(doubles, 0);
        (2, 2)
    });

    assert_eq!(board.position, visit + 4);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[g2j], 1);
    assert_eq!(board.arrivals[visit + 4], 1);

    // Check counts
    assert_eq!(board.turns, 2);
    assert_eq!(board.moves, 2);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(
        board
            .arrival_reason
            .iter()
            .map(|reasons| reasons.iter().sum::<u64>())
            .sum::<u64>(),
        1
    );
}

#[test]
fn test_jail_rolls_succ2() {
    let mut board = Board::new(Strategy::JailWait, false);

    let g2j = Space::find(Space::GoToJail);
    let visit = Space::find(Space::Visit);

    // Position 2 spaces before go to jail
    board.position = g2j - 2;

    // Roll double 2
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[g2j], 1);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 2);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[g2j], 2);

    // Roll to get out (success)
    board.turn_with_dice(|_board, doubles| {
        // Should only get one move
        assert_eq!(doubles, 0);
        (2, 2)
    });

    assert_eq!(board.position, visit + 4);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[g2j], 2);
    assert_eq!(board.arrivals[visit + 4], 1);

    // Check counts
    assert_eq!(board.turns, 3);
    assert_eq!(board.moves, 3);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(
        board
            .arrival_reason
            .iter()
            .map(|reasons| reasons.iter().sum::<u64>())
            .sum::<u64>(),
        2
    );
}

#[test]
fn test_jail_rolls_succ3() {
    let mut board = Board::new(Strategy::JailWait, false);

    let g2j = Space::find(Space::GoToJail);
    let visit = Space::find(Space::Visit);

    // Position 2 spaces before go to jail
    board.position = g2j - 2;

    // Roll double 2
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[g2j], 1);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 2);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[g2j], 2);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, visit);
    assert_eq!(board.jailroll, 3);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[g2j], 3);

    // Roll to get out (success)
    board.turn_with_dice(|_board, doubles| {
        // Should only get one move
        assert_eq!(doubles, 0);
        (2, 2)
    });

    assert_eq!(board.position, visit + 4);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[g2j][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[g2j][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[g2j], 3);
    assert_eq!(board.arrivals[visit + 4], 1);

    // Check counts
    assert_eq!(board.turns, 4);
    assert_eq!(board.moves, 4);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(
        board
            .arrival_reason
            .iter()
            .map(|reasons| reasons.iter().sum::<u64>())
            .sum::<u64>(),
        3
    );
}

#[test]
fn test_chance_to_cc() {
    let mut board = Board::new(Strategy::JailWait, false);

    let ch3 = Space::find(Space::Chance(2));
    let cc3 = Space::find(Space::CommunityChest(2));

    // Position 5 spaces before chance 3
    board.position = ch3 - 5;

    board.chcardchoose = |_rng, _deck| CHCard::Back3;
    board.cccardchoose = |_rng, _deck| CCCard::Inconsequential;

    // Roll 5 to land on chance which will send us back 3 to the community chest
    board.turn_with_dice(|_board, _doubles| (2, 3));

    assert_eq!(board.position, cc3);
    assert_eq!(board.arrival_reason[cc3][MoveReason::CHCard as usize], 1);
    assert_eq!(board.arrivals[cc3], 1);

    // Check counts
    assert_eq!(board.turns, 1);
    assert_eq!(board.moves, 1);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(
        board
            .arrival_reason
            .iter()
            .map(|reasons| reasons.iter().sum::<u64>())
            .sum::<u64>(),
        1
    );
}

#[test]
fn test_chance_to_cc_to_go() {
    let mut board = Board::new(Strategy::JailWait, false);

    let go = Space::find(Space::Go);
    let ch3 = Space::find(Space::Chance(2));

    // Position 5 spaces before chance 3
    board.position = ch3 - 5;

    board.chcardchoose = |_rng, _deck| CHCard::Back3;
    board.cccardchoose = |_rng, _deck| CCCard::GoGo;

    // Roll 5 to land on chance which will send us back 3 to the community chest which will then send us to Go
    board.turn_with_dice(|_board, _doubles| (2, 3));

    assert_eq!(board.position, go);
    assert_eq!(board.arrival_reason[go][MoveReason::CCCard as usize], 1);
    assert_eq!(board.arrivals[go], 1);

    // Check counts
    assert_eq!(board.turns, 1);
    assert_eq!(board.moves, 1);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(
        board
            .arrival_reason
            .iter()
            .map(|reasons| reasons.iter().sum::<u64>())
            .sum::<u64>(),
        1
    );
}
