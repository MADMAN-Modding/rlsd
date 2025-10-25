/// Commands to be sent over the server
#[derive(PartialEq)]
pub enum Commands {
    /// Add data to the current stats
    INPUT,
    /// Rename device
    RENAME,
    /// Rename a device remotely (admin)
    ADMINRENAME,
    /// Get device_id for new client
    SETUP,
    /// Remove a device
    REMOVE,
    /// List devices on the server (admin)
    LIST,
    /// Updates the server (admin)
    UpdateServer,
    /// Downloads the database from the server (admin)
    DownloadDatabase,
    /// Stop the server
    EXIT,
    /// Error, command probably wasn't found
    ERROR,
}

impl Commands {
    pub fn to_string<'a>(self) -> &'a str {
        match self {
            Commands::INPUT             => "INPUT!",
            Commands::RENAME            => "RENAME!",
            Commands::ADMINRENAME       => "ADMINRENAME!",
            Commands::SETUP             => "SETUP!",
            Commands::REMOVE            => "REMOVE!",
            Commands::LIST              => "LIST!",
            Commands::UpdateServer      => "UPDATE_SERVER!",
            Commands::DownloadDatabase   => "DOWNLOAD_DATABASE!",
            Commands::EXIT              => "EXIT!",
            Commands::ERROR             => "ERROR!",
        }
    }
}

pub trait CommandTraits {
    fn to_command(&self) -> Commands;
}

impl CommandTraits for String {
    fn to_command(&self) -> Commands {
        match self.replace("!", "").as_str() {
            "INPUT"             => Commands::INPUT,
            "RENAME"            => Commands::RENAME,
            "ADMINRENAME"       => Commands::ADMINRENAME,
            "SETUP"             => Commands::SETUP,
            "REMOVE"            => Commands::REMOVE,
            "LIST"              => Commands::LIST,
            "UPDATE_SERVER"      => Commands::UpdateServer,
            "DOWNLOAD_DATABASE"   => Commands::DownloadDatabase,
            "EXIT"              => Commands::EXIT,
            _                   => Commands::ERROR,
        }
    }
}

impl CommandTraits for str {
    fn to_command(&self) -> Commands {
        match self.replace("!", "").as_str() {
            "INPUT"             => Commands::INPUT,
            "RENAME"            => Commands::RENAME,
            "ADMINRENAME"       => Commands::ADMINRENAME,
            "SETUP"             => Commands::SETUP,
            "REMOVE"            => Commands::REMOVE,
            "LIST"              => Commands::LIST,
            "UPDATE_SERVER"      => Commands::UpdateServer,
            "DOWNLOAD_DATABASE"   => Commands::DownloadDatabase,
            "EXIT"              => Commands::EXIT,
            _                   => Commands::ERROR,
        }
    }
}
