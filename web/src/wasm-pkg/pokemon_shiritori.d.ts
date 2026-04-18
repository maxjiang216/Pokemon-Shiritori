/* tslint:disable */
/* eslint-disable */

export class GameHandle {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Attempt to play `input` as a human move.
     * Returns `{ok: boolean, error?: string, name?: string}`.
     */
    apply_human_move(input: string): any;
    cpu_name(): string;
    /**
     * CPU takes its turn. Returns `{name: string|null, lost: boolean}`.
     * Call this when `is_human_turn()` is false and `is_over()` is false.
     */
    cpu_take_turn(): any;
    /**
     * Returns the CPU's preferred move name without applying it (for the hint feature).
     * Returns `null` if no legal moves exist.
     */
    hint(): any;
    /**
     * Move history as `Array<{name: string, by_human: boolean}>`.
     */
    history_json(): any;
    human_won(): boolean;
    is_human_turn(): boolean;
    is_over(): boolean;
    /**
     * Legal names for the current position, alphabetically sorted.
     */
    legal_names(): Array<any>;
    /**
     * Create a new game.
     *
     * - `agent`: one of `random`, `greedy`, `deadend`, `rollout`, `hybrid`
     * - `depth`: minimax depth for rollout/hybrid agents
     * - `rollouts`: random simulations per leaf for rollout/hybrid
     * - `count`: how many Gen 1 Pokémon to include (1–151)
     * - `human_first`: whether the human moves first
     */
    constructor(agent: string, depth: number, rollouts: number, count: number, human_first: boolean);
    /**
     * All pool names in national dex order — index + 1 gives the dex ID.
     */
    pool_names(): Array<any>;
    /**
     * Number of Pokémon still in play (not yet used).
     */
    remaining_count(): number;
    /**
     * Returns `null` (opening move) or an uppercase single character like `"U"`.
     */
    required_letter(): any;
    /**
     * Total moves played so far.
     */
    used_count(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_gamehandle_free: (a: number, b: number) => void;
    readonly gamehandle_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly gamehandle_is_human_turn: (a: number) => number;
    readonly gamehandle_is_over: (a: number) => number;
    readonly gamehandle_human_won: (a: number) => number;
    readonly gamehandle_required_letter: (a: number) => any;
    readonly gamehandle_cpu_name: (a: number) => [number, number];
    readonly gamehandle_remaining_count: (a: number) => number;
    readonly gamehandle_used_count: (a: number) => number;
    readonly gamehandle_pool_names: (a: number) => any;
    readonly gamehandle_legal_names: (a: number) => any;
    readonly gamehandle_history_json: (a: number) => any;
    readonly gamehandle_apply_human_move: (a: number, b: number, c: number) => any;
    readonly gamehandle_cpu_take_turn: (a: number) => any;
    readonly gamehandle_hint: (a: number) => any;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
