use crate::message::EchoMessage; // Import the message format defined by protobuf
use log::{error, info, warn}; // Import logging macros
use prost::Message; // For encoding and decoding protobuf messages
use std::{
    io::{self, ErrorKind, Read, Write}, // For input/output operations
    net::{TcpListener, TcpStream},      // For network operations
    sync::{
        atomic::{AtomicBool, Ordering}, // For atomic operations on shared state
        Arc,                            // For sharing state across threads
    },
    time::Duration, // For adding delays
};
use threadpool::ThreadPool; // For managing a pool of threads

// A struct representing the client connected to the server
struct Client {
    stream: TcpStream, // Network stream for communicating with the client
}

impl Client {
    // Constructor to create a new client instance
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    // Handles communication with the client
    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512]; // Buffer to store incoming data
        let bytes_read = self.stream.read(&mut buffer)?; // Read data from the client

        // If no data is read, the client has disconnected
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Ok(());
        }

        // Attempt to decode the incoming data as a protobuf message
        if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
            info!("Received: {}", message.content); // Log the received message
                                                    // Echo the message back to the client
            let payload = message.encode_to_vec();
            self.stream.write_all(&payload)?; // Send the encoded message
            self.stream.flush()?; // Ensure all data is sent immediately
        } else {
            error!("Failed to decode message"); // Log an error if decoding fails
        }

        Ok(())
    }
}

// The main server struct
pub struct Server {
    listener: TcpListener,       // Listens for incoming client connections
    is_running: Arc<AtomicBool>, // Shared state to manage server's running status
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?; // Bind the server to the specified address
        let is_running = Arc::new(AtomicBool::new(false)); // Initialize the running flag
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Mark the server as running
        info!("Server is running on {}", self.listener.local_addr()?); // Log the server address

        // Enable non-blocking mode to prevent the listener from halting the server
        self.listener.set_nonblocking(true)?;

        let pool = ThreadPool::new(16); // Create a thread pool with 16 threads

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr); // Log new connection
                    let is_running = self.is_running.clone(); // Clone the running flag for the thread

                    // Use the thread pool to handle the client
                    pool.execute(move || {
                        let mut client = Client::new(stream); // Create a new client instance
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                // Handle client communication
                                error!("Error handling client: {}", e); // Log errors
                                break; // Exit the loop on error
                            }
                        }
                        info!("Client handler thread exiting.");
                    });
                }
                // Handle cases where no new connection is available
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100)); // Reduce CPU usage by sleeping briefly
                }
                // Handle unexpected errors while accepting connections
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopped."); // Log server shutdown
        Ok(())
    }

    /// Stops the server by setting the running flag to false
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst); // Mark the server as stopped
            info!("Shutdown signal sent."); // Log the shutdown signal
        } else {
            warn!("Server was already stopped or not running."); // Log a warning if the server isn't running
        }
    }
}
