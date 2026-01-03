import { useEffect, useReducer } from "react";
import styles from "./App.module.css";
import boardStyles from "./Board.module.css";

import { GameState, generate_state } from "../rust/pkg";
import type { StateView } from "../rust/bindings/StateView.ts";
import type { Border } from "../rust/bindings/Border.ts";
import type { Position } from "../rust/bindings/Position.ts";
import { clsx } from "clsx";

type AppState = {
  gameState: GameState;
  view: StateView;
};

type ToggleAction = {
  type: "TOGGLE";
  border: Border;
};

type Action =
  | ToggleAction
  | { type: "NEW_GAME" }
  | { type: "CHECK" }
  | { type: "UNDO" }
  | { type: "REDO" }
  | { type: "HINT" }
  | { type: "CLEAR" };

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
      break;
    }
    case "CHECK": {
      state.gameState.check_solution();
      break;
    }
    case "UNDO": {
      state.gameState.undo();
      break;
    }
    case "REDO": {
      state.gameState.redo();
      break;
    }
    case "HINT": {
      state.gameState.take_hint();
      break;
    }
    case "NEW_GAME": {
      state.gameState.free();
      return makeInitialState();
    }
  }

  return {
    ...state,
    view: state.gameState.get_view() as StateView,
  };
}

function App() {
  const [state, dispatch] = useReducer(reducer, null, makeInitialState);

  useEffect(() => {
    return () => state.gameState.free();
  }, []);

  useEffect(() => {
    const beforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
    };
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, []);

  useEffect(() => {
    const promise = navigator.wakeLock.request("screen").catch(console.warn);
    return () => {
      promise.then((sentinel) => sentinel?.release().catch(console.warn));
    };
  }, []);

  return (
    <div className={styles.appContainer}>
      <div className={styles.gameLayout}>
        <div className={styles.board}>
          <Board
            view={state.view}
            onToggle={(border) => dispatch({ type: "TOGGLE", border })}
          />
        </div>

        <div className={styles.controls}>
          <button
            className={styles.btn}
            onClick={() => dispatch({ type: "UNDO" })}
            disabled={!state.view.has_past}
          >
            Undo
          </button>
          <button
            className={styles.btn}
            onClick={() => dispatch({ type: "REDO" })}
            disabled={!state.view.has_future}
          >
            Redo
          </button>
          {!state.view.is_solved && (
            <button
              className={styles.btn}
              onClick={() => dispatch({ type: "HINT" })}
            >
              Hint
            </button>
          )}
          {state.view.is_solved && (
            <button
              className={styles.btn}
              onClick={() => dispatch({ type: "NEW_GAME" })}
            >
              New game
            </button>
          )}
          {!state.view.is_solved && (
            <button
              className={styles.btn}
              onClick={() => dispatch({ type: "CHECK" })}
            >
              Check Solution
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

type BoardProps = {
  view: StateView;
  onToggle: (border: Border) => void;
};

function Board({ view, onToggle }: BoardProps) {
  const VIEW_BOX_SIZE = 100.0;
  const WALL_CELL_RATIO = 0.1;
  const SIZE = view.horizontal_borders[0].length;
  const CELL_SIZE = VIEW_BOX_SIZE / (SIZE + (SIZE + 1.0) * WALL_CELL_RATIO);
  const WALL_SIZE = CELL_SIZE * WALL_CELL_RATIO;

  // Helper to generate the diamond-shaped hit area for walls
  const getWallPoints = (p1: Position, p2: Position) => {
    const x_min =
      WALL_SIZE / 2.0 +
      ((WALL_SIZE + CELL_SIZE) * (p1.column + p2.column)) / 2.0;
    const x_max = x_min + CELL_SIZE + WALL_SIZE;
    const x_mid = (x_min + x_max) / 2.0;
    const y_min =
      WALL_SIZE / 2.0 + ((WALL_SIZE + CELL_SIZE) * (p1.row + p2.row)) / 2.0;
    const y_max = y_min + CELL_SIZE + WALL_SIZE;
    const y_mid = (y_min + y_max) / 2.0;
    return { x_min, x_max, x_mid, y_min, y_max, y_mid };
  };

  return (
    <svg
      viewBox={`0 0 ${VIEW_BOX_SIZE} ${VIEW_BOX_SIZE}`}
      className={boardStyles.board}
    >
      {
        // Render all cells
        Array.from({ length: SIZE }).map((_, row) =>
          Array.from({ length: SIZE }).map((_, col) => {
            const x = (WALL_SIZE + CELL_SIZE) * col;
            const y = (WALL_SIZE + CELL_SIZE) * row;
            const centerless = view.error?.centerless_cells.some(
              (p) => p.row === row && p.column === col,
            );

            return (
              <rect
                key={`cell-${row}-${col}`}
                x={x}
                y={y}
                width={CELL_SIZE + 2.0 * WALL_SIZE}
                height={CELL_SIZE + 2.0 * WALL_SIZE}
                className={clsx(
                  boardStyles.cell,
                  centerless && boardStyles.centerless,
                )}
              />
            );
          }),
        )
      }

      {
        // Render outer frame, the border rectangle
        <rect
          x={WALL_SIZE / 2.0}
          y={WALL_SIZE / 2.0}
          width={VIEW_BOX_SIZE - WALL_SIZE}
          height={VIEW_BOX_SIZE - WALL_SIZE}
          strokeWidth={WALL_SIZE}
          className={boardStyles.outerBorder}
        />
      }

      {
        // Render all vertical borders
        view.vertical_borders.map((rowArray, row) =>
          rowArray.map((active, column) => {
            const p1 = { row, column };
            const p2 = { row, column: column + 1 };
            const { x_mid, y_min, x_max, y_mid, y_max, x_min } = getWallPoints(
              p1,
              p2,
            );
            const dangling = view.error?.dangling_borders.some(
              (border) =>
                border.p1.row === row &&
                border.p1.column === column &&
                border.p2.column === column + 1,
            );
            const objective = view.objective.walls.some(
              (border) =>
                border.p1.row === row &&
                border.p1.column === column &&
                border.p2.column === column + 1,
            );

            const onClick = () => {
              if (!objective) {
                onToggle({ p1, p2 });
              }
            };
            return (
              <g
                key={`vertical-wall-${row}-${column}`}
                className={clsx(
                  boardStyles.wallGroup,
                  active && boardStyles.active,
                  dangling && boardStyles.dangling,
                  objective && boardStyles.objective,
                )}
                onClick={onClick}
              >
                <line
                  x1={x_mid}
                  y1={y_min}
                  x2={x_mid}
                  y2={y_max}
                  strokeWidth={WALL_SIZE}
                  className={boardStyles.wallLine}
                />
                <polygon
                  points={`${x_mid},${y_min} ${x_max},${y_mid} ${x_mid},${y_max} ${x_min},${y_mid}`}
                  className={boardStyles.wallTouch}
                />
              </g>
            );
          }),
        )
      }

      {
        // Render all horizontal borders
        view.horizontal_borders.map((rowArr, row) =>
          rowArr.map((active, column) => {
            const p1 = { row, column };
            const p2 = { row: row + 1, column };
            const { x_min, y_mid, x_max, y_min, y_max, x_mid } = getWallPoints(
              p1,
              p2,
            );
            const dangling = view.error?.dangling_borders.some(
              (border) =>
                border.p1.row === row &&
                border.p1.column === column &&
                border.p2.row === row + 1,
            );
            const objective = view.objective.walls.some(
              (border) =>
                border.p1.row === row &&
                border.p1.column === column &&
                border.p2.row === row + 1,
            );

            const onClick = () => {
              if (!objective) {
                onToggle({ p1, p2 });
              }
            };
            return (
              <g
                key={`vertical-wall-${row}-${column}`}
                className={clsx(
                  boardStyles.wallGroup,
                  active && boardStyles.active,
                  dangling && boardStyles.dangling,
                  objective && boardStyles.objective,
                )}
                onClick={onClick}
              >
                <line
                  x1={x_min}
                  y1={y_mid}
                  x2={x_max}
                  y2={y_mid}
                  strokeWidth={WALL_SIZE}
                  className={boardStyles.wallLine}
                />
                <polygon
                  points={`${x_mid},${y_min} ${x_max},${y_mid} ${x_mid},${y_max} ${x_min},${y_mid}`}
                  className={boardStyles.wallTouch}
                />
              </g>
            );
          }),
        )
      }

      {
        // Render galaxy centers
        view.objective.centers.map((center, i) => {
          const cx =
            WALL_SIZE / 2.0 +
            ((WALL_SIZE + CELL_SIZE) / 2.0) * (center.position.column + 1);
          const cy =
            WALL_SIZE / 2.0 +
            ((WALL_SIZE + CELL_SIZE) / 2.0) * (center.position.row + 1);
          const r = CELL_SIZE / 2.5 - WALL_SIZE;

          const cut = view.error?.cut_centers.some(
            (p) =>
              p.row === center.position.row &&
              p.column === center.position.column,
          );
          const asymmetric = view.error?.asymmetric_centers.some(
            (p) =>
              p.row === center.position.row &&
              p.column === center.position.column,
          );

          return (
            <g
              key={`galaxy-center-${i}`}
              className={clsx(
                boardStyles.galaxyCenter,
                cut && boardStyles.cut,
                asymmetric && boardStyles.asymmetric,
              )}
            >
              <circle cx={cx} cy={cy} r={r} />
              {center.size && (
                <text x={cx} y={cy}>
                  {center.size}
                </text>
              )}
            </g>
          );
        })
      }
    </svg>
  );
}

export default App;
