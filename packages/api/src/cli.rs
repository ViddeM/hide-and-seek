use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Hide and Seek game server", version, long_about = None)]
pub struct Cli {
    /// PostgreSQL connection URL
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    /// Address and port to bind the HTTP server
    #[arg(long, env = "HOST", default_value = "0.0.0.0:8080")]
    pub host: String,
}
