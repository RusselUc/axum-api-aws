# Axum Auth API (Demo) 🦀

This is a backend API built with Rust using the [Axum](https://github.com/tokio-rs/axum) framework, integrating AWS services such as Cognito and DynamoDB.  
The main goal of this project is to learn how to structure an API in Rust and test basic user authentication using AWS Cognito.

---

## 🛠️ Technologies Used

- **Rust** (edition 2021)
- **Axum** – Asynchronous web framework
- **Tokio** – Async runtime
- **AWS Cognito** – User authentication
- **AWS DynamoDB** – Data persistence

---

## 📂 Project Structure

The project follows a modular layered architecture:

```
src/
├── domains/       # Domain logic (e.g., auth)
├── handlers/      # HTTP request handlers
├── models/        # Data structures
├── routes/        # API route definitions
├── services/      # External service initializers (Cognito, DynamoDB)
├── main.rs        # Entry point
```

---

## 🚀 Running the Project

Make sure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- An AWS account with access to Cognito and DynamoDB
- AWS credentials configured (via `~/.aws/credentials` or environment variables)

Then simply run:

```bash
cargo run
```

---

## 🔐 Available Routes

| Method | Endpoint          | Description                      |
|--------|-------------------|----------------------------------|
| POST   | `/users`          | Create a new user                |
| GET    | `/users`          | List all users                   |
| POST   | `/users/confirm`  | Confirm a user via Cognito       |

---

## 🧪 Project Status

> This is a **demo project** built for learning purposes only.  
It does not yet include advanced validation, detailed error handling, or full persistence logic in DynamoDB.

---

## 📅 Demo

This project will be presented as a technical demo on **June 17, 2025**.

---

## 👥 Authors

- [smtamay russeluc]
