# Contributing to Namada REST API

Thanks for your interest in contributing! Here's how you can help:

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```sh
   git clone https://github.com/YOUR_USERNAME/namada-api.git
   cd namada-api
   ```

## Making Changes

1. Create a new branch:
   ```sh
   git checkout -b feature/your-feature-name
   ```

2. Make your changes

3. Test your changes:
   ```sh
   cargo test
   ```

4. Format your code:
   ```sh
   cargo fmt
   ```

## Submitting Changes

1. Push your changes:
   ```sh
   git push origin feature/your-feature-name
   ```

2. Create a Pull Request on GitHub

## Project Structure

```
src/
├── api/          # API route handlers
├── config/       # Configuration management
├── models/       # Data models and types
├── services/     # Business logic
└── utils/        # Shared utilities
```

## Guidelines

- Add tests for new functionality
- Update documentation when needed
- Keep your changes focused and well-described
- Make sure all tests pass before submitting

## Questions?

Open an issue if you have any questions!

## License

By contributing, you agree that your contributions will be licensed under the project's GPL-3.0 License. 