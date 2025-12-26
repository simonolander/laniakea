use crate::model::board::Board;
use crate::model::board_error::BoardError;
use crate::model::history::History;
use crate::model::objective::Objective;
use crate::model::universe::Universe;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

const GENERATE_SOLVED: bool = false;

#[wasm_bindgen]
pub struct State {
    /// The universe as it was generated, used for providing hints
    #[wasm_bindgen(skip)]
    pub universe: Universe,
    /// The current board state
    #[wasm_bindgen(skip)]
    pub board: Board,
    /// The set of centers and walls that the player needs to solve for
    #[wasm_bindgen(skip)]
    pub objective: Objective,
    /// The errors of the current board state, None means that the player isn't done solving
    #[wasm_bindgen(skip)]
    pub error: Option<BoardError>,
    /// History of board states
    #[wasm_bindgen(skip)]
    pub history: History,
}

#[wasm_bindgen]
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

    pub fn get_view(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&StateView::from(self)).unwrap_throw()
    }
}

/// ```rust
#[derive(Serialize)]
pub struct StateView {
    pub vertical_borders: Vec<Vec<bool>>,
    pub horizontal_borders: Vec<Vec<bool>>,
    pub objective: Objective,
    pub error: Option<BoardError>,
    pub has_future: bool,
    pub has_past: bool,
}

impl From<&State> for StateView {
    fn from(state: &State) -> Self {
        StateView {
            vertical_borders: state.board.get_vertical_borders(),
            horizontal_borders: state.board.get_horizontal_borders(),
            objective: state.objective.clone(),
            error: state.error.clone(),
            has_future: state.history.has_future(),
            has_past: state.history.has_past(),
        }
    }
}
