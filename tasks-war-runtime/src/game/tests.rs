use super::*;

#[test]
fn create_new_task() {
    let task = Task::new(0, (10, 15));

    assert_eq!(10, task.pos.0);
    assert_eq!(15, task.pos.1);
    assert_eq!(0, task.player);
    assert_eq!(64, task.weight);
}

#[test]
fn create_new_game() {
    let game = Game::new();

    assert_eq!(1, game.get_tasks(0).len());
    assert_eq!(1, game.get_tasks(1).len());
}

#[test]
fn look_distance() {
    let task = Task::new(0, (10, 10));

    assert_eq!(64, task.look_distance());
    assert_eq!(1, task.move_distance());
}

#[test]
fn move_right() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), 1, Direction::Right);

    assert_eq!((10, 11), game.get_tasks(0)[0].pos);
}

#[test]
fn move_right_too_far() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), 2, Direction::Right);

    assert_eq!((10, 11), game.get_tasks(0)[0].pos);
}

#[test]
fn move_left() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), 1, Direction::Left);

    assert_eq!((10, 9), game.get_tasks(0)[0].pos);
}

#[test]
fn move_up() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), 1, Direction::Up);

    assert_eq!((9, 10), game.get_tasks(0)[0].pos);
}

#[test]
fn move_down() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), 1, Direction::Down);

    assert_eq!((11, 10), game.get_tasks(0)[0].pos);
}

#[test]
fn roll_over_left() {
    let task0 = Task::new(0, (0, 0));
    let task1 = Task::new(1, (10, 10));

    let mut game = Game::with_tasks(vec![task0, task1]);

    game.move_task(TaskId(0, 0), 1, Direction::Left);

    assert_eq!((0, game.board_size.1 - 1), game.get_tasks(0)[0].pos);
}

#[test]
fn look_too_far() {
    let task0 = Task::with_weight(0, (10, 10), 4);
    let task1 = Task::new(1, (10, 10));

    let mut game = Game::with_tasks(vec![task0, task1]);

    let res = game.look(TaskId(0, 0), 5, 0);

    assert_eq!(LookResult::Null, res);
}

#[test]
fn look_none() {
    let task0 = Task::with_weight(0, (10, 10), 64);
    let task1 = Task::new(1, (10, 10));

    let game = Game::with_tasks(vec![task0, task1]);

    let res = game.look(TaskId(0, 0), 5, 0);

    assert_eq!(LookResult::None, res);
}

#[test]
fn look_opponent() {
    let task0 = Task::new(0, (10, 10));
    let task1 = Task::new(1, (15, 12));

    let game = Game::with_tasks(vec![task0, task1]);

    let res = game.look(TaskId(0, 0), 5, 2);

    assert_eq!(LookResult::Opponent, res);
}

#[test]
fn look_player() {
    let task0 = Task::new(0, (10, 10));
    let task1 = Task::new(0, (12, 12));
    let task2 = Task::new(1, (15, 12));

    let game = Game::with_tasks(vec![task0, task1, task2]);

    let res = game.look(TaskId(0, 0), 2, 2);

    assert_eq!(LookResult::Player, res);
}

#[test]
fn look_fruit() {
    let task0 = Task::new(0, (10, 10));
    let task1 = Task::new(1, (15, 12));

    let game = Game::with_tasks(vec![task0, task1]);

    let res = game.look(TaskId(0, 0), 5, 5);

    assert_eq!(LookResult::Food, res);
}

#[test]
fn display_empty_board() {
    let game = Game::with_full_customization(Some(vec![]), Some(vec![]), Some(BoardSize(4, 4)));

    let mut s2: String = String::new();

    s2.push_str("____\n");
    s2.push_str("____\n");
    s2.push_str("____\n");
    s2.push_str("____\n");

    let s = format!("{}", game);

    assert_eq!(s2, s);
}

#[test]
fn display_game() {
    let task0 = Task::new(0, (2, 1));
    let task1 = Task::new(1, (3, 3));

    let game = Game::with_full_customization(
        Some(vec![task0, task1]),
        Some(vec![FruitPos {
            fruit: Fruit::Banana,
            pos: (1, 2),
        }]),
        Some(BoardSize(4, 4)),
    );

    let mut s2: String = String::new();

    s2.push_str("____\n");
    s2.push_str("__B_\n");
    s2.push_str("_0__\n");
    s2.push_str("___1\n");

    let s = format!("{}", game);

    assert_eq!(s2, s);
}

#[test]
fn task_player_move_two_times() {
    let task0 = Task::with_weight(0, (0, 0), 32);

    let mut game =
        Game::with_full_customization(Some(vec![task0]), Some(vec![]), Some(BoardSize(5, 5)));

    game.move_task(TaskId(0, 0), 1, Direction::Right);
    game.move_task(TaskId(0, 0), 1, Direction::Right);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)));
    assert_eq!(BoardContent::None, game.board[0][1]);
    assert_eq!(BoardContent::Tasks(vec![TaskId(0, 0)]), game.board[0][2]);

    assert_eq!((0, 2), game.get_task(TaskId(0, 0)).pos);
}

#[test]
fn task_player_collision() {
    let task0 = Task::with_weight(0, (0, 0), 32);
    let task1 = Task::with_weight(1, (0, 1), 20);

    let mut game = Game::with_full_customization(
        Some(vec![task0, task1]),
        Some(vec![]),
        Some(BoardSize(5, 5)),
    );

    game.move_task(TaskId(0, 0), 1, Direction::Right);
    game.move_task(TaskId(0, 0), 1, Direction::Right);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)));
    assert_eq!(BoardContent::None, game.board[0][1]);
    assert_eq!(BoardContent::Tasks(vec![TaskId(0, 0)]), game.board[0][2]);

    assert!(!game.get_task(TaskId(0, 0)).is_dead);
    assert!(game.get_task(TaskId(1, 0)).is_dead);

    assert_eq!((0, 2), game.get_task(TaskId(0, 0)).pos);
}

