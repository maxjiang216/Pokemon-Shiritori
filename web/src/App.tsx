import { useEffect, useRef, useState } from 'react';
import type { GameHandle } from './wasm-pkg/pokemon_shiritori';
import SetupScreen from './components/SetupScreen';
import GameScreen from './components/GameScreen';
import VictoryScreen from './components/VictoryScreen';
import RulesModal from './components/RulesModal';

export interface GameConfig {
  agent: string;
  rollouts: number;
  count: number;
  humanFirst: boolean;
}

export interface Records {
  wins: number;
  losses: number;
  streak: number;
  bestStreak: number;
}

type Screen = 'loading' | 'setup' | 'playing' | 'over';
type Tab = 'battle' | 'pokedex' | 'records';

const RECORDS_KEY = 'shiritori-records';

function loadRecords(): Records {
  try {
    const raw = localStorage.getItem(RECORDS_KEY);
    if (raw) return JSON.parse(raw) as Records;
  } catch {}
  return { wins: 0, losses: 0, streak: 0, bestStreak: 0 };
}

function saveRecords(r: Records) {
  localStorage.setItem(RECORDS_KEY, JSON.stringify(r));
}

export default function App() {
  const [screen, setScreen] = useState<Screen>('loading');
  const [tab, setTab] = useState<Tab>('battle');
  const [showRules, setShowRules] = useState(false);
  const [config, setConfig] = useState<GameConfig>({
    agent: 'hybrid',
    rollouts: 30,
    count: 151,
    humanFirst: true,
  });
  const [records, setRecords] = useState<Records>(loadRecords);
  const gameRef = useRef<GameHandle | null>(null);
  const [gameKey, setGameKey] = useState(0);
  const startTimeRef = useRef<number>(0);
  const [elapsedMs, setElapsedMs] = useState(0);

  // Load WASM module on mount.
  useEffect(() => {
    import('./wasm-pkg/pokemon_shiritori')
      .then(mod => mod.default())
      .then(() => setScreen('setup'))
      .catch(err => console.error('WASM init failed', err));
  }, []);

  function startGame(cfg: GameConfig) {
    import('./wasm-pkg/pokemon_shiritori').then(mod => {
      const prev = gameRef.current;
      if (prev) prev.free();
      gameRef.current = new mod.GameHandle(
        cfg.agent,
        4,
        cfg.rollouts,
        cfg.count,
        cfg.humanFirst,
      );
      setConfig(cfg);
      setGameKey(k => k + 1);
      startTimeRef.current = Date.now();
      setScreen('playing');
      setTab('battle');
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

  const game = gameRef.current;

  return (
    <div className="app-shell">
      {showRules && <RulesModal onClose={() => setShowRules(false)} />}

      <header className="top-bar">
        <span className="logo">Shiritori</span>
        <div className="top-bar-actions">
          <button className="icon-btn" onClick={() => setShowRules(true)} title="How to play">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4M12 8h.01"/>
            </svg>
          </button>
          <button className="icon-btn" title="Settings" disabled>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="3"/>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
            </svg>
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
          <SetupScreen config={config} records={records} onStart={startGame} />
        )}

        {screen === 'playing' && game && tab === 'battle' && (
          <GameScreen
            key={gameKey}
            game={game}
            config={config}
            onGameOver={handleGameOver}
          />
        )}

        {screen === 'over' && game && tab === 'battle' && (
          <VictoryScreen
            game={game}
            records={records}
            elapsedMs={elapsedMs}
            onRestart={handleRestart}
          />
        )}

        {tab === 'pokedex' && <PokedexTab count={config.count} />}
        {tab === 'records' && <RecordsTab records={records} />}
      </main>

      {screen !== 'loading' && (
        <nav className="bottom-nav">
          <button
            className={`nav-item ${tab === 'pokedex' ? 'active' : ''}`}
            onClick={() => setTab('pokedex')}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="2" y="3" width="20" height="14" rx="2"/>
              <path d="M8 21h8M12 17v4"/>
            </svg>
            Pokédex
          </button>

          <button className="battle-fab" onClick={() => { setTab('battle'); if (screen === 'over') setScreen('over'); }}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
              <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
            </svg>
            Battle
          </button>

          <button
            className={`nav-item ${tab === 'records' ? 'active' : ''}`}
            onClick={() => setTab('records')}
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

// ── Pokédex stub ───────────────────────────────────────────────────────────

import { GEN1_NAMES, getArtworkUrl } from './gen1data';

function PokedexTab({ count }: { count: number }) {
  const names = GEN1_NAMES.slice(0, count);
  return (
    <div>
      <div className="section-label" style={{ marginBottom: 16 }}>
        {count} Pokémon in pool
      </div>
      <div className="pokedex-grid">
        {names.map((name, i) => (
          <div key={name} className="pokedex-item">
            <img
              src={getArtworkUrl(name)}
              alt={name}
              loading="lazy"
              onError={e => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
            />
            <div className="pokedex-item-name">#{i + 1} {name}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ── Records stub ───────────────────────────────────────────────────────────

function RecordsTab({ records }: { records: Records }) {
  const total = records.wins + records.losses;
  const pct = total > 0 ? Math.round((records.wins / total) * 100) : 0;
  return (
    <div className="records-page">
      <div className="records-hero">
        <div className="records-hero-num">{pct}%</div>
        <div className="records-hero-label">Win rate</div>
      </div>
      <div className="records-row">
        <div className="stat-card" style={{ flex: 1 }}>
          <div className="stat-label">Wins</div>
          <div className="stat-value" style={{ color: 'var(--green)' }}>{records.wins}</div>
        </div>
        <div className="stat-card" style={{ flex: 1 }}>
          <div className="stat-label">Losses</div>
          <div className="stat-value" style={{ color: 'var(--red)' }}>{records.losses}</div>
        </div>
      </div>
      <div className="records-row">
        <div className="stat-card highlight" style={{ flex: 1 }}>
          <div className="stat-label">Current streak</div>
          <div className="stat-value">{records.streak}</div>
        </div>
        <div className="stat-card" style={{ flex: 1 }}>
          <div className="stat-label">Best streak</div>
          <div className="stat-value">{records.bestStreak}</div>
        </div>
      </div>
    </div>
  );
}
