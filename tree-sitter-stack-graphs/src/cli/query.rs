// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2023, stack-graphs authors.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueHint;
use itertools::Itertools;
use lsp_positions::PositionedSubstring;
use lsp_positions::SpanCalculator;
use stack_graphs::storage::FileStatus;
use stack_graphs::storage::SQLiteReader;
use stack_graphs::storage::StorageError;
use std::path::Path;
use std::path::PathBuf;

use crate::loader::FileReader;

use super::util::sha1;
use super::util::wait_for_input;
use super::util::ConsoleFileLogger;
use super::util::FileLogger;
use super::util::SourcePosition;

/// Analyze sources
#[derive(Args)]
pub struct QueryArgs {
    /// Wait for user input before starting analysis. Useful for profiling.
    #[clap(long)]
    pub wait_at_start: bool,

    #[clap(subcommand)]
    target: Target,
}

impl QueryArgs {
    pub fn run(self, db_path: &Path) -> anyhow::Result<()> {
        if self.wait_at_start {
            wait_for_input()?;
        }
        let mut db = SQLiteReader::open(&db_path)?;
        self.target.run(&mut db)
    }
}

#[derive(Subcommand)]
pub enum Target {
    Definition(Definition),
}

impl Target {
    pub fn run(self, db: &mut SQLiteReader) -> anyhow::Result<()> {
        match self {
            Self::Definition(cmd) => cmd.run(db),
        }
    }
}

#[derive(Parser)]
pub struct Definition {
    /// Reference source positions, formatted as PATH:LINE:COLUMN.
    #[clap(
        value_name = "SOURCE_POSITION",
        required = true,
        value_hint = ValueHint::AnyPath,
        parse(try_from_str),
    )]
    pub references: Vec<SourcePosition>,
}

impl Definition {
    pub fn run(self, db: &mut SQLiteReader) -> anyhow::Result<()> {
        for mut reference in self.references {
            reference.canonicalize()?;

            let mut file_reader = FileReader::new();

            let log_path = PathBuf::from(reference.to_string());
            let mut logger = ConsoleFileLogger::new(&log_path, true, true);

            logger.processing();

            let source_path = reference.path.to_string_lossy();
            let source = file_reader.get(&reference.path)?;
            let tag = sha1(source);

            match db.status_for_file(&source_path, Some(&tag))? {
                FileStatus::Indexed => {}
                _ => {
                    logger.failure("file not indexed", None);
                    return Ok(());
                }
            }

            let lines = PositionedSubstring::lines_iter(source);
            let mut span_calculator = SpanCalculator::new(source);

            db.load_graph_for_file(&reference.path.to_string_lossy())?;
            let (graph, _, _) = db.get();

            let reference = match reference.to_assertion_source(graph, lines, &mut span_calculator)
            {
                Ok(result) => result,
                Err(_) => {
                    logger.failure("invalid file or position", None);
                    return Ok(());
                }
            };

            if reference.iter_references(graph).next().is_none() {
                logger.failure("no references", None);
                return Ok(());
            }
            let starting_nodes = reference.iter_references(graph).collect::<Vec<_>>();

            let mut actual_paths = Vec::new();
            let mut reference_paths = Vec::new();
            match db.find_all_complete_partial_paths(
                starting_nodes,
                &stack_graphs::NoCancellation,
                |_g, _ps, p| {
                    reference_paths.push(p.clone());
                },
            ) {
                Ok(_) => {}
                Err(StorageError::Cancelled(..)) => {
                    logger.failure("path finding timed out", None);
                    return Ok(());
                }
                err => err?,
            };

            let (graph, partials, _) = db.get();
            for reference_path in &reference_paths {
                if reference_paths
                    .iter()
                    .all(|other| !other.shadows(partials, reference_path))
                {
                    actual_paths.push(reference_path.clone());
                }
            }

            if actual_paths.is_empty() {
                logger.warning("no definitions", None);
                return Ok(());
            }

            logger.success(
                "found definitions:",
                Some(
                    &actual_paths
                        .into_iter()
                        .enumerate()
                        .map(|(idx, path)| {
                            let file = match graph[path.end_node].id().file() {
                                Some(f) => graph[f].to_string(),
                                None => "?".to_string(),
                            };
                            let line_col = match graph.source_info(path.end_node) {
                                Some(p) => format!(
                                    "{}:{}",
                                    p.span.start.line + 1,
                                    p.span.start.column.grapheme_offset + 1
                                ),
                                None => "?:?".to_string(),
                            };
                            format!("  {:2}: {}:{}", idx, file, line_col)
                        })
                        .join("\n"),
                ),
            );
        }
        Ok(())
    }
}
