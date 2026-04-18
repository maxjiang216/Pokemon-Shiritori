import gensJson from './gens_1_6_en.json';
import PRIMARY_TYPES_BY_NATIONAL_DEX from './primary_types_1_721.json';

/** Keys "1" … "6" → English names in national dex order for that generation. */
export type GensJson = Record<string, string[]>;
const GENS = gensJson as GensJson;

/** Full national dex #1–721 in order (gens 1–6). */
export const NATIONAL_DEX_ORDER: string[] = (['1', '2', '3', '4', '5', '6'] as const).flatMap(
  k => GENS[k] ?? [],
);

/** Gen 1 names only (convenience). */
export const GEN1_NAMES: string[] = GENS['1'] ?? [];

/** Build the play pool in national-dex order for a subset of generations (1–6). */
export function poolNamesForGenerations(generations: number[]): string[] {
  const g = [...new Set(generations)]
    .filter(n => n >= 1 && n <= 6)
    .sort((a, b) => a - b);
  if (g.length === 0) {
    return poolNamesForGenerations([1, 2, 3, 4, 5, 6]);
  }
  return g.flatMap(gen => GENS[String(gen)] ?? []);
}

export function getNationalDexId(name: string): number {
  const i = NATIONAL_DEX_ORDER.indexOf(name);
  return i >= 0 ? i + 1 : 0;
}

export interface TypeStyle {
  bg: string;
  text: string;
}

export const TYPE_STYLES: Record<string, TypeStyle> = {
  Normal:   { bg: "#A0A090", text: "#fff" },
  Fire:     { bg: "#E8622A", text: "#fff" },
  Water:    { bg: "#4080E8", text: "#fff" },
  Grass:    { bg: "#50A840", text: "#fff" },
  Electric: { bg: "#D8A800", text: "#fff" },
  Ice:      { bg: "#70C8C8", text: "#fff" },
  Fighting: { bg: "#B82820", text: "#fff" },
  Poison:   { bg: "#8838A0", text: "#fff" },
  Ground:   { bg: "#B89020", text: "#fff" },
  Rock:     { bg: "#988828", text: "#fff" },
  Bug:      { bg: "#809020", text: "#fff" },
  Ghost:    { bg: "#583890", text: "#fff" },
  Steel:    { bg: "#A0A0C8", text: "#fff" },
  Dragon:   { bg: "#5820E8", text: "#fff" },
  Dark:     { bg: "#604848", text: "#fff" },
  Psychic:  { bg: "#D81870", text: "#fff" },
  Flying:   { bg: "#6890E8", text: "#fff" },
  Fairy:    { bg: "#E898A8", text: "#fff" },
};

export function getArtworkUrl(name: string): string {
  const id = getNationalDexId(name);
  if (id === 0) return "";
  return `https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/other/official-artwork/${id}.png`;
}

/** Primary type for national dex #1–721 (modern typings, incl. Fairy retcons). */
export function getType(name: string): string {
  const id = getNationalDexId(name);
  if (id >= 1 && id <= PRIMARY_TYPES_BY_NATIONAL_DEX.length) {
    return PRIMARY_TYPES_BY_NATIONAL_DEX[id - 1]!;
  }
  return "Normal";
}
