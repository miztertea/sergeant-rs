// Hand-counted JavaScript fixture for the F5 corpus gate.
import fs from "node:fs";
import { join, resolve } from "node:path";

export const LIMIT = 8;

// An arrow function bound to a const is deliberately NOT a symbol under this
// extractor's syntax-only rule (A1-09): the declaration is a lexical binding,
// not a function declaration. The manifest's count depends on that.
export const double = (x) => x * 2;

export function topLevel(value) {
  function inner(x) {
    return x + 1;
  }
  return inner(value);
}

export class Counter {
  constructor() {
    this.hits = 0;
  }

  bump() {
    this.hits += 1;
  }
}

export default function main() {
  return [fs, join, resolve, LIMIT, double, topLevel, new Counter()];
}
