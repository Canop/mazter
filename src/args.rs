#[derive(Debug, clap::Parser)]
#[clap(
    author,
    version,
    about = "Mazes in your terminal - doc at https://dystroy.org/mazter"
)]
pub struct Args {
    /// don't play, just print a random maze
    #[clap(long, value_parser)]
    pub build: bool,

    /// forget all achievements of the user
    #[clap(long, value_parser)]
    pub reset: bool,

    /// print the Hall of Fame
    #[clap(long, value_parser)]
    pub hof: bool,

    /// level to play or print - default is the first not won
    #[clap(long, value_parser)]
    pub level: Option<usize>,

    /// user playing
    #[clap(short, long, value_parser, default_value_t = whoami::username())]
    pub user: String,

    /// let mazter play alone
    #[clap(long, value_parser)]
    pub screen_saver: bool,
}
