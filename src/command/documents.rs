use get_outline::getoutline_connection;
use crate::config::Configuration;
use crate::logic::{
    self,
    documents::{ListError, ListOptions}
};

#[derive(clap::Args)]
pub struct DocumentsArgs {
    #[command(subcommand)]
    subcommand: DocumentsSubcommands,
}

#[derive(clap::Subcommand)]
pub enum DocumentsSubcommands {
    /// List documents that you have access to in GetOutline
    List(ListArgs),
}

/// Runs subcommands of "documents"
pub fn exec_documents(args: &DocumentsArgs, config: &Configuration) {
   match &args.subcommand {
       DocumentsSubcommands::List(list_args) => list(list_args, config),
   }
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// The page of results to display
    #[arg(short, long, default_value_t=0)]
    page: u32,
    /// The number of documents to show per page
    #[arg(short, long, default_value_t=15)]
    results_per_page: u32,
    /// Only show documents you wrote
    #[arg(short='o', long)]
    mine_only: bool,
}

/// Runs the "documents list" subcommand
fn list(args: &ListArgs, config: &Configuration) {
    let client_result = getoutline_connection::get_http_client(&config.get_outline_info.api_key);
    let client = match client_result {
        Ok(clnt) => clnt,
        Err(error) => {
            println!("Failed to connect to GetOutline while establishing the connection!");
            println!("More detail: {}", error);
            return;
        }
    };

    let list_opts = ListOptions {
        page: args.page,
        results_per_page: args.results_per_page,
        own_documents_only: args.mine_only,
    };

    let list_result = logic::documents::list(&client, &client, &list_opts);
    if let Err(error) = list_result {
        match error {
            ListError::BadCredentials => {
                println!("The credentials used to access GetOutline didn't seem to work. Try using a different token!");
            }

            ListError::CouldNotGetAuth(err) => {
                println!("Could not list your documents because we had trouble looking up information about who you are from GetOutline.");
                println!("More detail: {}", err);
            }

            ListError::CouldNotListDocuments(err) => {
                println!("Something went wrong when trying to list your available documents!");
                println!("More detail: {}", err);
            }
        }
    }
}
