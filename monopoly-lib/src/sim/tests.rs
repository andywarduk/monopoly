use super::*;

#[test]
fn test_jail_rolls_fail() {
    let mut board = Board::new(Strategy::JailWait, false);

    // Position 2 spaces before go to jail
    board.position = 28;

    // Roll double 2 - go to jail
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[10], 1);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 2);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[10], 2);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 3);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[10], 3);

    // Roll to get out (fail) - should now be on just visiting
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[10], 4);

    // Roll to move - should now be on just visiting
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, 13);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[10], 4);
    assert_eq!(board.arrivals[13], 1);

    // Check counts
    assert_eq!(board.turns, 5);
    assert_eq!(board.moves, 5);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(board.arrival_reason.iter().map(|reasons| reasons.iter().sum::<u64>()).sum::<u64>(), 3);
}

#[test]
fn test_jail_rolls_succ1() {
    let mut board = Board::new(Strategy::JailWait, false);

    // Position 2 spaces before go to jail
    board.position = 28;

    // Roll double 2
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[10], 1);

    // Roll to get out (success)
    board.turn_with_dice(|_board, doubles| {
        // Should only get one move
        assert_eq!(doubles, 0);
        (2, 2)
    });

    assert_eq!(board.position, 14);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[10], 1);
    assert_eq!(board.arrivals[14], 1);

    // Check counts
    assert_eq!(board.turns, 2);
    assert_eq!(board.moves, 2);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(board.arrival_reason.iter().map(|reasons| reasons.iter().sum::<u64>()).sum::<u64>(), 1);
}

#[test]
fn test_jail_rolls_succ2() {
    let mut board = Board::new(Strategy::JailWait, false);

    // Position 2 spaces before go to jail
    board.position = 28;

    // Roll double 2
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[10], 1);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 2);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[10], 2);

    // Roll to get out (success)
    board.turn_with_dice(|_board, doubles| {
        // Should only get one move
        assert_eq!(doubles, 0);
        (2, 2)
    });

    assert_eq!(board.position, 14);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[10], 2);
    assert_eq!(board.arrivals[14], 1);

    // Check counts
    assert_eq!(board.turns, 3);
    assert_eq!(board.moves, 3);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(board.arrival_reason.iter().map(|reasons| reasons.iter().sum::<u64>()).sum::<u64>(), 2);
}

#[test]
fn test_jail_rolls_succ3() {
    let mut board = Board::new(Strategy::JailWait, false);

    // Position 2 spaces before go to jail
    board.position = 28;

    // Roll double 2
    board.turn_with_dice(|_board, _doubles| (1, 1));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 1);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrivals[10], 1);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 2);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 1);
    assert_eq!(board.arrivals[10], 2);

    // Roll to get out (fail) - still in jail
    board.turn_with_dice(|_board, _doubles| (1, 2));

    assert_eq!(board.position, 10);
    assert_eq!(board.jailroll, 3);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[10], 3);

    // Roll to get out (success)
    board.turn_with_dice(|_board, doubles| {
        // Should only get one move
        assert_eq!(doubles, 0);
        (2, 2)
    });

    assert_eq!(board.position, 14);
    assert_eq!(board.jailroll, 0);
    assert_eq!(board.arrival_reason[10][MoveReason::GoToJail as usize], 1);
    assert_eq!(board.arrival_reason[10][MoveReason::NoDouble as usize], 2);
    assert_eq!(board.arrivals[10], 3);
    assert_eq!(board.arrivals[14], 1);

    // Check counts
    assert_eq!(board.turns, 4);
    assert_eq!(board.moves, 4);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(board.arrival_reason.iter().map(|reasons| reasons.iter().sum::<u64>()).sum::<u64>(), 3);
}

#[test]
fn test_chance_to_cc() {
    let mut board = Board::new(Strategy::JailWait, false);

    // Position 5 spaces before chance 3
    board.position = 31;

    board.chcardchoose = |_board| CHCard::Back3;
    board.cccardchoose = |_board| CCCard::Inconsequential;

    // Roll 5 to land on chance which will send us back 3 to the community chest
    board.turn_with_dice(|_board, _doubles| (2, 3));

    assert_eq!(board.position, 33);
    assert_eq!(board.arrival_reason[33][MoveReason::CHCard as usize], 1);
    assert_eq!(board.arrivals[33], 1);

    // Check counts
    assert_eq!(board.turns, 1);
    assert_eq!(board.moves, 1);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(board.arrival_reason.iter().map(|reasons| reasons.iter().sum::<u64>()).sum::<u64>(), 1);
}

#[test]
fn test_chance_to_cc_to_go() {
    let mut board = Board::new(Strategy::JailWait, false);

    // Position 5 spaces before chance 3
    board.position = 31;

    board.chcardchoose = |_board| CHCard::Back3;
    board.cccardchoose = |_board| CCCard::GoGo;

    // Roll 5 to land on chance which will send us back 3 to the community chest which will then send us to Go
    board.turn_with_dice(|_board, _doubles| (2, 3));

    assert_eq!(board.position, 0);
    assert_eq!(board.arrival_reason[0][MoveReason::CCCard as usize], 1);
    assert_eq!(board.arrivals[0], 1);

    // Check counts
    assert_eq!(board.turns, 1);
    assert_eq!(board.moves, 1);
    assert_eq!(board.arrivals.iter().sum::<u64>(), board.moves);
    assert_eq!(board.arrival_reason.iter().map(|reasons| reasons.iter().sum::<u64>()).sum::<u64>(), 1);
}
