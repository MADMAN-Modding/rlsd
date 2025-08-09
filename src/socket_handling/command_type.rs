/// Commands to be sent over the server
#[derive(PartialEq)]
pub enum Commands {
    /// Add data to the current stats
    INPUT,
    /// Rename device
    RENAME,
    /// Rename a device remotely
    AdminRename,
    /// Get device_id for new client
    SETUP,
    /// Remove a device
    REMOVE,
    /// List devices on the server
    LIST,
    /// Stop the server
    EXIT,
    /// Error, command probably wasn't found
    ERROR,
}

impl Commands {
    pub fn to_string<'a>(self) -> &'a str {
        match self {
            Commands::INPUT         => "INPUT!",
            Commands::RENAME        => "RENAME!",
            Commands::AdminRename   => "AdminRename!",
            Commands::SETUP         => "SETUP!",
            Commands::REMOVE        => "REMOVE!",
            Commands::LIST          => "LIST!",
            Commands::EXIT          => "EXIT!",
            Commands::ERROR         => "ERROR!",
        }
    }
}

pub trait CommandTraits {
    fn to_command(&self) -> Commands;
}

impl CommandTraits for String {
    fn to_command(&self) -> Commands {
        match self.replace("!", "").as_str() {
            "INPUT"         => Commands::INPUT,
            "RENAME"        => Commands::RENAME,
            "AdminRename"   => Commands::AdminRename,
            "SETUP"         => Commands::SETUP,
            "REMOVE"        => Commands::REMOVE,
            "LIST"          => Commands::LIST,
            "EXIT"          => Commands::EXIT,
            _               => Commands::ERROR,
        }
    }
}

impl CommandTraits for str {
    fn to_command(&self) -> Commands {
        match self.replace("!", "").as_str() {
            "INPUT"         => Commands::INPUT,
            "RENAME"        => Commands::RENAME,
            "AdminRename"   => Commands::AdminRename,
            "SETUP"         => Commands::SETUP,
            "REMOVE"        => Commands::REMOVE,
            "LIST"          => Commands::LIST,
            "EXIT"          => Commands::EXIT,
            _               => Commands::ERROR,
        }
    }
}
