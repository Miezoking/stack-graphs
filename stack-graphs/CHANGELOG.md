# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.12.0 -- 2023-07-27

### Added

- New `SQLiteReader::clear` and `SQLiteReader::clear_paths` methods that make it easier to reuse instances.
- The method `SQLiteReader::load_graph_for_file` now returns the file handle for the loaded file.

### Changed

- The `Appendable` trait has been simplified. Its `Ctx` type parameter is gone, in favor of a separate trait `ToAppendable` that is used to find appendables for a handle. The type itself moved from the `cycles` to the `stitching` module.
- The `ForwardPartialPathStitcher` has been generalized so that it can be used to build paths from a database, graph edges, or a SQL reader. It now takes a type parameter indicating the type of candidates it uses. Instead `StackGraph`, `PartialPaths` and `Database` instances, it expects a value that implements the `ForwardCandidates` trait. The `ForwardPartialPathStitcher::process_next_phase` expects an additional `extend_until` closure that controls whether the extended paths are considered for further extension or not (using `|_,_,_| true` retains old behavior).
- The SQLite database implementation is using a new schema which stores binary instead of JSON values, resulting in faster write times and smaller databases.
- Renamed method `SQLiteReader::load_graph_for_file_or_directory` to `SQLiteReader::load_graphs_for_file_or_directory`.

### Fixed

- A panic in `AppendingCycleDetector::is_cyclic` that occurred because variables were not always renamed before attempting to concatenate partial paths.
- An inverted condition in `PartialSymbolStack::has_variable` that resulted in incorrect return values.
- A bug in `Partial*Stack::unify` that resulted in recursive bindings (`$1 => SYMBOL,$1`). Any unification that would result in a recursive binding now returns an error.
- A bug in `PartialPath::append` that would incorrectly allow appending edges that added symbols to the precondition symbol stack, even if that stack had no variable.

### Removed

- The `ForwardPartialPathStitcher::from_nodes` function has been removed. Callers are responsible for creating the right initial paths, which can be done using `PartialPath::from_node`.
- The `PartialPaths::find_minimal_partial_path_set_in_file` method has been removed in favor of `ForwardPartialPathStitcher::find_minimal_partial_path_set_in_file`.
- The `PartialPaths::find_all_complete_paths` method has been removed in favor of `ForwardPartialPathStitcher::find_all_complete_partial_paths` using a `GraphEdgeCandidates` instance for the `candidates` argument.
- The `Database::find_all_complete_paths` method has been removed in favor of `ForwardPartialPathStitcher::find_all_complete_partial_paths` using a `DatabaseCandidates` instance for the `candidates` argument.
- The `SQLiteReader::find_all_complete_partial_paths` method has been removed in favor of `ForwardPartialPathStitcher::find_all_complete_partial_paths` using a `SQLiteReader` instance for the `candidates` argument.
- The `OwnedOrDatabasePath` has been removed because it was obsolete with the changes to `Appendable`.

## v0.11.0 -- 2023-06-08

### Added

- New `Defines` and `Refers` assertions test for the presence of definitions and references, respectively, without resolution.
- A method `StackGraph::get_file` to look up an existing file.
- A field named `fully_qualified_name` was added to `SourceInfo`.
- A new `serde` module (requiring the `serde` feature) adds serialization and deserialization of stack graphs and partial paths.
- Stack graphs can now record debug information for edges as it did for noeds. This is also displayed in the HTML visualization when hovering over an edge arrow.
- A new `storage` module (requiring the `storage` feature) implements a simple SQLite database for storing stack graphs and partial paths.

### Fixed

- A bug in `PartialPath::concatenate` that prevented stitching partial paths that were joined at a pop or push node.

### Changed

- The `IncorrectDefinitions` error is renamed to `IncorrectlyDefined`, and `IncorrectDefinitions` is the error used for the `Defines` assertion.
- The `PartialPaths::find_all_partial_paths_in_file` method has been replaced by `PartialPaths::find_minimal_partial_path_set_in_file`, which computes a smaller set. The `ForwardPartialPathStitcher::find_locally_maximal_partial_path_set` function can be used to compute the set previously returned by `find_all_partial_paths_in_file` from the minimal partial path set.
- The cycle detection algorithm, which had the dual repsonisbility of detecting path cycles and culling duplicate paths, has been replaced by a new approach consisting of a new cycle detection algorithm and an optional duplicate path detection algorithm. This fixes issues with the old approach which was based on a heuristic and would sometimes keep too many paths, and sometimes throw away too many. The new algorithms are precise instead of using a heuristic and resolve unpredictable resolution behavior.
- An empty scope stack postcondition is not required anymore for a path to be considered complete. (Symbol stack postconditions must still be empty in complete paths!)
- The visualization code requires the `visualization` feature instead of the `json` feature, and implies the `serde` feature.
- A new `CancelAfterDuration` cancellation flag implementation has been added to easily set timeout-based cancellation.

### Removed

- The method `StackGraph::get_file_unchecked` is removed. Use the new `StackGraph::get_file` instead.
- All the functionality related to `Path` has been removed in favor of using `PartialPath`. In general, the `Path` behavior of can be achieved by using a `PartialPath` with finite preconditions (i.e., no symbol or scope stack variables), which can be created using `PartialPath::eliminate_precondition_stack_variables`.

## v0.10.2 -- 2023-01-10

### Changed

