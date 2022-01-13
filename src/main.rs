use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use tokio::sync::oneshot::channel;
use crate::frontmatter::read_frontmatter;
use crate::image::create_images_from_template;
use crate::static_server::start_server;

mod static_server;
mod image;
mod frontmatter;

/// A program to create twitter cards/open graph images
/// for your website.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build the cards/images
    Build {
        /// Port on which to serve the template
        #[clap(short, long)]
        port: Option<u16>,

        /// Folder where the image template file is saved
        #[clap(short, long)]
        template_folder: PathBuf,

        /// Folder where the blog's md files are saved
        #[clap(short, long)]
        blog_folder: PathBuf,

        /// Full file path of chrome browser
        #[clap(short, long)]
        chrome_path: PathBuf,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Build { port, template_folder, blog_folder, chrome_path} => {
            build(&template_folder, port, blog_folder, chrome_path);
        }
    }
}

#[tokio::main]
pub(crate) async fn build(root_path: &Path, port: Option<u16>, blog_folder: PathBuf, chrome_path: PathBuf) {
    let (server_ready_tx, server_ready_rx) = channel();
    let (images_created_tx, images_created_rx) = channel();
    let port = port.unwrap_or(3000);
    tokio::spawn(async move {
        let frontmatter_list = read_frontmatter(&blog_folder);
        server_ready_rx.await.expect("Failed to receive server ready signal.");
        create_images_from_template(frontmatter_list, chrome_path.as_path(), port);
        images_created_tx.send(()).expect("Failed to send images created signal.");
    });
    start_server(root_path, port, server_ready_tx, images_created_rx).await;
}