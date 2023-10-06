

#[test]
fn example_integration_test() {
    let factory = tasks_war_runtime::runtime::bots::RandomBotFactory::new();
    let runner = tasks_war_runtime::runtime::GameRunner::new(factory);

    let _result = runner.run_game();
}