- The up and down text arrows in the labels of push and pop nodes in the visualization have been replaced by bigger arrows that are part of the node shape. This makes it easier to quickly identify push and pop nodes in the graph.

### Fixed

- The `bitvec` dependency was updated to fix installation problems.

## 0.10.1 -- 2022-09-07

### Changed

- The amount of work done in `PartialPaths::find_all_partial_paths_in_file`
  is reduced. The documentation now makes it clear that the
  function itself is responsible for returning a set that is big
  enough to cover all complete paths. Callers should not have to
  filter this set further anymore, and any visitors filtering with
  `PartialPath::is_complete_as_possible` or `PartialPath::is_productive`
  should remove those checks, as they may interfere with future
  ompitmizations of this fynction.

## 0.10.0 -- 2022-08-23

### Added

- Cancellation of potentially long-running operations is now supported
  by passing an implementation of the `CancellationFlag` trait. The
  `NoCancellation` type provides a noop implementation.

### Changed

- `Assertion::run` requires an extra cancellation flag parameter.
- `PartialPaths::find_all_partial_paths_in_file`, `Paths::find_all_paths`,
  `Paths::remove_shadowed_paths`, `PathStitcher::find_all_complete_paths`
  and `ForwardPartialPathStitcher::find_all_complete_partial_paths`
  require an extra cancellation flag parameter and now return a `Result`
  to indicate if the computation was successful or cancelled.
- C API functions `sg_path_arena_find_all_complete_paths` and
  `sg_partial_path_arena_find_partial_paths_in_file` require an extra
  parameter `const size_t *cancellation_flag` parameter and return an
  `sg_result` value to indicate if the operation was successful or
  cancelled. A null pointer can be passed if no cancellation is necessary.

## stack-graphs 0.9.0 - 2022-06-29

### Added

- C API functions `sg_partial_path_database_ensure_both_directions` and
  `sg_partial_path_database_ensure_forwards` to ensure the available
  directions of deques in partial paths in a database.

## stack-graphs 0.8.0 - 2022-05-19

### Added

- Source info field `definiens_span` for the source span that corresponds
  to the definiens of a function declaration (i.e. the body of a function
  declaration, not an assignment).

## stack-graphs 0.7.3 -- 2022-05-17

### Fixed

- We no longer add divergent partial paths to a `Database`.  A divergent partial
  path starts at the root node and has an empty symbol stack precondition.  That
  empty precondition means that it can be concatenated to _any_ path that
  currently ends at the root node — including the result of that concatenation!
  That gives us a divergence, since we can continually prepend the path's
  postcondition to the current symbol stack, forever.  To avoid this divergence,
  we skip these partial paths when constructing a database.

## stack-graphs 0.7.2 -- 2022-05-09

### Added

- Method `StackGraph::add_from_graph` to copy the one graph into another.

## stack-graphs 0.7.1 - 2022-04-19

### Fixes

- Resolves build problems of version 0.7.0

## stack-graphs 0.7.0 - 2022-04-19

### Added

- The module `stack_graphs::assert` defines assertions that can be run against a
  stack graph to test resolution behavior.

- The module `stack_graphs::json` defines JSON rendering of stack graphs and paths.
  This module requires the `json` feature.

- The module `stack_graphs::visualization` defines rendering an interactive HTML
  visualization of stack graphs.  This module requires the `json` feature.

- Stack graph nodes can have associated `DebugInfo`, consisting of key-value pairs
  of strings.

### Changed

- Internal and exported scopes as separate node types are removed and replaced by
  a single scope node type. Whether a scope is exported is indicated by a boolean
  attribute.

  The `Node::{Exported,Internal}Scope` are replaced by a single `Node::Scope`, and
  their implementation types `{Exported,Internal}ScopeNode` are replaced by the type
  `ScopeNode`. The `ScopeNode::is_exported` field indicates whether a scope is
  exported or internal. The `StackGraph::add_{internal,exported}_scope_node` methods
  are replaced by `StackGraph::add_scope_node`.

  In the C API, the enum values `sg_node_kind::SG_NODE_KIND_EXPORTED_SCOPE` and
  `sg_node_kind:SG_NODE_KIND_INTERNAL_SCOPE` are replaced by a single value
  `sg_node_kind::SG_NODE_KIND_SCOPE`.  The field `sg_node::is_clickable` is renamed
  to `sg_node::is_endpoint`.

### Removed

### Deprecated

## stack-graphs 0.6.0 - 2022-03-03

### Added

- The new `ForwardPartialPathStitcher::from_partial_paths` constructor lets you
  seed a path stitching search with a specific list of partial paths (which need
  not be in the `Database`).  This can be used, for instance, to implement “find
  qualified definitions”, where we look for any definition of a fully qualified
  name (expressed as a symbol stack).  There is a new test case that shows an
  example implementation.

### Changed

- The `ForwardPartialPathStitcher::new` constructor has been renamed to
  `from_nodes`, to be more consistent with the new `from_partial_paths`
  constructor.

## stack-graphs 0.5.0 - 2022-02-17

### Added

- Added the ability to bound the amount of work performed in each phase of the
  path stitching algorithm.  Exposed in the C API via the following functions:

  - `sg_forward_path_stitcher_set_max_work_per_phase`
  - `sg_forward_partial_path_stitcher_set_max_work_per_phase`

- Added a `grapheme_offset` field to the `sg_offset` type in the C API, to track
  grapheme positions along with UTF-8 and UTF-16 positions.
