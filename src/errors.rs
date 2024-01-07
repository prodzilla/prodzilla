pub trait MapToSendError<T, E> {
    fn map_to_send_err(self) -> Result<T, Box<dyn std::error::Error + Send>>;
}

impl<T, E> MapToSendError<T, E> for Result<T, E>
where
    E: std::error::Error + Send + 'static,
{
    fn map_to_send_err(self) -> Result<T, Box<dyn std::error::Error + Send>> {
        self.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)
    }
}
