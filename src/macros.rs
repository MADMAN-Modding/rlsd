#[macro_export]
/// Returns the user input from terminal as `String`
/// 
/// # Parameters
/// * (Optional) prompt `str` - The string to be printed before taking input.
macro_rules! input {
    () => {
        input!("NO_INPUT")
    };

    ($prompt:expr) => {
        {
            use std::io;

            if ($prompt != "NO_INPUT") {
                // Prints the prompt to the terminal
                println!("{}", $prompt.blue())
            }

            let mut user_input: String = String::new();

            // Gets user input
            io::stdin()
                .read_line(&mut user_input)
                .expect("failed to read from stdin");

            user_input.trim().to_string()
        }
    }
}
