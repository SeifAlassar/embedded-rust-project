use embedded_recruitment_task::message::{client_message, server_message, AddRequest, EchoMessage};
use embedded_recruitment_task::server::Server;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

mod client;

struct ServerHandle {
    server: Arc<Server>,
    handle: JoinHandle<()>,
}

impl ServerHandle {
    fn new(server: Arc<Server>, handle: JoinHandle<()>) -> Self {
        ServerHandle { server, handle }
    }

    fn stop(self) {
        self.server.stop();
        self.handle.join().expect("Server thread panicked");
    }
}

fn setup_server_thread(server: Arc<Server>) -> ServerHandle {
    let server_clone = Arc::clone(&server); // Clone the Arc
    let handle = thread::spawn(move || {
        server_clone.run().expect("Server encountered an error");
    });
    ServerHandle::new(server, handle) // Use the original server here
}

fn create_server(port: u16) -> Arc<Server> {
    Arc::new(Server::new(&format!("localhost:{}", port)).expect("Failed to start server"))
}

#[test]
fn test_client_connection() {
    let server = create_server(8081); // Unique port for this test
    let server_handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8081, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    server_handle.stop();
}

#[test]
fn test_client_echo_message() {
    let server = create_server(8082); // Unique port for this test
    let server_handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8082, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for EchoMessage"
    );

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    server_handle.stop();
}

#[test]
fn test_multiple_echo_messages() {
    let server = create_server(8083); // Unique port for this test
    let server_handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8083, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        assert!(client.send(message).is_ok(), "Failed to send message");

        let response = client.receive();
        assert!(
            response.is_ok(),
            "Failed to receive response for EchoMessage"
        );

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    server_handle.stop();
}

#[test]
fn test_multiple_clients() {
    let server = create_server(8084); // Unique port for this test
    let server_handle = setup_server_thread(server.clone());

    let mut clients: Vec<client::Client> = vec![
        client::Client::new("localhost", 8084, 1000),
        client::Client::new("localhost", 8084, 1000),
        client::Client::new("localhost", 8084, 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send message"
            );

            let response = client.receive();
            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    server_handle.stop();
}

#[test]
fn test_client_add_request() {
    let server = create_server(8085); // Unique port for this test
    let server_handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8085, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for AddRequest"
    );

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    server_handle.stop();
}
