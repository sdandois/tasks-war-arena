use super::commons::*;
use super::fruits::*;
use super::tasks::*;

#[derive(Clone, Copy)]
pub struct BoardSize(pub usize, pub usize);

#[derive(PartialEq, Eq, Debug, Default)]
pub enum BoardContent {
    #[default]
    None,
    Tasks(Vec<TaskId>),
    Food(Fruit),
}

#[derive(Debug)]
pub struct Board(Vec<Vec<BoardContent>>);

impl Board {
    pub fn new(board_size: BoardSize) -> Board {
        let mut board = Vec::with_capacity(board_size.0);

        for _i in 0..board_size.0 {
            let mut v = Vec::with_capacity(board_size.1);

            for _j in 0..board_size.1 {
                v.push(BoardContent::None)
            }
            board.push(v)
        }

        Board(board)
    }

    pub fn add_task(&mut self, pos: Position, task_id: TaskId) {
        self.0[pos.0][pos.1] = match std::mem::take(&mut self.0[pos.0][pos.1]) {
            BoardContent::None => BoardContent::Tasks(vec![task_id]),
            BoardContent::Tasks(mut tts) => BoardContent::Tasks({
                tts.push(task_id);

                tts
            }),
            BoardContent::Food(_) => todo!(),
        };
    }

    pub fn get_content_mut(&mut self, pos: Position) -> &mut BoardContent {
        &mut self.0[pos.0][pos.1]
    }

    pub fn get_content(&self, pos: Position) -> &BoardContent {
        &self.0[pos.0][pos.1]
    }
}

impl std::ops::Index<usize> for Board {
    type Output = Vec<BoardContent>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::Index<Position> for Board {
    type Output = BoardContent;

    fn index(&self, index: Position) -> &Self::Output {
        self.get_content(index)
    }
}

impl std::fmt::Display for BoardContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            BoardContent::None => write!(f, "_"),
            BoardContent::Tasks(t) => write!(f, "{}", t.first().unwrap().0),
            BoardContent::Food(fruit) => write!(f, "{}", fruit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_by_position() {
        let b = Board::new(BoardSize(20, 20));

        assert_eq!(BoardContent::None, b[(1, 1)]);
    }
}
