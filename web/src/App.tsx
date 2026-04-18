import { useEffect, useRef, useState } from 'react';
import type { GameHandle } from './wasm-pkg/pokemon_shiritori';
import type { Records } from './types';
import SetupScreen from './components/SetupScreen';
import GameScreen from './components/GameScreen';
import VictoryScreen from './components/VictoryScreen';
import RulesModal from './components/RulesModal';
import SlidePanel from './components/SlidePanel';
import PokedexPanel from './components/PokedexPanel';
import RecordsPanel from './components/RecordsPanel';

export interface GameConfig {
  agent: string;
  rollouts: number;
  humanFirst: boolean;
  /** National dex generations 1–6 included in the pool (subset). */
  generations: number[];
}

export type { Records } from './types';

type Screen = 'loading' | 'setup' | 'playing' | 'over';
type LibraryOverlay = 'pokedex' | 'records' | null;

const RECORDS_KEY = 'shiritori-records';
const THEME_KEY = 'shiritori-theme';

function readStoredTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light';
  const s = localStorage.getItem(THEME_KEY);
  if (s === 'dark' || s === 'light') return s;
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

const EMPTY_RECORDS: Records = { wins: 0, losses: 0, streak: 0, bestStreak: 0 };

function loadRecords(): Records {
  try {
    const raw = localStorage.getItem(RECORDS_KEY);
    if (raw) return JSON.parse(raw) as Records;
  } catch {}
  return { ...EMPTY_RECORDS };
}

function saveRecords(r: Records) {
  localStorage.setItem(RECORDS_KEY, JSON.stringify(r));
}

