# Rust Redis Clone

This is a simple Redis clone written in Rust, aimed at providing a basic implementation of key-value storage and retrieval functionalities similar to Redis.

## Features

- **Key-Value Storage**: Store and retrieve data using a key-value pair system.
- **String Data**: Supports storage and retrieval of string data.
- **Basic Commands**: Implements basic Redis-like commands such as `SET`, `GET`, `DEL`, etc.
- **Concurrency**: Utilizes Rust's concurrency features for efficient parallel processing.

## Installation

1. Make sure you have Rust installed. If not, you can get it from [rustup.rs](https://rustup.rs/).
2. Clone this repository:

   ```bash
   git clone https://github.com/Onizuka893/redis-rust.git
   ```

3. Navigate into the project directory:

   ```bash
   cd redis-rust
   ```

4. Build the project:

   ```bash
   ./spawn_redis_server.sh
   ```

## Usage

1. Use a Redis client to connect to the server. By default, the server listens on `localhost:6379`.

2. Start sending Redis commands similar to the ones you use with the actual Redis server.

## Example

```bash
$ redis-cli
127.0.0.1:6379> SET mykey "hello"
OK
127.0.0.1:6379> GET mykey
"hello"
```

## Contributing

Contributions are welcome! If you find any bugs or want to propose new features, feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
