use clap::Parser;

#[derive(Parser)] // requires `derive` feature
#[clap(name = "qc")]
#[clap(bin_name = "qc")]
enum QC {
    FetchClues(qc::commands::fetch_clues::Args),
    ExportClues(qc::commands::export_clues::Args),
    Find(qc::commands::find::Args),
    Print(qc::commands::print::Args),
}

fn main() {
    match QC::parse() {
        QC::FetchClues(args) => qc::commands::fetch_clues::run(args),
        QC::ExportClues(args) => qc::commands::export_clues::run(args),
        QC::Find(args) => qc::commands::find::run(args),
        QC::Print(args) => qc::commands::print::run(args),
    }
}
