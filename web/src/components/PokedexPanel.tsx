import { getArtworkUrl, getNationalDexId, poolNamesForGenerations } from '../gen1data';

export default function PokedexPanel({ generations }: { generations: number[] }) {
  const names = poolNamesForGenerations(generations);
  const label =
    generations.length === 6
      ? 'Gens 1–6'
      : generations.length === 1
        ? `Gen ${generations[0]}`
        : `Gens ${generations.join(', ')}`;
  return (
    <div>
      <div className="section-label" style={{ marginBottom: 16 }}>
        {label} — {names.length} Pokémon
      </div>
      <div className="pokedex-grid">
        {names.map(name => (
          <div key={name} className="pokedex-item">
            <img
              src={getArtworkUrl(name)}
              alt={name}
              loading="lazy"
              onError={e => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
            />
            <div className="pokedex-item-name">#{getNationalDexId(name)} {name}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
