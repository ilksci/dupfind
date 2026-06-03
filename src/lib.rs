pub mod cli;
pub mod cleaner;
pub mod error;
pub mod hasher;
pub mod reporter;
pub mod scanner;

/// Top-level entry point called from `main.rs`.
pub fn run() -> error::Result<()> {
    let args = cli::parse_args();

    // Phase 1: Scan files
    let file_infos = scanner::scan(&args)?;

    if file_infos.is_empty() {
        println!("No files found matching the criteria.");
        return Ok(());
    }

    println!("Found {} files, scanning for duplicates...", file_infos.len());

    // Phase 2: Hash and find duplicates
    let duplicates = hasher::find_duplicates(file_infos)?;

    if duplicates.is_empty() {
        println!("No duplicate files found.");
        return Ok(());
    }

    let total_dup_count: usize = duplicates.iter().map(|g| g.files.len()).sum();
    println!(
        "Found {} duplicate groups with {} total duplicate files.",
        duplicates.len(),
        total_dup_count,
    );

    // Phase 3: Export report
    if let Some(ref output_path) = args.output {
        reporter::export(&duplicates, output_path)?;
        println!("Report exported to: {}", output_path.display());
    }

    // Phase 4: Clean duplicates
    if let Some(ref strategy_arg) = args.delete {
        let strategy: cleaner::KeepStrategy = strategy_arg.clone().into();
        cleaner::clean(&duplicates, &strategy)?;
    }

    Ok(())
}
