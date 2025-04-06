# Simple Chat Application

A real-time chat application implemented in both Rust and Go, featuring WebSocket connections for instant messaging.

## Features

- User registration and authentication
- Real-time messaging using WebSockets
- Message history
- User presence tracking
- Cross-Origin Resource Sharing (CORS) support
- REST API endpoints for user management

## Rust Implementation

### Prerequisites

- Rust (latest stable version)
- Cargo package manager

### Setup and Running

1. Navigate to the Rust project directory:
```bash
cd rustchatapp
```

2. Install dependencies and build:
```bash
cargo build
```

3. Run the server:
```bash
RUST_LOG=info cargo run
```

The server will start at `http://127.0.0.1:8082`

### API Endpoints

- `GET /` - Home endpoint
- `POST /register` - Register new user
- `POST /login` - User login
- `GET /users` - List all users
- `GET /ws` - WebSocket connection endpoint
- `GET /messages` - Get chat history

### UI 
- cd ui
- open index.html in your browser
- Register a user
- login as the registered user

### Example Usage

```bash
# Register a new user
curl -X POST http://127.0.0.1:8082/register \
  -H "Content-Type: application/json" \
  -d '{"first_name":"John","last_name":"Doe"}'

# Login
curl -X POST http://127.0.0.1:8082/login \
  -H "Content-Type: application/json" \
  -d '{"first_name":"John","last_name":"Doe"}'
```

### WebSocket Connection

Connect to WebSocket endpoint:
```
ws://127.0.0.1:8082/ws?userId=<user_id>
```

Message format:
```json
{
    "sender_id": 1,
    "receiver_id": 2,
    "content": "Hello!"
}
```

## Project Structure

```
rustchatapp/
├── src/
│   ├── main.rs        # Main application file
│   ├── messages.rs    # Message handling
│   └── users.rs       # User management
└── Cargo.toml         # Dependencies and project config
└── ui/
    ├── index.html   # Web client interface
    ├── index.css    # Styles for web client
    └── index.js     # Client-side JavaScript
```

## Dependencies

### Rust Version
```toml
[dependencies]
actix-web = "4.10.2"
actix-web-actors = "4.3.1"
actix = "0.13"
actix-cors = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
env_logger = "0.10"
log = "0.4"
url = "2.2"
```

## Go Implementation

### Prerequisites

- Go 1.16 or higher
- Git (for package management)

### Setup and Running

1. Navigate to the Go project directory:
```bash
cd realtimechat
```

2. Install dependencies:
```bash
go mod download
```

3. Run the server:
```bash
go run main.go
```

The server will start at `http://localhost:8082`

### Project Structure

```
realtimechat/
├── main.go           # Main application file
├── go.mod           # Go module definition
├── go.sum           # Go module checksums
├── messages/
│   └── messages.go  # Message handling logic
├── users/
│   └── users.go     # User management logic
└── ui/
    ├── index.html   # Web client interface
    ├── index.css    # Styles for web client
    └── index.js     # Client-side JavaScript
```

### API Endpoints

- `GET /` - Serves the web client interface
- `POST /register` - Register new user
- `POST /login` - User login
- `GET /users` - List all users
- `GET /ws` - WebSocket connection endpoint
- `GET /messages` - Get chat history

### UI 
- cd ui
- open index.html in your browser
- Register a user
- login as the registered user

### Example Usage

```bash
# Register a new user
curl -X POST http://localhost:8082/register \
  -H "Content-Type: application/json" \
  -d '{"firstName":"John","lastName":"Doe"}'

# Login
curl -X POST http://localhost:8082/login \
  -H "Content-Type: application/json" \
  -d '{"firstName":"John","lastName":"Doe"}'
```

### WebSocket Connection

Connect to WebSocket endpoint:
```
ws://localhost:8080/ws?userId=<user_id>
```

Message format:
```json
{
    "senderId": 1,
    "receiverId": 2,
    "content": "Hello!"
}
```

### Dependencies

```go
require (
    github.com/gorilla/websocket v1.5.0
    github.com/gin-gonic/gin v1.9.1
    github.com/gin-contrib/cors v1.4.0
)
```

### Web Client

The Go implementation includes a built-in web client interface accessible at `http://localhost:8080`. The UI provides:

- User registration and login
- Real-time chat interface
- User list
- Message history
- Online status indicators

## Contributors
- Kevin Chemutai
- Krishna Teja Nuthalapati
- Milan Bista
- Shimon Bhandari
- Soniya Padamati
- Venkata Mounisha Yarava

