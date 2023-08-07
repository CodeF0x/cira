// error messages
pub const ERROR_NOT_FOUND: &str = "Could not find ticket with id";
pub const ERROR_INVALID_ID: &str = "ID must be an integer higher than 0";
pub const ERROR_INVALID_JSON: &str = "Malformed JSON sent";
pub const ERROR_COULD_NOT_CREATE_TICKET: &str = "Could not create ticket";
pub const ERROR_COULD_NOT_GET: &str = "Could not get tickets";
pub const ERROR_COULD_NOT_UPDATE: &str = "Could not update ticket with id";
pub const ERROR_COULD_NOT_DELETE: &str = "Could not delete ticket with id";
pub const CANNOT_LOGOUT: &str = "Could not log you out";
pub const ERROR_NOT_LOGGED_IN: &str = "You're not logged in";
pub const ERROR_COULD_NOT_CREATE_USER: &str = "Could not create user";
pub const ERROR_INCORRECT_PASSWORD: &str = "Incorrect email or password";
pub const ERROR_NO_USER_FOUND: &str = "No user found";

// success messages
pub const SUCCESS_LOGIN: &str = "Successfully logged in";
pub const SUCCESS_LOGOUT: &str = "Successfully logged out";
