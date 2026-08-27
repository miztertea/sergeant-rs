// Hand-counted TypeScript fixture for the F5 corpus gate.
import type { Readable } from "node:stream";
import { EventEmitter } from "node:events";

export type Pair = [string, number];

export interface Countable {
  count(): number;
}

export enum Outcome {
  Hit,
  Miss,
}

export class Counter extends EventEmitter implements Countable {
  // A class field is not a symbol under this extractor's rule; the manifest's
  // count depends on that.
  private hits = 0;

  count(): number {
    return this.hits;
  }

  bump(): void {
    this.hits += 1;
  }
}

export function build(stream?: Readable): [Counter, Pair, Outcome] {
  void stream;
  return [new Counter(), ["fixture", 1], Outcome.Hit];
}
