use clap::Parser;

#[derive(Parser, Debug)]
pub struct CreateTokenArgs {
    #[clap(short, long)]
    pub token_name: String,
    #[clap(short, long)]
    pub token_symbol: String,
    #[clap(short, long)]
    pub token_decimals: u8,
}

pub async fn handle_create_token(args: &CreateTokenArgs) -> anyhow::Result<()> {
    println!("create token: {:?}", args);
    Ok(())
}
