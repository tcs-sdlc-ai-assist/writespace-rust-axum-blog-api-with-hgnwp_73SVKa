# Changelog

All notable changes to the WriteSpace project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-01-15

### Added

- **JWT Authentication**
  - User login with username and password returning a signed JWT token
  - User registration with display name, username, and password
  - Token-based session management with 24-hour expiration
  - Secure password hashing using bcrypt with cost factor 12

- **Blog Post CRUD**
  - Create new blog posts with title and content (authenticated users)
  - Read individual posts with author information and edit/delete permissions
  - Update existing posts (owner or admin only)
  - Delete posts (owner or admin only)
  - List all posts for authenticated users, latest 3 posts for public visitors

- **Admin Dashboard**
  - Platform statistics overview: total posts, total users, total admins
  - Recent posts listing with quick navigation
  - Protected admin-only routes with role-based middleware

- **User Management**
  - Admin-only user listing with role badges and creation dates
  - Create new users with configurable roles (user or admin)
  - Delete users with protection against deleting the default admin or self
  - Input validation for usernames, passwords, and display names

- **Role-Based Access Control**
  - Two roles: `user` and `admin`
  - `optional_auth` middleware for public routes with optional user context
  - `require_auth` middleware for authenticated-only routes
  - `require_admin` middleware for admin-only routes
  - Middleware layering with correct execution order (auth before admin check)

- **Static Frontend**
  - Home page with latest posts and call-to-action sections
  - Blog listing page with post cards showing author avatars and role badges
  - Individual post view with formatted content and edit/delete controls
  - Post creation and editing form with character count and validation
  - Login and registration pages with client-side validation
  - Admin dashboard page with statistics cards and recent posts
  - User management page with create user form and delete confirmation modal
  - Shared `app.js` module with JWT decoding, API fetch wrapper, auth guards, navigation rendering, toast notifications, and UI utilities
  - Responsive design using Tailwind CSS via CDN

- **Database**
  - PostgreSQL database with `users` and `posts` tables
  - UUID primary keys with `uuid-ossp` extension
  - Automatic timestamping with `TIMESTAMPTZ` columns
  - Foreign key constraint from posts to users with cascade delete
  - Indexes on `posts.created_at` and `posts.author_id`
  - Automatic database migrations on startup using sqlx
  - Default admin user seeding from environment variables

- **Vercel Deployment**
  - Serverless Rust function via `@vercel/rust` builder
  - Static file serving via `@vercel/static` builder
  - Request routing: `/api/*` to Rust handler, `/*` to static files
  - Body conversion between `vercel_runtime::Body` and `axum::body::Body`
  - CORS configuration allowing all origins with standard headers

- **Observability**
  - Structured logging with `tracing` and `tracing-subscriber`
  - Environment-configurable log levels via `RUST_LOG`
  - HTTP request tracing via `tower_http::trace::TraceLayer`

- **Error Handling**
  - Custom `AppError` enum with variants: NotFound, Unauthorized, Forbidden, Conflict, BadRequest, InternalError
  - Consistent JSON error responses with `{ "error": "message" }` format
  - Automatic conversion from `sqlx::Error`, `bcrypt::BcryptError`, and `jsonwebtoken::errors::Error`
  - Unique constraint violation detection for duplicate username handling