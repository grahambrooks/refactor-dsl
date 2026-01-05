/**
 * Client application using mylib v1.0.0
 *
 * This client needs to be upgraded to work with mylib v2.0.0.
 * The upgrade analyzer should detect all the necessary changes.
 */

// Simulating imports from mylib v1
// import { User, Status, Config, Item, getUser, processData, connect, save, query, find, parse, deprecatedFn, utils } from 'mylib';

// Simulated types (from v1)
interface User {
  id: number;
  name: string;
}

type Status = 'connected' | 'disconnected' | 'error';

interface Config {
  host: string;
}

interface Item {
  key: string;
  value: string;
}

// Simulated library functions
function getUser(id: number): User | null {
  return id > 0 ? { id, name: `User${id}` } : null;
}

function processData(data: number[]): number[] {
  return data.map(b => b + 1);
}

async function connect(host: string): Promise<Status> {
  return 'connected';
}

async function save(data: string, sync: boolean): Promise<void> {
  console.log('Saved:', data, sync);
}

function query(a: string, b: number, c: boolean): Item[] {
  return c ? [{ key: a, value: String(b) }] : [];
}

function find(key: string): Item {
  return { key, value: 'found' };
}

function parse(input: string): string {
  return input.toUpperCase();
}

function deprecatedFn(): void {
  console.log('deprecated');
}

const utils = {
  helper(): string {
    return 'helper';
  }
};

// Main application
async function main(): Promise<void> {
  console.log('=== Client Application ===\n');

  // Using getUser (should become fetchUser)
  const user: User | null = getUser(42);
  if (user) {
    console.log('Found user:', user);
  }

  // Using another getUser call
  const admin = getUser(1);
  if (admin) {
    console.log('Admin:', admin);
  }

  // Using processData (should become transformData)
  const data = [1, 2, 3, 4, 5];
  const processed = processData(data);
  console.log('Processed data:', processed);

  // Using connect (needs port and timeout parameters)
  const config: Config = { host: 'localhost' };
  try {
    const status: Status = await connect(config.host);
    console.log('Connected:', status);
  } catch (e) {
    console.error('Connection error:', e);
  }

  // Using save (sync parameter should be removed)
  await save('important data', true);
  await save('more data', false);

  // Using query (parameters should be reordered)
  const results = query('search', 10, true);
  console.log('Query results:', results.length, 'items');

  // Another query call
  const moreResults = query('filter', 5, false);
  console.log('More results:', moreResults.length, 'items');

  // Using find (should return Item | null now)
  const item: Item = find('my_key');
  console.log('Found item:', item.key);

  // Using parse (return type changed)
  try {
    const result: string = parse('hello world');
    console.log('Parsed:', result);
  } catch (e) {
    console.error('Parse error:', e);
  }

  // Using deprecatedFn (should be removed)
  deprecatedFn();

  // Using utils namespace (should become helpers)
  const greeting = utils.helper();
  console.log('Greeting:', greeting);

  console.log('\n=== Done ===');
}

/** A service that uses the library */
class UserService {
  private config: Config;

  constructor(host: string) {
    this.config = { host };
  }

  getUserById(id: number): User | null {
    // Uses getUser internally
    return getUser(id);
  }

  processUserData(data: number[]): number[] {
    // Uses processData internally
    return processData(data);
  }

  async connectToServer(): Promise<Status> {
    // Uses connect internally
    return connect(this.config.host);
  }

  async saveUser(user: User): Promise<void> {
    // Uses save internally
    const data = `${user.id}:${user.name}`;
    await save(data, true);
  }

  searchUsers(queryStr: string, limit: number): Item[] {
    // Uses query internally
    return query(queryStr, limit, true);
  }

  findUserItem(key: string): Item {
    // Uses find internally
    return find(key);
  }

  getHelperMessage(): string {
    // Uses utils.helper internally
    return utils.helper();
  }
}

// Run main
main().catch(console.error);

export { UserService, User, Status, Config };
