import { useState } from 'react';
import type { GameConfig, Records } from '../App';

interface Agent {
  id: string;
  label: string;
  sub: string;
  tip: string;
}

const AGENTS: Agent[] = [
  { id: 'random',   label: 'Random',   sub: 'Unpredictable', tip: 'Plays completely at random — good for learning.' },
  { id: 'greedy',   label: 'Greedy',   sub: 'Max letters',   tip: 'Always tries to land on letters with fewest options.' },
  { id: 'deadend',  label: 'DeadEnd',  sub: 'Trap maker',    tip: 'Hunts dead-end letters using exact retrograde analysis.' },
  { id: 'rollout',  label: 'Rollout',  sub: 'Advanced engine', tip: 'Simulates thousands of random games to find the best move.' },
  { id: 'hybrid',   label: 'Hybrid',   sub: 'Balanced',      tip: 'Combines exact retrograde labels with rollout search.' },
  { id: 'exact',    label: 'Exact',    sub: 'Perfect play',  tip: 'Fully optimal — only available with small pools (≤ 15).' },
];

interface Props {
  config: GameConfig;
  records: Records;
  onStart: (cfg: GameConfig) => void;
}

export default function SetupScreen({ config, onStart }: Props) {
  const [agent, setAgent] = useState(config.agent);
  const [rollouts, setRollouts] = useState(config.rollouts);
  const [count, setCount] = useState(config.count);
  const [humanFirst, setHumanFirst] = useState(config.humanFirst);

  const selected = AGENTS.find(a => a.id === agent) ?? AGENTS[3];
  const pct = ((count - 1) / 150) * 100;
  const endName = count === 1 ? 'Bulbasaur' : count === 151 ? 'Mew' : `#${count}`;
  const startName = 'Bulbasaur';

  function handleStart() {
    const effectiveAgent = agent === 'exact' && count > 15 ? 'hybrid' : agent;
    onStart({ agent: effectiveAgent, rollouts, count, humanFirst });
  }

  return (
    <div>
      <div className="setup-intro">
        <div className="tag tag-blue" style={{ display: 'inline-flex', marginBottom: 14 }}>
          ⚔️ Battle Configuration
        </div>
        <h1 className="setup-title">Configure Your Challenge</h1>
        <p className="setup-subtitle">
          Prepare for the ultimate word battle across the Kanto region.
        </p>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <div className="setup-card-header">
          <h2>Opponent Strength</h2>
          <span className="tag tag-pink">AI Logic</span>
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

        <div className="slider-field">
          <div className="slider-header">
            <div>
              <label>Pokédex Pool Size</label>
              <div className="slider-sub">Limit the word list to the first N Pokémon.</div>
            </div>
            <div className="slider-value">{count}</div>
          </div>
          <input
            type="range"
            className="range-input"
            min={1}
            max={151}
            value={count}
            style={{ '--pct': `${pct}%` } as React.CSSProperties}
            onChange={e => setCount(Number(e.target.value))}
          />
          <div className="range-labels">
            <span>1 ({startName})</span>
            <span>151 ({endName !== 'Bulbasaur' ? endName : 'Mew'})</span>
          </div>
        </div>

        <button className="btn btn-primary" onClick={handleStart}>
          ⚡ Start Battle
        </button>

        {agent === 'exact' && count > 15 && (
          <p style={{ fontSize: 12, color: 'var(--red)', marginTop: 8, textAlign: 'center' }}>
            Exact agent requires pool ≤ 15. Will use Hybrid instead.
          </p>
        )}
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
