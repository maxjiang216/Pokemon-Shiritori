import { useEffect, useState } from 'react';
import type { GameConfig } from '../App';
import { poolNamesForGenerations } from '../gen1data';

interface Agent {
  id: string;
  label: string;
  sub: string;
  tip: string;
}

const AGENTS: Agent[] = [
  { id: 'random',   label: 'Random',   sub: 'Unpredictable', tip: 'Picks a legal move uniformly at random — good for learning the rules.' },
  { id: 'greedy',   label: 'Greedy',   sub: 'Max letters',   tip: 'Always tries to land on letters with fewest options.' },
  { id: 'deadend',  label: 'DeadEnd',  sub: 'Balanced',      tip: 'Retrograde + SCC check first; rollout search when the position is still messy.' },
  { id: 'rollout',  label: 'Rollout',  sub: 'Advanced engine', tip: 'Simulates thousands of random games to find the best move.' },
];

interface Props {
  config: GameConfig;
  onStart: (cfg: GameConfig) => void;
}

const DEFAULT_GENS = [1, 2, 3, 4, 5, 6];

export default function SetupScreen({ config, onStart }: Props) {
  const [agent, setAgent] = useState(config.agent);
  const [rollouts, setRollouts] = useState(config.rollouts);
  const [humanFirst, setHumanFirst] = useState(config.humanFirst);
  const [generations, setGenerations] = useState<number[]>(
    config.generations.length ? config.generations : DEFAULT_GENS,
  );

  useEffect(() => {
    setAgent(config.agent);
    setRollouts(config.rollouts);
    setHumanFirst(config.humanFirst);
    setGenerations(config.generations.length ? config.generations : DEFAULT_GENS);
  }, [config]);

  const selected = AGENTS.find(a => a.id === agent) ?? AGENTS[2];

  function toggleGeneration(g: number) {
    setGenerations(prev => {
      if (prev.includes(g)) {
        if (prev.length <= 1) return prev;
        return prev.filter(x => x !== g);
      }
      return [...prev, g].sort((a, b) => a - b);
    });
  }

  function handleStart() {
    onStart({ agent, rollouts, humanFirst, generations });
  }

  const poolSize = poolNamesForGenerations(generations).length;

  return (
    <div>
      <div className="setup-intro">
        <div className="tag tag-blue" style={{ display: 'inline-flex', marginBottom: 14 }}>
          ⚔️ Battle Configuration
        </div>
        <h1 className="setup-title">Configure Your Challenge</h1>
        <p className="setup-subtitle">
          Choose generations 1–6 and battle with the national dex pool you want. Random openings
          always leave your opponent a reply (no instant dead-letter wins on turn one).
        </p>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <div className="setup-card-header">
          <h2>Generations</h2>
          <span className="tag tag-blue">{poolSize} Pokémon</span>
        </div>
        <p className="setup-gens-caption">Which generations are in the word pool?</p>
        <div className="gen-chip-row">
          {[1, 2, 3, 4, 5, 6].map(g => (
            <button
              key={g}
              type="button"
              className={`gen-chip ${generations.includes(g) ? 'active' : ''}`}
              onClick={() => toggleGeneration(g)}
            >
              Gen {g}
            </button>
          ))}
        </div>

        <div className="setup-card-header" style={{ marginTop: 22 }}>
          <h2>CPU engine</h2>
          <span className="tag tag-pink">Strategy</span>
        </div>

        <div className="agent-grid">
          {AGENTS.map(a => (
            <button
              key={a.id}
              className={`agent-btn ${agent === a.id ? 'active' : ''}`}
              onClick={() => setAgent(a.id)}
            >
              <span className="agent-name">{a.label}</span>
              <span className="agent-sub">{a.sub}</span>
            </button>
          ))}
        </div>

        <div className="setup-row">
          <div className="setup-field">
            <label>Who goes first?</label>
            <div className="toggle-group">
              <button
                className={`toggle-btn ${humanFirst ? 'active' : ''}`}
                onClick={() => setHumanFirst(true)}
              >
                Player
              </button>
              <button
                className={`toggle-btn ${!humanFirst ? 'active' : ''}`}
                onClick={() => setHumanFirst(false)}
              >
                CPU
              </button>
            </div>
          </div>

          <div className="setup-field">
            <label>Engine depth</label>
            <div className="number-input-wrap">
              <input
                type="number"
                min={1}
                max={10000}
                value={rollouts}
                onChange={e => setRollouts(Math.max(1, Math.min(10000, Number(e.target.value) || 1)))}
              />
              <span className="unit-label">Rollouts</span>
            </div>
          </div>
        </div>

        <button className="btn btn-primary" onClick={handleStart}>
          ⚡ Start Battle
        </button>
      </div>

      <div className="pro-tip">
        <div className="pro-tip-icon">💡</div>
        <div className="pro-tip-text">
          <strong>Pro Tip</strong>
          <p>{selected.tip}</p>
        </div>
      </div>
    </div>
  );
}
