use std::path::PathBuf;

use eyre::Result;
use structopt::StructOpt;

use atuin_client::database::Sqlite;
use atuin_client::settings::Settings as ClientSettings;
use atuin_common::utils::uuid_v4;
use atuin_server::settings::Settings as ServerSettings;

mod event;
mod history;
mod import;
mod init;
mod login;
mod register;
mod search;
mod server;
mod stats;
mod sync;

#[derive(StructOpt)]
pub enum AtuinCmd {
    #[structopt(
        about="manipulate shell history",
        aliases=&["h", "hi", "his", "hist", "histo", "histor"],
    )]
    History(history::Cmd),

    #[structopt(about = "import shell history from file")]
    Import(import::Cmd),

    #[structopt(about = "start an atuin server")]
    Server(server::Cmd),

    #[structopt(about = "calculate statistics for your history")]
    Stats(stats::Cmd),

    #[structopt(about = "output shell setup")]
    Init(init::Cmd),

    #[structopt(about = "generates a UUID")]
    Uuid,

    #[structopt(about = "interactive history search")]
    Search {
        #[structopt(long, short, about = "filter search result by directory")]
        cwd: Option<String>,

        #[structopt(long = "exclude-cwd", about = "exclude directory from results")]
        exclude_cwd: Option<String>,

        #[structopt(long, short, about = "filter search result by exit code")]
        exit: Option<i64>,

        #[structopt(long = "exclude-exit", about = "exclude results with this exit code")]
        exclude_exit: Option<i64>,

        #[structopt(long, short, about = "only include results added before this date")]
        before: Option<String>,

        #[structopt(long, about = "only include results after this date")]
        after: Option<String>,

        #[structopt(long, short, about = "open interactive search UI")]
        interactive: bool,

        #[structopt(long, short, about = "use human-readable formatting for time")]
        human: bool,

        query: Vec<String>,
    },

    #[structopt(about = "sync with the configured server")]
    Sync {
        #[structopt(long, short, about = "force re-download everything")]
        force: bool,
    },

    #[structopt(about = "login to the configured server")]
    Login(login::Cmd),

    #[structopt(about = "register with the configured server")]
    Register(register::Cmd),

    #[structopt(about = "print the encryption key for transfer to another machine")]
    Key,
}

impl AtuinCmd {
    pub async fn run(self) -> Result<()> {
        let client_settings = ClientSettings::new()?;
        let server_settings = ServerSettings::new()?;

        let db_path = PathBuf::from(client_settings.db_path.as_str());

        let mut db = Sqlite::new(db_path).await?;

        match self {
            Self::History(history) => history.run(&client_settings, &mut db).await,
            Self::Import(import) => import.run(&mut db).await,
            Self::Server(server) => server.run(&server_settings).await,
            Self::Stats(stats) => stats.run(&mut db, &client_settings).await,
            Self::Init(init) => init.run(),
            Self::Search {
                cwd,
                exit,
                interactive,
                human,
                exclude_exit,
                exclude_cwd,
                before,
                after,
                query,
            } => {
                search::run(
                    cwd,
                    exit,
                    interactive,
                    human,
                    exclude_exit,
                    exclude_cwd,
                    before,
                    after,
                    &query,
                    &mut db,
                )
                .await
            }

            Self::Sync { force } => sync::run(&client_settings, force, &mut db).await,
            Self::Login(l) => l.run(&client_settings),
            Self::Register(r) => register::run(
                &client_settings,
                r.username.as_str(),
                r.email.as_str(),
                r.password.as_str(),
            ),
            Self::Key => {
                let key = std::fs::read(client_settings.key_path.as_str())?;
                println!("{}", base64::encode(key));
                Ok(())
            }

            Self::Uuid => {
                println!("{}", uuid_v4());
                Ok(())
            }
        }
    }
}
