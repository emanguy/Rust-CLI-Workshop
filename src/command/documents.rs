use crate::config::Configuration;
use crate::logic::{
    self,
    documents::{ListError, ListOptions, RetrieveError},
};
use crate::{file_saver, getoutline_connection};
use clap::Args;

#[derive(clap::Args)]
pub struct DocumentsArgs {
    #[command(subcommand)]
    subcommand: DocumentsSubcommands,
}

#[derive(clap::Subcommand)]
pub enum DocumentsSubcommands {
    /// List documents that you have access to in GetOutline
    List(ListArgs),
    /// Fetch a document from GetOutline and save it to disk
    Save(SaveArgs),
}

/// Runs subcommands of "documents"
pub fn exec_documents(args: &DocumentsArgs, config: &Configuration) {
    match &args.subcommand {
        DocumentsSubcommands::List(list_args) => list(list_args, config),
        DocumentsSubcommands::Save(save_args) => save(save_args, config),
    }
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// The page of results to display
    #[arg(short, long, default_value_t = 0)]
    page: u32,
    /// The number of documents to show per page
    #[arg(short, long, default_value_t = 15)]
    results_per_page: u32,
    /// Only show documents you wrote
    #[arg(short = 'o', long)]
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

#[derive(Args)]
pub struct SaveArgs {
    /// The ID of the document in GetOutline to download to a file
    doc_id: String,
    /// The name of the new file to be created (the .md file extension will be appended automatically if you don't add it)
    #[arg(short, long)]
    file_name: Option<String>,
}

/// Runs the "documents save" subcommand
fn save(args: &SaveArgs, config: &Configuration) {
    let doc_saver = file_saver::FileDocumentSaver;
    let client_result = getoutline_connection::get_http_client(&config.get_outline_info.api_key);

    let client = match client_result {
        Ok(client) => client,
        Err(error) => {
            println!("Could not connect to the GetOutline API!");
            println!("More detail: {}", error);
            return;
        }
    };

    let retrieve_options = logic::documents::RetrieveOptions {
        suggested_name: args.file_name.as_deref(),
    };
    let doc_result =
        logic::documents::retrieve(&client, &doc_saver, &args.doc_id, &retrieve_options);
    match doc_result {
        Ok(_) => println!("Document saved successfully!"),
        Err(RetrieveError::BadAuth) => println!("The API token provided was rejected by GetOutline, try generating another one and try again!"),
        Err(RetrieveError::DocumentDoesNotExist { requested_id}) => println!("GetOutline could not find a document with the ID \"{}\".", requested_id),
        Err(RetrieveError::DocumentRetrieveFailed(error)) => {
            println!("Could not fetch the requested document from GetOutline for an unknown reason!!");
            println!("More information: {}", error);
        },
        Err(RetrieveError::DocumentSaveFailed(error)) => {
            println!("Could not save the document to disk!");
            println!("More information: {}", error);
        }
        Err(RetrieveError::SameNameCouldNotSave { name }) =>
            println!("A file with the same name already exists (\"{}\"), please suggest a different name using the appropriate flag.", name),
    }
}
