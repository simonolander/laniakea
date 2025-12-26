import { useEffect, useRef } from "react";
import styles from "./App.module.css";

import { generate_state, State } from "../rust/pkg";

function App() {
  const stateRef = useRef<State | null>(null);
  console.log(
    "State generated:",
    stateRef.current?.get_view(),
    stateRef.current,
  );

  useEffect(() => {
    const state = generate_state();
    stateRef.current = state;
    return () => state.free();
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
