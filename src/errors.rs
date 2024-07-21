use std::error::Error;

use crate::probe::model::{ExpectField, ExpectOperation};

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

pub struct ExpectationFailedError {
    pub field: ExpectField,
    pub expected: String,
    pub body: String,
    pub operation: ExpectOperation,
    pub status_code: u32,
}

impl Error for ExpectationFailedError {}

impl std::fmt::Display for ExpectationFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Failed to meet expectation for field '{:?}' with operation {:?} {:?}.",
            self.field, self.operation, self.expected,
        )
    }
}

impl std::fmt::Debug for ExpectationFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Failed to meet expectation for field '{:?}' with operation {:?} {:?}. Received: status '{}', body '{}'",
            self.field, self.operation, self.expected, self.status_code, self.body
        )
    }
}
