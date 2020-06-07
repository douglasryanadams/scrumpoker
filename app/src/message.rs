use serde::{Deserialize, Serialize};
use tungstenite::{Message};
use uuid::Uuid;

use log::{warn, debug};
use std::fmt;


#[derive(Debug, PartialEq)]
pub(crate) enum MessageAction {
    CreateSession,
    JoinSession,
}

#[derive(Debug, PartialEq)]
pub(crate) struct MessageHeader {
    pub(crate) action: MessageAction,
    session_id: String,
    user_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageHeaderJson {
    action: String,
    session_id: String,
    user_id: String,
}

impl fmt::Display for MessageHeaderJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "action={};session_id={};user_id={};", self.action, self.session_id, self.user_id)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ResponseMessageJson {
    status_code: u16,
    status: String,
    message: String,
}

pub(crate) trait ResponseJsonString {
    fn get_message(&self) -> String;

    fn get_response_json_string(&self) -> String {
        let response_json = ResponseMessageJson {
            status_code: 400,
            status: String::from("bad_request"),
            message: self.get_message(),
        };
        return match serde_json::to_string(&response_json) {
            Ok(msg) => msg,
            Err(error) => {
                warn!("Failed to turn response into JSON String!");
                self.get_message()
            }
        };
    }
}


pub(crate) struct CreateSessionMessage {
    header: MessageHeader,
    session_name: String,
}

pub(crate) struct JoinSessionMessage {
    header: MessageHeader,
    session_id: uuid::Uuid,
    user_name: String,
}


#[derive(Debug)]
pub(crate) struct ParseMessageError {
    message: String
}

impl ResponseJsonString for ParseMessageError {
    fn get_message(&self) -> String {
        return format!("Error parsing message: {}", self.message);
    }
}

impl fmt::Display for ParseMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_message())
    }
}


/// Takes the websocket message body and extracts the header from the message
///
/// Personal Note: This is on of my first implementations of a Rust method.
pub(crate) fn parse_header(message: &Message) -> Result<MessageHeader, ParseMessageError> {
    let message_string = message.to_string();
    let header_json: MessageHeaderJson = match serde_json::from_str(&message_string) {
        Ok(j) => j,
        Err(_) => {
            debug!("Invalid JSON Format: {}", &message_string);
            return Err(ParseMessageError {
                message: String::from("we could not parse string provided into a valid JSON Object. \
                Check for syntax errors.")
            });
        }
    };

    let user_id: Uuid = match Uuid::parse_str(&header_json.user_id) {
        Ok(id_str) => id_str,
        Err(_) => {
            debug!("Invalid UUID: {}", &header_json.user_id);
            return Err(ParseMessageError {
                message: String::from("we could not parse string provided for 'user_id' into a \
                valid UUID.")
            });
        }
    };

    let mut header = MessageHeader {
        action: MessageAction::CreateSession,
        session_id: header_json.session_id,
        user_id,
    };

    header.action = match header_json.action.as_str() {
        "create_session" => MessageAction::CreateSession,
        "join_session" => MessageAction::JoinSession,
        _ => return Err(ParseMessageError {
            message: String::from(format!("Invalid action received: {}", header_json.action))
        })
    };
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_message_json(user_id: Option<String>) -> MessageHeaderJson {
        let message_instance = MessageHeaderJson {
            action: String::from("create_session"),
            session_id: String::from("ECTO-1"),
            user_id: user_id.unwrap_or(String::from("683d711e-fe25-443c-8102-43d4245a6884")),
        };
        return message_instance;
    }


    fn get_message_from_json(message_instance: &MessageHeaderJson) -> Message {
        let message_instance_string = serde_json::to_string(message_instance).unwrap();
        return Message::Text(message_instance_string);
    }

    fn get_expected() -> MessageHeader {
        let expected = MessageHeader {
            action: MessageAction::CreateSession,
            session_id: String::from("ECTO-1"),
            user_id: Uuid::parse_str("683d711e-fe25-443c-8102-43d4245a6884").unwrap(),
        };
        return expected;
    }

    /// Tests the "happy path" with a valid request body
    #[test]
    fn test_parse_header() {
        let message_instance = get_message_json(None);
        let message = get_message_from_json(&message_instance);

        let expected = get_expected();
        let actual = parse_header(&message).unwrap();
        assert_eq!(expected, actual);
    }

    /// Tests that the method still works with extra keys in the JSON body
    #[test]
    fn test_parse_header_extra_keys() {
        let message_instance_string = String::from(r#"
            {
                "action": "create_session",
                "session_id": "ECTO-1",
                "user_id": "683d711e-fe25-443c-8102-43d4245a6884",
                "unrelated_key": "unused_value"
            }"#);
        let message = Message::Text(message_instance_string);

        let expected = get_expected();
        let actual = parse_header(&message).unwrap();
        assert_eq!(expected, actual);
    }


    /// Tests that we get the appropriate Error from a malformed JSON String
    #[test]
    fn test_parse_header_bad_json() {
        let message = Message::Text(String::from("{ bad json"));
        match parse_header(&message) {
            Ok(_) => assert!(false),
            Err(e) => print!("{}", e)
        }
        // Pass if it reaches here
    }

    /// Tests that we get the appropriate Error from a malformed UUID for user_id
    #[test]
    fn test_parse_header_bad_user_id() {
        let message_instance = get_message_json(Some(String::from("baduuid")));
        let message = get_message_from_json(&message_instance);
        match parse_header(&message) {
            Ok(_) => assert!(false),
            Err(e) => print!("{}", e)
        }
        // Pass if it reaches here
    }

    /// Tests that returning
    #[test]
    fn test_get_response_json_string() {
        let error_message = ParseMessageError {
            message: String::from("Placeholder Error Message")
        };
        let expected = ResponseMessageJson {
            status_code: 400,
            status: String::from("bad_request"),
            message: String::from("Error parsing message: Placeholder Error Message"),
        };
        let err_msg_string = error_message.get_response_json_string();
        let actual: ResponseMessageJson = match serde_json::from_str(&err_msg_string) {
            Ok(rmj) => rmj,
            Err(_) => {
                assert!(false);
                return;
            }
        };
        assert_eq!(expected, actual);
    }
}

