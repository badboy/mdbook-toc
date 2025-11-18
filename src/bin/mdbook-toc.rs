use clap::{Parser, Subcommand};
use mdbook_preprocessor::Preprocessor;
use mdbook_preprocessor::errors::Error;
use mdbook_toc::Toc;

use std::io;
use std::process;

#[derive(Parser)]
#[command(about, version)]
struct App {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check whether a renderer is supported by this preprocessor
    Supports { renderer: String },
}

fn main() {
    let app = App::parse();

    if let Some(Cmd::Supports { renderer }) = app.cmd {
        handle_supports(&renderer);
    } else if let Err(e) = handle_preprocessing() {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn handle_preprocessing() -> Result<(), Error> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
        eprintln!(
            "Warning: The mdbook-toc preprocessor was built against version \
             {} of mdbook, but we're being called from version {}",
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = Toc.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(renderer: &str) -> ! {
    let supported = Toc.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if let Ok(true) = supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
