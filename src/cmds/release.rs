use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Show the full raw JSON of the release.
    #[arg(short = 'r')]
    raw: bool,

    /// Name of the user to find the release as.
    #[clap(env = "BS_USER")]
    user: String,

    // ID of the release to look for.
    #[arg(value_name = "RELEASE ID")]
    id: String,
}

pub async fn command(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
