import { useCallback, useEffect, useRef, useState } from 'react';
import type { GameHandle } from '../wasm-pkg/pokemon_shiritori';
import type { GameConfig } from '../App';
import { getArtworkUrl, getType, TYPE_STYLES } from '../gen1data';

interface MoveResult { ok: boolean; error?: string; name?: string; }
interface CpuResult  { name: string | null; lost: boolean; }
interface HistEntry  { name: string; by_human: boolean; }

interface Move {
  name: string;
  byHuman: boolean;
}

interface Props {
  game: GameHandle;
  config: GameConfig;
  onGameOver: (humanWon: boolean) => void;
}

type Mode = 'entry' | 'picker';

export default function GameScreen({ game, config, onGameOver }: Props) {
  const [humanTurn, setHumanTurn] = useState(() => game.is_human_turn());
  const [isOver, setIsOver] = useState(false);
  const [cpuThinking, setCpuThinking] = useState(false);
  const [lastPlayed, setLastPlayed] = useState<Move | null>(null);
  const [requiredLetter, setRequiredLetter] = useState<string | null>(
    () => game.required_letter() as string | null
  );
  const [usedCount, setUsedCount] = useState(() => game.used_count());
  const [remainingCount, setRemainingCount] = useState(() => game.remaining_count());
  const [legalNames, setLegalNames] = useState<string[]>(() => game.legal_names() as string[]);
  const [history, setHistory] = useState<Move[]>([]);

  const [mode, setMode] = useState<Mode>('entry');
  const [input, setInput] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [hint, setHint] = useState<string | null>(null);
  const [pickerSearch, setPickerSearch] = useState('');

  const inputRef = useRef<HTMLInputElement>(null);
  const historyEndRef = useRef<HTMLDivElement>(null);

  // Sync game state from WASM into local React state after each move.
  const syncState = useCallback(() => {
    setHumanTurn(game.is_human_turn());
    setIsOver(game.is_over());
    setRequiredLetter(game.required_letter() as string | null);
    setUsedCount(game.used_count());
    setRemainingCount(game.remaining_count());
    setLegalNames(game.legal_names() as string[]);
    const h = game.history_json() as HistEntry[];
    setHistory(h.map(e => ({ name: e.name, byHuman: e.by_human })));
    setHint(null);
    setInput('');
    setError(null);
    setPickerSearch('');
  }, [game]);

  // Trigger CPU move whenever it's the CPU's turn.
  useEffect(() => {
    if (isOver || humanTurn) return;
    setCpuThinking(true);
    const timer = setTimeout(() => {
      const result = game.cpu_take_turn() as CpuResult;
      setCpuThinking(false);
      if (result.lost || !result.name) {
        syncState();
        onGameOver(true);
      } else {
        setLastPlayed({ name: result.name, byHuman: false });
        syncState();
        if (game.is_over()) onGameOver(game.human_won());
      }
    }, 800);
    return () => clearTimeout(timer);
  }, [humanTurn, isOver, game, onGameOver, syncState]);

  // If CPU goes first, trigger immediately on mount.
  useEffect(() => {
    if (!game.is_human_turn() && !game.is_over()) {
      setHumanTurn(false);
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  function submitMove(name: string) {
    if (!name.trim()) return;
    const result = game.apply_human_move(name) as MoveResult;
    if (!result.ok) {
      setError(result.error ?? 'Invalid move');
      return;
    }
    setLastPlayed({ name: result.name!, byHuman: true });
    syncState();
    if (game.is_over()) onGameOver(game.human_won());
    setTimeout(() => inputRef.current?.focus(), 100);
  }

  function handleEntrySubmit(e: React.FormEvent) {
    e.preventDefault();
    submitMove(input);
  }

  function handleHint() {
    if (hint !== null) { setHint(null); return; }
    const h = game.hint() as string | null;
    setHint(h);
  }

  const filteredLegal = legalNames.filter(n =>
    n.toLowerCase().includes(pickerSearch.toLowerCase())
  );
  const topSuggestions = legalNames.slice(0, 4);

  const artUrl = lastPlayed ? getArtworkUrl(lastPlayed.name) : null;
  const pokéType = lastPlayed ? getType(lastPlayed.name) : null;
  const typeStyle = pokéType ? (TYPE_STYLES[pokéType] ?? { bg: '#888', text: '#fff' }) : null;

  const lastLetter = lastPlayed
    ? lastPlayed.name.replace(/[^a-zA-Z]/g, '').slice(-1).toUpperCase()
    : null;
  const firstLetter = lastPlayed
    ? lastPlayed.name.replace(/[^a-zA-Z]/g, '')[0]?.toUpperCase()
    : null;

  const turnLabel = cpuThinking
    ? `${game.cpu_name()} is thinking…`
    : humanTurn
    ? 'Your Turn'
    : `${game.cpu_name()}'s Turn`;

  const totalInPool = config.count;

  return (
    <div>
      {/* Status bar */}
      <div className="game-status-bar">
        <div className="turn-indicator">
          <div className={`turn-dot ${cpuThinking ? 'thinking' : !humanTurn ? 'cpu' : ''}`} />
          <span className="turn-label">{turnLabel}</span>
        </div>
        <span className="used-count">
          Used <strong>{usedCount}</strong> / {totalInPool}
        </span>
      </div>

      {/* Last played card */}
      <div className="card last-played-card">
        <div className="last-played-tag">Last Played</div>
        {lastPlayed ? (
          <>
            <div className="pokemon-art-wrap">
              {artUrl ? (
                <img
                  className="pokemon-art"
                  src={artUrl}
                  alt={lastPlayed.name}
                  loading="lazy"
                  onError={e => { (e.target as HTMLImageElement).style.display = 'none'; }}
                />
              ) : (
                <div className="pokemon-art-placeholder">🎮</div>
              )}
            </div>
            <div className="pokemon-name-row">
              {typeStyle && pokéType && (
                <span
                  className="type-badge"
                  style={{ background: typeStyle.bg, color: typeStyle.text }}
                >
                  {pokéType}
                </span>
              )}
              <span className="pokemon-name">{lastPlayed.name}</span>
            </div>
            <div className="letter-info">
              <div className="letter-field">
                <span className="letter-field-label">Ends with</span>
                <span className="letter-value">{lastLetter}</span>
              </div>
              <span className="letter-dash">—</span>
              <div className="letter-field">
                <span className="letter-field-label">Next starts</span>
                <span className="letter-value">{requiredLetter ?? '?'}</span>
              </div>
            </div>
          </>
        ) : (
          <div style={{ textAlign: 'center', padding: '24px 0', color: 'var(--text-3)' }}>
            <div style={{ fontSize: 48, marginBottom: 8 }}>🎮</div>
            <p style={{ fontSize: 14 }}>
              {humanTurn
                ? 'You go first! Name any Gen 1 Pokémon.'
                : `${game.cpu_name()} goes first…`}
            </p>
          </div>
        )}
      </div>

      {/* Input area */}
      {humanTurn && !isOver && !cpuThinking && (
        <div>
          <div className="input-controls">
            <div className="mode-toggle">
              <button
                className={`mode-btn ${mode === 'entry' ? 'active' : ''}`}
                onClick={() => setMode('entry')}
              >
                Entry
              </button>
              <button
                className={`mode-btn ${mode === 'picker' ? 'active' : ''}`}
                onClick={() => setMode('picker')}
              >
                Picker
              </button>
            </div>
            <button
              className={`hint-btn ${hint !== null ? 'revealed' : ''}`}
              onClick={handleHint}
            >
              💡 {hint !== null ? hint : 'Hint'}
            </button>
          </div>

          {hint !== null && (
            <div className="hint-text">
              <span>💡</span>
              <span>Engine suggests: <strong>{hint}</strong></span>
            </div>
          )}

          {mode === 'entry' ? (
            <>
              {/* Top legal suggestions as chips */}
              {topSuggestions.length > 0 && (
                <div className="legal-chips">
                  {topSuggestions.map(name => (
                    <button
                      key={name}
                      className="legal-chip"
                      onClick={() => submitMove(name)}
                    >
                      {name}
                      {(() => {
                        const t = getType(name);
                        const s = TYPE_STYLES[t];
                        return s ? (
                          <span className="type-badge" style={{ background: s.bg, color: s.text, fontSize: 10 }}>
                            {t}
                          </span>
                        ) : null;
                      })()}
                    </button>
                  ))}
                </div>
              )}
              <form onSubmit={handleEntrySubmit} className="entry-row">
                <input
                  ref={inputRef}
                  className="entry-input"
                  type="text"
                  value={input}
                  onChange={e => { setInput(e.target.value); setError(null); }}
                  placeholder={
                    requiredLetter
                      ? `Enter Pokémon name starting with ${requiredLetter}…`
                      : 'Enter any Pokémon name…'
                  }
                  autoComplete="off"
                  autoFocus
                />
                <button type="submit" className="submit-btn" disabled={!input.trim()}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                    <line x1="5" y1="12" x2="19" y2="12"/>
                    <polyline points="12 5 19 12 12 19"/>
                  </svg>
                </button>
              </form>
              {error && <div className="error-msg">{error}</div>}
            </>
          ) : (
            <>
              <input
                className="picker-search"
                type="text"
                placeholder="Search legal Pokémon…"
                value={pickerSearch}
                onChange={e => setPickerSearch(e.target.value)}
                autoFocus
              />
              {filteredLegal.length === 0 ? (
                <p style={{ color: 'var(--text-3)', fontSize: 14, textAlign: 'center', padding: 20 }}>
                  No legal moves found.
                </p>
              ) : (
                <div className="picker-list">
                  {filteredLegal.map(name => {
                    const t = getType(name);
                    const s = TYPE_STYLES[t];
                    return (
                      <button key={name} className="picker-item" onClick={() => submitMove(name)}>
                        <span>{name}</span>
                        {s && (
                          <span className="type-badge" style={{ background: s.bg, color: s.text, fontSize: 11 }}>
                            {t}
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              )}
            </>
          )}
        </div>
      )}

      {/* CPU thinking indicator */}
      {cpuThinking && (
        <div className="cpu-thinking-msg">
          <div className="spinner" style={{ width: 20, height: 20, borderWidth: 2 }} />
          {game.cpu_name()} is thinking…
        </div>
      )}

      {/* Chain history */}
      {history.length > 0 && (
        <div className="chain-section">
          <div className="chain-header">
            <span className="section-label">Chain History</span>
            <span style={{ fontSize: 11, color: 'var(--text-3)' }}>{history.length} moves</span>
          </div>
          <div className="chain-scroll">
            {history.map((move, i) => (
              <div key={i} className={`chain-card ${move.byHuman ? 'human' : 'cpu'}`}>
                <div className="chain-player">
                  {i === 0 ? 'Start' : move.byHuman ? 'You' : game.cpu_name()}
                </div>
                <div className="chain-pokemon">{move.name}</div>
                <div className="chain-ends">
                  Ends: {move.name.replace(/[^a-zA-Z]/g, '').slice(-1).toUpperCase()}
                </div>
              </div>
            ))}
            <div ref={historyEndRef} />
          </div>
        </div>
      )}
    </div>
  );
}
