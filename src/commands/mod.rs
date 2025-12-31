use clap::Subcommand;

pub mod edit;
pub mod init;
pub mod list;

#[derive(Debug, Subcommand)]
pub enum Commands {
	/// List installed templates
	List,

	/// Open the templates directory in your preferred editor
	Edit {
		/// Editor to use
		#[clap(short, long)]
		editor: Option<String>,
	},

	/// Create a new project from a template
	Init(init::InitArgs),
}
