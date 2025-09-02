use {
    anyhow::{Context, Result},
    clap::{Parser, Subcommand},
    penumbra_disclosure::{api, client::DisclosureClient},
};

#[derive(Parser)]
struct Cli {
    #[arg(
        long,
        help = "penumbra grpc node",
        default_value = "http://localhost:8080"
    )]
    grpc_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "run the disclosure api service")]
    Api {
        #[arg(
            long,
            help = "url to expose the api on",
            default_value = "localhost:1337"
        )]
        listen_url: String,
    },
    #[command(about = "generate a disclosure bundle for a transaction")]
    DiscloseTransaction {
        #[arg(
            long,
            help = "full vieweing key that can decrypt at least part of the transaction"
        )]
        full_viewing_key: String,
        #[arg(long, help = "the transaction hash to generate the bundle for")]
        transaction_hash: String,
    },
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Api { listen_url } => api::server::start_api(cli.grpc_url, listen_url).await,
        Commands::DiscloseTransaction {
            full_viewing_key,
            transaction_hash,
        } => {
            let dc = DisclosureClient::new(&cli.grpc_url, &full_viewing_key.parse()?).await?;

            let dc = dc.lock().await;

            dc.sync()
                .await
                .with_context(|| "failed to sync disclosure client")?;

            let bundle = dc
                .transaction(&transaction_hash)
                .await
                .with_context(|| "failed to generate disclosure bundle")?;

            println!("{bundle:#?}");
            Ok(())
        }
    }
}
