/// used to send different types of data to vsmtp's processes.
#[derive(Debug)]
pub struct ProcessMessage {
    /// id of the mail context
    pub message_id: String,
}

#[cfg(test)]
mod test {
    use crate::ProcessMessage;

    #[test]
    fn debug() {
        println!(
            "{:?}",
            ProcessMessage {
                message_id: "foo".to_string()
            }
        );
    }
}
