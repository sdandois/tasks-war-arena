use crate::game_with_history::Command;

use super::game::*;

use super::game_memento::*;

pub struct GameReplay {
    game: Game,
    history: Vec<HistoryEntry>,
    cursor: usize,
}

impl From<GameMemento> for GameReplay {
    fn from(value: GameMemento) -> Self {
        GameReplay {
            game: Game::from_config(value.config.clone()),
            history: value.history,
            cursor: 0,
        }
    }
}

impl GameReplay {
    pub fn advance(&mut self) -> Option<()> {
        if self.cursor < self.history.len() {
            let HistoryEntry { task_id, command } = self.history[self.cursor].clone();
            self.cursor += 1;

            self.accept(task_id, &command);

            Some(())
        } else {
            None
        }
    }

    pub fn advance_skipping_looks(&mut self) -> Option<()> {
        while self.cursor < self.history.len() && self.history[self.cursor].command.is_look() {
            self.cursor += 1;
        }

        self.advance()
    }

    fn accept(&mut self, task_id: TaskId, command: &Command) {
        match command {
            Command::Move(random_delta, random_dir) => {
                self.game.move_task(task_id, *random_delta, *random_dir);
            }
            Command::Split => {
                let _res = self.game.split(task_id);
            }
            Command::Look(delta_x, delta_y) => {
                let _res = self.game.look(task_id, *delta_x, *delta_y);
            }
            Command::Pass => (),
        }
    }

    pub fn current(&self) -> &Game {
        &self.game
    }
}

#[cfg(test)]
mod tests {
    use crate::game_with_history::GameWithHistory;

    use super::*;

    #[test]
    fn replay_history() {
        let mut game = GameWithHistory::from_config(GameConfig {
            seed: 1,
            board_size: BoardSize(5, 5),
        });

        game.accept(TaskId(0, 0), &Command::Move(1, Direction::Right));
        game.accept(TaskId(1, 0), &Command::Move(1, Direction::Left));

        let memento = game.dump();

        let mut game_replay = GameReplay::from(memento);

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 0

Player 0 tasks: A
Player 1 tasks: A
_____
0____
_G1__
_____
__G__
";

        assert_eq!(exp_str, game_replay.current().to_string());

        game_replay.advance();

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 0

Player 0 tasks: A
Player 1 tasks: A
_____
_0___
_G1__
_____
__G__
";

        assert_eq!(exp_str, game_replay.current().to_string());

        game_replay.advance();

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 16

Player 0 tasks: A
Player 1 tasks: A
_____
_0___
_1___
_____
__G__
";
        print!("{}", game_replay.current());
        assert_eq!(exp_str, game_replay.current().to_string());
    }

    #[test]
    fn replay_history_skipping_looks() {
        let mut game = GameWithHistory::from_config(GameConfig {
            seed: 1,
            board_size: BoardSize(5, 5),
        });

        game.accept(TaskId(0, 0), &Command::Move(1, Direction::Right));
        game.accept(TaskId(0, 0), &Command::Look(1, 1));
        game.accept(TaskId(0, 0), &Command::Look(1, 1));
        game.accept(TaskId(1, 0), &Command::Move(1, Direction::Left));

        let memento = game.dump();

        let mut game_replay = GameReplay::from(memento);

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 0

Player 0 tasks: A
Player 1 tasks: A
_____
0____
_G1__
_____
__G__
";

        assert_eq!(exp_str, game_replay.current().to_string());

        game_replay.advance_skipping_looks();

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 0

Player 0 tasks: A
Player 1 tasks: A
_____
_0___
_G1__
_____
__G__
";

        assert_eq!(exp_str, game_replay.current().to_string());

        game_replay.advance_skipping_looks();

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 16

Player 0 tasks: A
Player 1 tasks: A
_____
_0___
_1___
_____
__G__
";
        print!("{}", game_replay.current());
        assert_eq!(exp_str, game_replay.current().to_string());
    }
}
