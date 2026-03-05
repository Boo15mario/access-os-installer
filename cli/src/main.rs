mod wizard;

fn main() {
    let mut dry_run = false;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--help" | "-h" => {
                print_help();
                return;
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                print_help();
                std::process::exit(2);
            }
        }
    }

    let mut wizard = wizard::Wizard::new(dry_run);
    if let Err(err) = wizard.run() {
        eprintln!("FATAL: {}", err);
        std::process::exit(1);
    }
}

fn print_help() {
    println!(
        "\
access-os-installer-cli

Usage:
  access-os-installer-cli [--dry-run]

Options:
  --dry-run   Print the computed install plan but do not modify disks or install packages
  -h, --help  Show this help
"
    );
}
