// The legacy CommonJS-interop import form, hand-counted for the corpus gate.
//
// `import x = require("...")` is parsed by tree-sitter-typescript as an
// `import_statement` with no `source` field of its own — the module string
// sits inside an `import_require_clause`. It is unambiguously an import, so it
// is unambiguously an edge.
import fsp = require("node:fs/promises");
import { EventEmitter } from "node:events";

export function reader(): EventEmitter {
  void fsp;
  return new EventEmitter();
}
