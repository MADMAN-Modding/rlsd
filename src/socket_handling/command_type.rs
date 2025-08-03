/// Commands to be sent over the server
#[derive(PartialEq)]
pub enum Commands {
    /// Add data to the current stats
    INPUT,
    RENAME,
    SETUP,
    EXIT,
    ERROR,
}

impl Commands {
    pub fn to_string<'a>(self) -> &'a str {
        match self {
            Self::INPUT => "INPUT!",
            Self::RENAME => "RENAME!",
            Self::SETUP => "SETUP!",
            Self::EXIT => "EXIT!",
            Self::ERROR => "ERROR!",
        }
    }
}

pub trait CommandTraits {
    fn to_command(&self) -> Commands;
}

impl CommandTraits for String {
    fn to_command(&self) -> Commands {
        match self.replace("!", "").as_str() {
            "INPUT" => Commands::INPUT,
            "RENAME" => Commands::RENAME,
            "SETUP" => Commands::SETUP,
            "EXIT" => Commands::EXIT,
            _ => Commands::ERROR,
        }
    }
}

impl CommandTraits for str {
    fn to_command(&self) -> Commands {
        match self.replace("!", "").as_str() {
            "INPUT" => Commands::INPUT,
            "RENAME" => Commands::RENAME,
            "SETUP" => Commands::SETUP,
            "EXIT" => Commands::EXIT,
            _ => Commands::ERROR,
        }
    }
}
