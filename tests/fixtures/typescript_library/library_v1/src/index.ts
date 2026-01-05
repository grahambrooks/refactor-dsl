/**
 * MyLib v1.0.0 - A sample TypeScript library for demonstrating API upgrades.
 */

/** A user in the system. */
export interface User {
  id: number;
  name: string;
}

/** Connection status. */
export type Status = 'connected' | 'disconnected' | 'error';

/** Configuration for the library. */
export interface Config {
  host: string;
}

/** Database query result item. */
export interface Item {
  key: string;
  value: string;
}

/** Get a user by their ID. */
export function getUser(id: number): User | null {
  if (id > 0) {
    return { id, name: `User${id}` };
  }
  return null;
}

/** Process raw data. */
export function processData(data: number[]): number[] {
  return data.map(b => b + 1);
}

/** Connect to a host. */
export async function connect(host: string): Promise<Status> {
  if (!host) {
    throw new Error('empty host');
  }
  return 'connected';
}

/** Save data with optional sync. */
export async function save(data: string, sync: boolean): Promise<void> {
  if (sync) {
    // Force sync
  }
  console.log('Saved:', data);
}

/** Query with multiple parameters. */
export function query(a: string, b: number, c: boolean): Item[] {
  const results: Item[] = [];
  if (c) {
    results.push({ key: a, value: String(b) });
  }
  return results;
}

/** Find an item by key. */
export function find(key: string): Item {
  return { key, value: 'found' };
}

/** Parse a string. */
export function parse(input: string): string {
  if (!input) {
    throw new Error('empty input');
  }
  return input.toUpperCase();
}

/** @deprecated Use fetchUser instead */
export function deprecatedFn(): void {
  console.log('This function is deprecated');
}

/** Utility functions. */
export namespace utils {
  /** A helper function that returns a greeting. */
  export function helper(): string {
    return 'Hello from utils';
  }

  /** Format a user for display. */
  export function formatUser(user: User): string {
    return `User(id=${user.id}, name=${user.name})`;
  }
}