#[test]
fn task_player_looses_collision() {
    let task0 = Task::with_weight(0, (0, 0), 32);
    let task1 = Task::with_weight(1, (0, 1), 40);

    let mut game = Game::with_full_customization(
        Some(vec![task0, task1]),
        Some(vec![]),
        Some(BoardSize(5, 5)),
    );

    game.move_task(TaskId(0, 0), 1, Direction::Right);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)));
    assert_eq!(BoardContent::Tasks(vec![TaskId(1, 0)]), game.board[0][1]);

    assert!(game.get_task(TaskId(0, 0)).is_dead);
    assert!(!game.get_task(TaskId(1, 0)).is_dead);

    assert_eq!((0, 1), game.get_task(TaskId(1, 0)).pos);
}

#[test]
fn eat_fruit() {
    let task0 = Task::with_weight(0, (0, 0), 32);

    let mut game = Game::with_full_customization(
        Some(vec![task0]),
        Some(vec![FruitPos {
            fruit: Fruit::Banana,
            pos: (0, 1),
        }]),
        Some(BoardSize(5, 5)),
    );

    game.move_task(TaskId(0, 0), 1, Direction::Right);
    game.move_task(TaskId(0, 0), 1, Direction::Right);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)));
    assert_eq!(BoardContent::None, game.board[0][1]);
    assert_eq!(BoardContent::Tasks(vec![TaskId(0, 0)]), game.board[0][2]);

    assert_eq!((0, 2), game.get_task(TaskId(0, 0)).pos);

    assert_eq!(Fruit::Banana.points(), game.points(0));
}

#[test]
fn tasks_collision_same_player() {
    let task0 = Task::with_weight(0, (0, 0), 32);
    let task1 = Task::with_weight(0, (0, 1), 40);

    let mut game = Game::with_full_customization(
        Some(vec![task0, task1]),
        Some(vec![]),
        Some(BoardSize(5, 5)),
    );

    game.move_task(TaskId(0, 0), 1, Direction::Right);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)));
    if let BoardContent::Tasks(tts) = &game.board[0][1] {
        assert!(tts.contains(&TaskId(0, 0)));
        assert!(tts.contains(&TaskId(0, 1)));
    } else {
        panic!("Must be BoardContent::Tasks")
    }

    assert!(!game.get_task(TaskId(0, 0)).is_dead);
    assert!(!game.get_task(TaskId(0, 1)).is_dead);

    assert_eq!((0, 1), game.get_task(TaskId(0, 0)).pos);
    assert_eq!((0, 1), game.get_task(TaskId(0, 1)).pos);

    game.move_task(TaskId(0, 0), 1, Direction::Right);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)));
    assert_eq!(BoardContent::Tasks(vec![TaskId(0, 1)]), game.board[0][1]);
    assert_eq!(BoardContent::Tasks(vec![TaskId(0, 0)]), game.board[0][2]);

    assert_eq!((0, 1), game.get_task(TaskId(0, 1)).pos);
    assert_eq!((0, 2), game.get_task(TaskId(0, 0)).pos);
}

#[test]
fn tasks_collision_multiple_tasks() {
    let task00 = Task::with_weight(0, (0, 1), 20);
    let task01 = Task::with_weight(0, (0, 1), 20);
    let task10 = Task::with_weight(1, (0, 0), 30);

    let mut game = Game::with_full_customization(
        Some(vec![task00, task01, task10]),
        Some(vec![]),
        Some(BoardSize(5, 5)),
    );

    game.move_task(TaskId(1, 0), 1, Direction::Right);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)));
    if let BoardContent::Tasks(tts) = &game.board[0][1] {
        assert!(!tts.contains(&TaskId(1, 0)));
        assert!(tts.contains(&TaskId(0, 0)));
        assert!(tts.contains(&TaskId(0, 1)));
    } else {
        panic!("Must be BoardContent::Tasks")
    }

    assert!(!game.get_task(TaskId(0, 0)).is_dead);
    assert!(!game.get_task(TaskId(0, 1)).is_dead);
    assert!(game.get_task(TaskId(1, 0)).is_dead);
}

#[test]
fn split() {
    let task0 = Task::with_weight(0, (0, 0), 32);
    let task1 = Task::with_weight(0, (1, 1), 32);

    let mut game = Game::with_full_customization(
        Some(vec![task0, task1]),
        Some(vec![]),
        Some(BoardSize(5, 5)),
    );

    let new_task_id = game.split(TaskId(0, 0)).unwrap();

    assert_eq!(TaskId(0, 2), new_task_id);

    if let BoardContent::Tasks(tts) = &game.board.get_content((0, 0)) {
        assert!(tts.contains(&TaskId(0, 0)));
        assert!(tts.contains(&new_task_id));
    } else {
        panic!("Must be BoardContent::Tasks")
    }

    let new_task = game.get_task(new_task_id);

    assert_eq!(16, game.get_task(TaskId(0, 0)).weight);
    assert_eq!(16, new_task.weight);
    assert_eq!((0, 0), new_task.pos);
}

#[test]
fn split_error() {
    let task0 = Task::with_weight(0, (0, 0), 1);

    let mut game =
        Game::with_full_customization(Some(vec![task0]), Some(vec![]), Some(BoardSize(5, 5)));

    assert_eq!(
        Some(super::error_messages::NOT_ENOUGH_WEIGHT),
        game.split(TaskId(0, 0)).err()
    );
}
