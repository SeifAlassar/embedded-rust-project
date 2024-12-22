# Solution Overview

## Architecture

### Server
- **Purpose**: Manages client connections and processes incoming requests.
- **Features**:
  - Listens on a specified port for incoming client connections.
  - Handles communication with each client in a separate thread using a thread pool.
  - Utilizes an atomic flag (`Arc<AtomicBool>`) to manage the server's lifecycle (start and stop).
  - Encodes and decodes messages using Protobuf for efficient communication.

### Client
- **Purpose**: Provides an interface for connecting to the server, sending requests, and receiving responses.
- **Features**:
  - Connects to the server using `TcpStream`.
  - Sends requests (e.g., echo or add) and receives responses using Protobuf encoding.
  - Manages timeouts and handles connection errors gracefully.

### Protobuf Messages
- Defines structured messages for client-server communication:
  - `EchoMessage`: Contains a `content` field for sending and receiving echo responses.
  - `AddRequest`: Contains two numeric fields (`a` and `b`) for addition operations.
  - `ServerMessage`: Encapsulates server responses for different types of requests.

---

## Tests

### Goals
Validate individual components and the end-to-end functionality of the client-server system.

### Test Cases
1. **test_client_connection**
   - Verifies that the client can successfully connect to and disconnect from the server.

2. **test_client_echo_message**
   - Ensures that the client can send an echo message and receive the same response from the server.

3. **test_multiple_echo_messages**
   - Tests the ability to handle multiple echo messages in sequence.

4. **test_multiple_clients**
   - Simulates multiple clients connecting and interacting with the server concurrently.

5. **test_client_add_request**
   - Verifies that the server correctly processes addition requests from the client.

---

## Implementation Details

### Server
1. **TCP Listener**:
   - Accepts incoming client connections using `TcpListener`.
2. **Thread Pool**:
   - Manages multiple client interactions concurrently using `threadpool::ThreadPool`.
3. **Message Decoding**:
   - Processes messages from clients and responds based on the message type.
4. **Lifecycle Management**:
   - Uses an atomic flag to gracefully start and stop the server.

### Client
1. **Connection Management**:
   - Establishes a connection to the server using `TcpStream` with a configurable timeout.
2. **Message Handling**:
   - Encodes requests using Protobuf and sends them to the server.
   - Decodes responses from the server using Protobuf.
3. **Error Handling**:
   - Detects connection errors and invalid messages.

---

## Error Handling
- **Connection Errors**: Managed using `io::Result` to ensure robust handling of connection issues.
- **Invalid Messages**: Detected and logged during Protobuf decoding.
- **Port Binding Issues**: Each test runs on a unique port to avoid conflicts.

---

## Changes Made
- Introduced unique ports for each test case to prevent address binding issues during concurrent tests.
- Implemented `ServerHandle` for clean server startup and shutdown across tests.
- Verified that `solution.md` does not interfere with the test functionality.

---

## Testing Results
- **Successful Tests**: Validates that individual and concurrent operations work as expected.
- **Resolution**: Ensures all tests pass without interference from external files.

