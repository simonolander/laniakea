import { useEffect, useReducer, useRef } from "react";
import styles from "./App.module.css";

import { GameState, generate_state } from "../rust/pkg";
import type { StateView } from "../rust/bindings/StateView.ts";
import type { Border } from "../rust/bindings/Border.ts";

type AppState = {
  gameState: GameState;
  view: StateView;
};

type ToggleAction = {
  type: "TOGGLE";
  border: Border;
};

type Action = ToggleAction | { type: "NEW_GAME" };

function makeInitialState(): AppState {
  const gameState = generate_state();
  const view = gameState.get_view() as StateView;
  return { gameState, view };
}

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case "TOGGLE": {
      const { border } = action;
      state.gameState.toggle_border(
        border.p1.row,
        border.p1.column,
        border.p2.row,
        border.p2.column,
      );
    }
  }

  return {
    ...state,
    view: state.gameState.get_view() as StateView,
  };
}

function App() {
  let [state, dispatch] = useReducer(reducer, null, makeInitialState);

  useEffect(() => {
    return () => state.gameState.free();
  }, []);

  return (
    <div className={styles.appContainer}>
      <div className={styles.gameLayout}>
        {/* Main Board Area */}
        <div className={styles.board}>
          <div className={styles.gridPlaceholder}>
            {/* Generating 100 mock cells */}
            {Array.from({ length: 100 }).map((_, i) => (
              <div key={i} className={styles.gridCell} />
            ))}
          </div>
        </div>

        {/* Controls Column/Row */}
        <div className={styles.controls}>
          <button className={styles.btn}>Check Solution</button>
          <button className={`${styles.btn} ${styles.btnSecondary}`}>
            Undo
          </button>
          <button className={`${styles.btn} ${styles.btnSecondary}`}>
            Redo
          </button>
          <button className={`${styles.btn} ${styles.btnSecondary}`}>
            Reset
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;
