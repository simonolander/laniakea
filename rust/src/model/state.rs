use crate::model::board::Board;
use crate::model::board_error::BoardError;
use crate::model::history::History;
use crate::model::objective::Objective;
use crate::model::universe::Universe;

const GENERATE_SOLVED: bool = false;

pub struct State {
    /// The universe as it was generated, used for providing hints
    pub universe: Universe,
    /// The current board state
    pub board: Board,
    /// The set of centers and walls that the player needs to solve for
    pub objective: Objective,
    /// The errors of the current board state, None means that the player isn't done solving
    pub error: Option<BoardError>,
    /// History of board states
    pub history: History,
}

impl State {
    pub fn generate(size: usize) -> State {
        let universe = Universe::generate(size, size);
        let objective = Objective::generate(&universe);
        let mut board = Board::new(size, size);
        let error = None;
        let history = History::new();

        if GENERATE_SOLVED {
            for border in universe.get_galaxies().iter().flat_map(|g| g.get_borders()) {
                board.add_wall(border.p1(), border.p2());
            }
        }

        State {
            universe,
            board,
            objective,
            error,
            history,
        }
    }
}
