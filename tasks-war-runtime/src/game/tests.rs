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
}

#[test]
fn move_right() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), Direction::Right);

    assert_eq!((10, 11), game.get_tasks(0)[0].pos);
}

#[test]
fn move_right_too_far() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), Direction::Right);

    assert_eq!((10, 11), game.get_tasks(0)[0].pos);
}

#[test]
fn move_left() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), Direction::Left);

    assert_eq!((10, 9), game.get_tasks(0)[0].pos);
}

#[test]
fn move_up() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), Direction::Up);

    assert_eq!((9, 10), game.get_tasks(0)[0].pos);
}

#[test]
fn move_down() {
    // let task = Task::new(0, (10,10));

    let mut game = Game::new();

    game.move_task(TaskId(0, 0), Direction::Down);

    assert_eq!((11, 10), game.get_tasks(0)[0].pos);
}

#[test]
fn roll_over_left() {
    let task0 = Task::new(0, (0, 0));
    let task1 = Task::new(1, (10, 10));

    let mut game = Game::with_tasks(vec![task0, task1]);

    game.move_task(TaskId(0, 0), Direction::Left);

    assert_eq!((0, game.board_size.1 - 1), game.get_tasks(0)[0].pos);
}

#[test]
fn look_too_far() {
    let task0 = Task::with_weight(0, (10, 10), 4);
    let task1 = Task::new(1, (10, 10));

    let game = Game::with_tasks(vec![task0, task1]);

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

    let s = format!("{}", game.board);

    assert_eq!(s2, s);
}

#[test]
fn display_board() {
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

    let s = format!("{}", game.board);

    assert_eq!(s2, s);
}

