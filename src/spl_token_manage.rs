use clap::Parser;

pub mod create_token;

#[derive(Parser, Debug)]
pub enum SplTokenMange {
    /// create spl token
    CreateToken(create_token::CreateTokenArgs),
}

pub async fn handle_spl_token_manage(spl_token_manage: &SplTokenMange) -> anyhow::Result<()> {
    match spl_token_manage {
        SplTokenMange::CreateToken(create_token_args) => {
            create_token::handle_create_token(create_token_args).await
        }
    }
}
