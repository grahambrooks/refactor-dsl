/**
 * MyLib v2.0.0 - A sample TypeScript library for demonstrating API upgrades.
 *
 * Breaking changes from v1.0.0:
 * - `User` renamed to `UserAccount` with new `email` field
 * - `Status` renamed to `ConnectionStatus`
 * - `getUser` renamed to `fetchUser`
 * - `processData` renamed to `transformData`
 * - `connect` now requires `port` and `timeout` parameters
 * - `save` no longer takes `sync` parameter
 * - `query` parameters reordered from (a, b, c) to (c, a, b)
 * - `find` now returns `Item | null` instead of `Item`
 * - `parse` now returns `ParseResult` instead of `string`
 * - `deprecatedFn` has been removed
 * - `utils` namespace renamed to `helpers`
 */

/** A user account in the system (renamed from User). */
export interface UserAccount {
  id: number;
  name: string;
  email: string; // New field in v2
}

/** Connection status (renamed from Status). */
export type ConnectionStatus = 'connected' | 'disconnected' | 'error';

/** Configuration for the library (with new port field). */
export interface Config {
  host: string;
  port: number; // New field in v2
}

/** Database query result item. */
export interface Item {
  key: string;
  value: string;
}

/** Result of parsing operations (new in v2). */
export interface ParseResult {
  output: string;
  warnings: string[];
}

/** Fetch a user by their ID (renamed from getUser). */
export function fetchUser(id: number): UserAccount | null {
  if (id > 0) {
    return { id, name: `User${id}`, email: `user${id}@example.com` };
  }
  return null;
}

/** Transform raw data (renamed from processData). */
export function transformData(data: number[]): number[] {
  return data.map(b => b + 1);
}

/** Connect to a host with port and timeout (signature changed). */
export async function connect(host: string, port: number, timeout: number): Promise<ConnectionStatus> {
  if (!host) {
    throw new Error('empty host');
  }
  if (port <= 0) {
    throw new Error('invalid port');
  }
  return 'connected';
}

/** Save data (sync parameter removed). */
export async function save(data: string): Promise<void> {
  console.log('Saved:', data);
}

/** Query with reordered parameters (c, a, b instead of a, b, c). */
export function query(c: boolean, a: string, b: number): Item[] {
  const results: Item[] = [];
  if (c) {
    results.push({ key: a, value: String(b) });
  }
  return results;
}

/** Find an item by key (now returns Item | null). */
export function find(key: string): Item | null {
  if (!key) {
    return null;
  }
  return { key, value: 'found' };
}

/** Parse a string into a ParseResult (return type changed). */
export function parse(input: string): ParseResult {
  if (!input) {
    return { output: '', warnings: ['empty input'] };
  }
  return { output: input.toUpperCase(), warnings: [] };
}

// Note: deprecatedFn has been removed in v2

/** Helper functions (renamed from utils). */
export namespace helpers {
  /** A helper function that returns a greeting. */
  export function helper(): string {
    return 'Hello from helpers';
  }

  /** Format a user account for display. */
  export function formatUser(user: UserAccount): string {
    return `UserAccount(id=${user.id}, name=${user.name}, email=${user.email})`;
  }
}
