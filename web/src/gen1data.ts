// Gen 1 Pokémon (national dex order, index+1 = dex ID).
export const GEN1_NAMES: string[] = [
  "Bulbasaur","Ivysaur","Venusaur","Charmander","Charmeleon","Charizard",
  "Squirtle","Wartortle","Blastoise","Caterpie","Metapod","Butterfree",
  "Weedle","Kakuna","Beedrill","Pidgey","Pidgeotto","Pidgeot",
  "Rattata","Raticate","Spearow","Fearow","Ekans","Arbok",
  "Pikachu","Raichu","Sandshrew","Sandslash","Nidoran♀","Nidorina",
  "Nidoqueen","Nidoran♂","Nidorino","Nidoking","Clefairy","Clefable",
  "Vulpix","Ninetales","Jigglypuff","Wigglytuff","Zubat","Golbat",
  "Oddish","Gloom","Vileplume","Paras","Parasect","Venonat",
  "Venomoth","Diglett","Dugtrio","Meowth","Persian","Psyduck",
  "Golduck","Mankey","Primeape","Growlithe","Arcanine","Poliwag",
  "Poliwhirl","Poliwrath","Abra","Kadabra","Alakazam","Machop",
  "Machoke","Machamp","Bellsprout","Weepinbell","Victreebel","Tentacool",
  "Tentacruel","Geodude","Graveler","Golem","Ponyta","Rapidash",
  "Slowpoke","Slowbro","Magnemite","Magneton","Farfetch'd","Doduo",
  "Dodrio","Seel","Dewgong","Grimer","Muk","Shellder",
  "Cloyster","Gastly","Haunter","Gengar","Onix","Drowzee",
  "Hypno","Krabby","Kingler","Voltorb","Electrode","Exeggcute",
  "Exeggutor","Cubone","Marowak","Hitmonlee","Hitmonchan","Lickitung",
  "Koffing","Weezing","Rhyhorn","Rhydon","Chansey","Tangela",
  "Kangaskhan","Horsea","Seadra","Goldeen","Seaking","Staryu",
  "Starmie","Mr. Mime","Scyther","Jynx","Electabuzz","Magmar",
  "Pinsir","Tauros","Magikarp","Gyarados","Lapras","Ditto",
  "Eevee","Vaporeon","Jolteon","Flareon","Porygon","Omanyte",
  "Omastar","Kabuto","Kabutops","Aerodactyl","Snorlax","Articuno",
  "Zapdos","Moltres","Dratini","Dragonair","Dragonite","Mewtwo","Mew",
];

// Primary type per Pokémon (Gen 1 classification).
export const GEN1_PRIMARY_TYPES: Record<string, string> = {
  Bulbasaur:"Grass",Ivysaur:"Grass",Venusaur:"Grass",
  Charmander:"Fire",Charmeleon:"Fire",Charizard:"Fire",
  Squirtle:"Water",Wartortle:"Water",Blastoise:"Water",
  Caterpie:"Bug",Metapod:"Bug",Butterfree:"Bug",
  Weedle:"Bug",Kakuna:"Bug",Beedrill:"Bug",
  Pidgey:"Normal",Pidgeotto:"Normal",Pidgeot:"Normal",
  Rattata:"Normal",Raticate:"Normal",Spearow:"Normal",Fearow:"Normal",
  Ekans:"Poison",Arbok:"Poison",
  Pikachu:"Electric",Raichu:"Electric",
  Sandshrew:"Ground",Sandslash:"Ground",
  "Nidoran♀":"Poison",Nidorina:"Poison",Nidoqueen:"Poison",
  "Nidoran♂":"Poison",Nidorino:"Poison",Nidoking:"Poison",
  Clefairy:"Normal",Clefable:"Normal",
  Vulpix:"Fire",Ninetales:"Fire",
  Jigglypuff:"Normal",Wigglytuff:"Normal",
  Zubat:"Poison",Golbat:"Poison",
  Oddish:"Grass",Gloom:"Grass",Vileplume:"Grass",
  Paras:"Bug",Parasect:"Bug",Venonat:"Bug",Venomoth:"Bug",
  Diglett:"Ground",Dugtrio:"Ground",
  Meowth:"Normal",Persian:"Normal",
  Psyduck:"Water",Golduck:"Water",
  Mankey:"Fighting",Primeape:"Fighting",
  Growlithe:"Fire",Arcanine:"Fire",
  Poliwag:"Water",Poliwhirl:"Water",Poliwrath:"Water",
  Abra:"Psychic",Kadabra:"Psychic",Alakazam:"Psychic",
  Machop:"Fighting",Machoke:"Fighting",Machamp:"Fighting",
  Bellsprout:"Grass",Weepinbell:"Grass",Victreebel:"Grass",
  Tentacool:"Water",Tentacruel:"Water",
  Geodude:"Rock",Graveler:"Rock",Golem:"Rock",
  Ponyta:"Fire",Rapidash:"Fire",
  Slowpoke:"Water",Slowbro:"Water",
  Magnemite:"Electric",Magneton:"Electric",
  "Farfetch'd":"Normal",Doduo:"Normal",Dodrio:"Normal",
  Seel:"Water",Dewgong:"Water",
  Grimer:"Poison",Muk:"Poison",
  Shellder:"Water",Cloyster:"Water",
  Gastly:"Ghost",Haunter:"Ghost",Gengar:"Ghost",
  Onix:"Rock",
  Drowzee:"Psychic",Hypno:"Psychic",
  Krabby:"Water",Kingler:"Water",
  Voltorb:"Electric",Electrode:"Electric",
  Exeggcute:"Grass",Exeggutor:"Grass",
  Cubone:"Ground",Marowak:"Ground",
  Hitmonlee:"Fighting",Hitmonchan:"Fighting",
  Lickitung:"Normal",
  Koffing:"Poison",Weezing:"Poison",
  Rhyhorn:"Ground",Rhydon:"Ground",
  Chansey:"Normal",Tangela:"Grass",Kangaskhan:"Normal",
  Horsea:"Water",Seadra:"Water",
  Goldeen:"Water",Seaking:"Water",
  Staryu:"Water",Starmie:"Water",
  "Mr. Mime":"Psychic",
  Scyther:"Bug",Jynx:"Ice",
  Electabuzz:"Electric",Magmar:"Fire",Pinsir:"Bug",
  Tauros:"Normal",Magikarp:"Water",Gyarados:"Water",
  Lapras:"Water",Ditto:"Normal",Eevee:"Normal",
  Vaporeon:"Water",Jolteon:"Electric",Flareon:"Fire",
  Porygon:"Normal",
  Omanyte:"Rock",Omastar:"Rock",Kabuto:"Rock",Kabutops:"Rock",
  Aerodactyl:"Rock",Snorlax:"Normal",
  Articuno:"Ice",Zapdos:"Electric",Moltres:"Fire",
  Dratini:"Dragon",Dragonair:"Dragon",Dragonite:"Dragon",
  Mewtwo:"Psychic",Mew:"Psychic",
};

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
  Dragon:   { bg: "#5820E8", text: "#fff" },
  Psychic:  { bg: "#D81870", text: "#fff" },
};

export function getDexId(name: string): number {
  const idx = GEN1_NAMES.indexOf(name);
  return idx >= 0 ? idx + 1 : 0;
}

export function getArtworkUrl(name: string): string {
  const id = getDexId(name);
  if (id === 0) return "";
  return `https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/other/official-artwork/${id}.png`;
}

export function getType(name: string): string {
  return GEN1_PRIMARY_TYPES[name] ?? "Normal";
}