#[test]
fn task_player_move_two_times() {
    let task0 = Task::with_weight(0, (0, 0), 32);

    let mut game =
        Game::with_full_customization(Some(vec![task0]), Some(vec![]), Some(BoardSize(5, 5)));

    game.move_task(TaskId(0, 0), Direction::Right);
    game.move_task(TaskId(0, 0), Direction::Right);

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

    game.move_task(TaskId(0, 0), Direction::Right);
    game.move_task(TaskId(0, 0), Direction::Right);

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

    game.move_task(TaskId(0, 0), Direction::Right);

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

    game.move_task(TaskId(0, 0), Direction::Right);
    game.move_task(TaskId(0, 0), Direction::Right);

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

    game.move_task(TaskId(0, 0), Direction::Right);

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

    game.move_task(TaskId(0, 0), Direction::Right);

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

    game.move_task(TaskId(1, 0), Direction::Right);

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
fn many_to_many_collision_same_team() {
    let task00 = Task::with_weight(0, (0, 0), 16);
    let task01 = Task::with_weight(0, (0, 0), 16);
    let task02 = Task::with_weight(0, (0, 1), 16);
    let task03 = Task::with_weight(0, (0, 1), 16);

    let mut game = Game::with_full_customization(
        Some(vec![task00, task01, task02, task03]),
        Some(vec![]),
        Some(BoardSize(5, 5)),
    );

    game.move_task(TaskId(0, 1), Direction::Right);

    if let BoardContent::Tasks(tts) = &game.board[0][0] {
        assert_eq!(1, tts.len());
        assert!(tts.contains(&TaskId(0, 0)));
        assert!(!tts.contains(&TaskId(0, 1)));
    } else {
        panic!("Must be BoardContent::Tasks")
    }

    if let BoardContent::Tasks(tts) = &game.board[0][1] {
        assert_eq!(3, tts.len());
        assert!(tts.contains(&TaskId(0, 1)));
        assert!(tts.contains(&TaskId(0, 2)));
        assert!(tts.contains(&TaskId(0, 3)));
    } else {
        panic!("Must be BoardContent::Tasks")
    }

    assert!(!game.get_task(TaskId(0, 0)).is_dead);
    assert!(!game.get_task(TaskId(0, 1)).is_dead);
    assert!(!game.get_task(TaskId(0, 2)).is_dead);
    assert!(!game.get_task(TaskId(0, 3)).is_dead);
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

#[test]
fn all_tasks_ids() {
    let game = Game::new();

    let result: Vec<TaskId> = game.get_all_task_ids();

    let expected = vec![TaskId(0, 0), TaskId(1, 0)];

    assert_eq!(2, result.len());

    assert!(result.iter().all(|tid| { expected.contains(tid) }));
    assert!(expected.iter().all(|tid| { result.contains(tid) }));
}

#[test]
fn new_game_is_not_finished() {
    let game = Game::new();

    assert!(!game.is_finished())
}

#[test]
fn only_player_zero_is_finished() {
    let game = Game::with_tasks(vec![Task::new(0, (0, 0))]);

    assert!(game.is_finished())
}

#[test]
fn when_all_tasks_die_the_game_is_finished() {
    let mut game = Game::with_tasks(vec![
        Task::with_weight(0, (0, 0), 64),
        Task::with_weight(1, (0, 1), 32),
    ]);

    game.move_task(TaskId(0, 0), Direction::Right);

    assert!(game.is_finished())
}

#[test]
fn kill_task() {
    let mut game = Game::with_tasks(vec![Task::new(0, (0, 0))]);

    game.kill(TaskId(0, 0));

    let task = game.get_task(TaskId(0, 0));

    assert!(task.is_dead);

    assert_eq!(BoardContent::None, *game.board.get_content((0, 0)))
}

#[test]
fn kill_task_with_friends() {
    let mut game = Game::with_tasks(vec![Task::new(0, (0, 0)), Task::new(0, (0, 0))]);

    game.kill(TaskId(0, 0));

    let task = game.get_task(TaskId(0, 0));

    assert!(task.is_dead);

    if let BoardContent::Tasks(tts) = game.board.get_content(task.pos) {
        assert_eq!(1, tts.len());
        assert!(tts.contains(&TaskId(0, 1)));
    } else {
        panic!("expected tasks")
    }
}

#[test]
fn formatting() {
    let mut game = Game::from_config(GameConfig {
        board_size: BoardSize(10, 10),
        seed: 124,
    });

    game.kill(TaskId(0, 0));

    let string = game.to_string();

    let expected = r"Player 0 points: 0
Player 1 points: 0

Player 0 tasks: D
Player 1 tasks: A
_________G
___G___B__
_____G____
______G___
__________
1________B
______G_GB
__G__G__B_
________B_
_G________
";

    assert_eq!(expected, string);
}

#[test]
fn get_config() {
    let config = GameConfig {
        seed: 1024,
        board_size: BoardSize(10, 10),
    };

    let game = Game::from_config(config.clone());

    let returned_config = game.get_config();

    assert_eq!(*returned_config, config);
}

#[test]
fn if_there_is_no_food_left_game_is_over() {
    let tasks = vec![Task::new(0, (1, 0)), Task::new(1, (8, 8))];

    let fruits = vec![
        FruitPos {
            fruit: Fruit::Banana,
            pos: (2, 0),
        },
        FruitPos {
            fruit: Fruit::Banana,
            pos: (3, 0),
        },
    ];

    let board_size = BoardSize(10, 10);

    let mut game = Game::with_full_customization(Some(tasks), Some(fruits), Some(board_size));

    assert!(!game.is_finished());

    game.move_task(TaskId(0, 0), Direction::Down);

    assert!(game.points(0) > 0);
    assert!(!game.is_finished());

    game.move_task(TaskId(0, 0), Direction::Down);

    assert!(game.is_finished());
}

#[test]
fn remaining_fruits_counts_double_fruits() {
    let tasks = vec![Task::new(0, (1, 0)), Task::new(1, (8, 8))];

    let fruits = vec![
        FruitPos {
            fruit: Fruit::Banana,
            pos: (2, 0),
        },
        FruitPos {
            fruit: Fruit::Banana,
            pos: (3, 0),
        },
        FruitPos {
            fruit: Fruit::Strawberry,
            pos: (3, 0),
        },
    ];

    let board_size = BoardSize(10, 10);

    let mut game = Game::with_full_customization(Some(tasks), Some(fruits), Some(board_size));

    assert!(!game.is_finished());

    game.move_task(TaskId(0, 0), Direction::Down);

    assert!(game.points(0) > 0);
    assert!(!game.is_finished());

    assert_eq!(
        BoardContent::Food(Fruit::Banana),
        *game.board.get_content((3, 0))
    );

    game.move_task(TaskId(0, 0), Direction::Down);

    assert!(game.is_finished());
}