export default function App() {
  const [screen, setScreen] = useState<Screen>('loading');
  const [libraryOverlay, setLibraryOverlay] = useState<LibraryOverlay>(null);
  const [showRules, setShowRules] = useState(false);
  const [config, setConfig] = useState<GameConfig>({
    agent: 'deadend',
    rollouts: 30,
    humanFirst: true,
    generations: [1, 2, 3, 4, 5, 6],
  });
  const [records, setRecords] = useState<Records>(loadRecords);
  const gameRef = useRef<GameHandle | null>(null);
  const [gameKey, setGameKey] = useState(0);
  const startTimeRef = useRef<number>(0);
  const [elapsedMs, setElapsedMs] = useState(0);
  const [theme, setTheme] = useState<'light' | 'dark'>(readStoredTheme);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    try {
      localStorage.setItem(THEME_KEY, theme);
    } catch {}
  }, [theme]);

  // Load WASM module on mount.
  useEffect(() => {
    import('./wasm-pkg/pokemon_shiritori')
      .then(mod => mod.default())
      .then(() => setScreen('setup'))
      .catch(err => console.error('WASM init failed', err));
  }, []);

  useEffect(() => {
    if (!libraryOverlay) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setLibraryOverlay(null);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [libraryOverlay]);

  // When returning to the home screen, release the WASM game handle (playing or finished match).
  useEffect(() => {
    if (screen !== 'setup') return;
    const g = gameRef.current;
    if (g) {
      g.free();
      gameRef.current = null;
    }
  }, [screen]);

  function startGame(cfg: GameConfig) {
    import('./wasm-pkg/pokemon_shiritori').then(mod => {
      const prev = gameRef.current;
      if (prev) prev.free();
      gameRef.current = new mod.GameHandle(
        cfg.agent,
        4,
        cfg.rollouts,
        cfg.generations.join(','),
        cfg.humanFirst,
      );
      setConfig(cfg);
      setGameKey(k => k + 1);
      startTimeRef.current = Date.now();
      setLibraryOverlay(null);
      setScreen('playing');
    });
  }

  function handleGameOver(humanWon: boolean) {
    const elapsed = Date.now() - startTimeRef.current;
    setElapsedMs(elapsed);
    const r = loadRecords();
    if (humanWon) {
      r.wins += 1;
      r.streak += 1;
      r.bestStreak = Math.max(r.bestStreak, r.streak);
    } else {
      r.losses += 1;
      r.streak = 0;
    }
    saveRecords(r);
    setRecords(r);
    setScreen('over');
  }

  function handleRestart() {
    setScreen('setup');
  }

  function exitToHome() {
    setLibraryOverlay(null);
    setScreen('setup');
  }

  function handleWipeRecords() {
    if (
      !window.confirm(
        'Clear all wins, losses, and streak stats? This cannot be undone.',
      )
    ) {
      return;
    }
    saveRecords(EMPTY_RECORDS);
    setRecords({ ...EMPTY_RECORDS });
  }

  const game = gameRef.current;

  return (
    <div className="app-shell">
      {showRules && <RulesModal onClose={() => setShowRules(false)} />}

      {libraryOverlay === 'pokedex' && (
        <SlidePanel title="Pokédex" onClose={() => setLibraryOverlay(null)}>
          <PokedexPanel generations={config.generations} />
        </SlidePanel>
      )}
      {libraryOverlay === 'records' && (
        <SlidePanel title="Records" onClose={() => setLibraryOverlay(null)}>
          <RecordsPanel records={records} onWipeRecords={handleWipeRecords} />
        </SlidePanel>
      )}

      <header className="top-bar">
        <button
          type="button"
          className="logo logo-home"
          onClick={() => {
            if (screen === 'loading') return;
            exitToHome();
          }}
          aria-label="Home"
        >
          Shiritori
        </button>
        <div className="top-bar-actions">
          <button className="icon-btn" onClick={() => setShowRules(true)} title="How to play">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4M12 8h.01"/>
            </svg>
          </button>
          <button
            type="button"
            className="icon-btn"
            title={theme === 'light' ? 'Dark mode' : 'Light mode'}
            aria-label={theme === 'light' ? 'Use dark theme' : 'Use light theme'}
            onClick={() => setTheme(t => (t === 'light' ? 'dark' : 'light'))}
          >
            {theme === 'light' ? (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden>
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
              </svg>
            ) : (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden>
                <circle cx="12" cy="12" r="4" />
                <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
              </svg>
            )}
          </button>
        </div>
      </header>

      <main className="page">
        {screen === 'loading' && (
          <div className="loading-screen">
            <div className="loading-logo">Shiritori</div>
            <div className="spinner" />
            <p style={{ color: 'var(--text-3)', fontSize: 14 }}>Loading engine…</p>
          </div>
        )}

        {screen === 'setup' && (
          <SetupScreen config={config} onStart={startGame} />
        )}

        {screen === 'playing' && game && (
          <GameScreen
            key={gameKey}
            game={game}
            onGameOver={handleGameOver}
            onExit={exitToHome}
          />
        )}

        {screen === 'over' && game && (
          <VictoryScreen
            game={game}
            records={records}
            elapsedMs={elapsedMs}
            onRestart={handleRestart}
          />
        )}

      </main>

      {screen !== 'loading' && (
        <nav className="bottom-nav">
          <button
            className={`nav-item ${libraryOverlay === 'pokedex' ? 'active' : ''}`}
            onClick={() =>
              setLibraryOverlay(o => (o === 'pokedex' ? null : 'pokedex'))
            }
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="2" y="3" width="20" height="14" rx="2"/>
              <path d="M8 21h8M12 17v4"/>
            </svg>
            Pokédex
          </button>

          <button
            className="battle-fab"
            onClick={() => setLibraryOverlay(null)}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
              <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
            </svg>
            Battle
          </button>

          <button
            className={`nav-item ${libraryOverlay === 'records' ? 'active' : ''}`}
            onClick={() =>
              setLibraryOverlay(o => (o === 'records' ? null : 'records'))
            }
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
            </svg>
            Records
          </button>
        </nav>
      )}
    </div>
  );
}
