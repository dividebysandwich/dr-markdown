# Dr. Markdown

A full-stack Rust application for online editing of markdown documents, featuring a REST API backend with SQLite storage and a Leptos-powered web frontend.

## Features

- **User Authentication**: Username/password authentication with JWT tokens
- **Document Management**: Create, edit, rename, and delete markdown documents
- **Real-time Preview**: Switch between edit and preview modes
- **Responsive UI**: Clean, modern interface with document sidebar
- **Configurable Registration**: Option to disable new user registration
- **Markdown Rendering**: Full markdown support with syntax highlighting

## Architecture

- **Backend**: Axum REST API with SQLite database
- **Frontend**: Leptos WebAssembly application
- **Authentication**: JWT-based with bcrypt password hashing
- **Database**: SQLite with SQLx migrations

## Prerequisites

- Rust (1.70+)
- `wasm-pack` for building the frontend
- `trunk` for serving the frontend during development

```bash
# Install required tools
cargo install trunk wasm-pack
```

## Setup

1. **Clone and setup the workspace**:
```bash
git clone <repository-url>
cd markdown-editor
```

2. **Backend setup**:
```bash
cd backend
cargo sqlx database create
cargo sqlx migrate run
cp ../.env.example .env
# Edit .env file with your configuration
# alternatively set environment variables such as DATABASE_URL
cargo run --release
```

The backend will start on `http://localhost:3001`

3. **Frontend setup** (in a new terminal):
```bash
cd frontend
trunk serve --release
```

The frontend will start on `http://localhost:8080`

## Configuration

The application can be configured via environment variables:

- `DATABASE_URL`: SQLite database path (default: `./documents.db`)
- `JWT_SECRET`: Secret key for JWT tokens (change in production!)
- `SERVER_PORT`: Backend server port (default: 3001)
- `ALLOW_REGISTRATION`: Allow new user registration (default: true)

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user
- `GET /api/auth/profile` - Get current user profile

### Documents
- `GET /api/documents` - List user's documents
- `POST /api/documents` - Create new document
- `GET /api/documents/:id` - Get document by ID
- `PUT /api/documents/:id` - Update document
- `DELETE /api/documents/:id` - Delete document

## Development

### Running in Development

1. **Backend** (terminal 1):
```bash
cd backend
cargo watch -x run
```

2. **Frontend** (terminal 2):
```bash
cd frontend
trunk serve --open
```

### Building for Production

1. **Backend**:
```bash
cd backend
cargo build --release
```

2. **Frontend**:
```bash
cd frontend
trunk build --release
```

The built frontend files will be in `frontend/dist/`.

### Database Migrations

Migrations are automatically applied when the backend starts. Migration files are located in `backend/migrations/`.

## Project Structure

```
markdown-editor/
├── Cargo.toml              # Workspace configuration
├── .env.example            # Environment variables template
├── README.md               # This file
├── backend/                # REST API backend
│   ├── Cargo.toml          # Backend dependencies
│   ├── src/
│   │   ├── main.rs         # Application entry point
│   │   ├── auth.rs         # Authentication logic
│   │   ├── config.rs       # Configuration management
│   │   ├── database.rs     # Database operations
│   │   ├── handlers.rs     # HTTP request handlers
│   │   ├── models.rs       # Data models
│   │   └── routes.rs       # Route definitions
│   └── migrations/         # Database migrations
│       └── 001_initial.sql # Initial schema
└── frontend/               # Leptos web frontend
    ├── Cargo.toml          # Frontend dependencies
    ├── index.html          # HTML template
    └── src/
        ├── main.rs         # Frontend entry point
        ├── app.rs          # Main app component
        ├── api.rs          # API client
        ├── auth.rs         # Authentication context
        ├── models.rs       # Frontend data models
        ├── components/     # Reusable components
        │   ├── mod.rs
        │   └── sidebar.rs  # Document sidebar
        └── pages/          # Page components
            ├── mod.rs
            ├── home.rs     # Home page with editor
            ├── login.rs    # Login page
            ├── register.rs # Registration page
            └── document.rs # Document view page
```

## Security Considerations

- **JWT Secret**: Change the default JWT secret in production
- **Password Hashing**: Uses bcrypt with default cost factor
- **CORS**: Currently configured for development (permissive)
- **Input Validation**: Server-side validation on all inputs
- **SQL Injection**: Protected by SQLx parameter binding

## Customization

### Disabling Registration

To prevent new users from registering, set the environment variable:
```bash
ALLOW_REGISTRATION=false
```

### Markdown Styling

The frontend includes custom CSS for markdown rendering in `frontend/index.html`. Modify the styles in the `<style>` section to customize the appearance.

### Database Schema

The database schema is defined in `backend/migrations/001_initial.sql`. To modify the schema:

1. Create a new migration file in `backend/migrations/`
2. Add your SQL changes
3. Restart the backend to apply migrations

## Troubleshooting

### Common Issues

1. **CORS Errors**: Make sure the backend is running on port 3001
2. **WebAssembly Errors**: Ensure `wasm-pack` is installed and up to date
3. **Database Errors**: Check that the database directory is writable
4. **JWT Errors**: Verify the JWT secret is set correctly

### Logs

The backend uses Rust's standard logging. Set `RUST_LOG=debug` for verbose output:
```bash
RUST_LOG=debug cargo run
```

