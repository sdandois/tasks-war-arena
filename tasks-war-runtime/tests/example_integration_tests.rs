use tasks_war_runtime::game::TaskId;

#[test]
fn example_integration_test() {
    let factory = tasks_war_runtime::runtime::bots::RandomBotFactory::new();
    let runner = tasks_war_runtime::runtime::GameRunner::new(factory);

    let _result = runner.run_game();
}

#[test]
fn game_test() {
    let mut g = tasks_war_runtime::game::Game::from_seed(543);

    g.look(tasks_war_runtime::game::TaskId(0, 0), 1, 1);
    g.move_task(TaskId(0, 0), 1, tasks_war_runtime::game::Direction::Down);
    g.split(TaskId(0, 0)).unwrap();
}
