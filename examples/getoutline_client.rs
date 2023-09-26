use dotenvy::dotenv;
use get_outline::getoutline_connection::{auth, documents, get_http_client};
use std::env;

fn main() {
    let _ = dotenv();
    let Ok(api_key) = env::var("GETOUTLINE_API_KEY") else {
        println!("No API key provided! Quitting...");
        return;
    };

    let http_client = match get_http_client(&api_key) {
        Ok(client) => client,
        Err(construct_err) => {
            println!("Could not build HTTP client: {}", construct_err);
            return;
        }
    };

    let auth_info = match auth::current(&http_client) {
        Ok(info) => info,
        Err(auth_retrieve_err) => {
            println!("Could not retrieve auth info: {}", auth_retrieve_err);
            println!("More information: {:#?}", auth_retrieve_err);
            return;
        }
    };

    println!(
        "Currently authenticated as {} with user ID {}!",
        auth_info.user.name, auth_info.user.id
    );

    let documents_request = documents::ListRequest::default();
    let some_documents = match documents::list(&http_client, &documents_request) {
        Ok(doc_list) => doc_list,
        Err(error) => {
            println!("Could not retrieve list of documents: {}", error);
            println!("More info: {:#?}", error);
            return;
        }
    };

    println!("Retrieved documents:");
    for document in some_documents.iter() {
        println!(
            "\t - {title} ({id})",
            title = document.title,
            id = document.id
        );
    }
}
