# MSOAuth

MSOAuth is a simple command-line tool for obtaining and refreshing Microsoft OAuth2 tokens.

## Installation

To install the MSOAuth tool, you'll need to have Rust and Cargo installed on your system. Once you have them, you can build the project using:

```bash
cargo build --release
```

This will create an executable in the `target/release` directory.

## Usage

The MSOAuth tool provides several command-line options:

- `--print-token`: Prints the current access token if it's valid. If the token is expired or close to expiring, it will attempt to refresh it.
- `--refresh`: Forces a token refresh.
- `--login`: Initiates a device login flow to obtain a new token.

### Example

You can use MSOAuth in your `mbsyncrc` file with:

```plaintext
PassCmd "msoauth --print-token"
```

This will ensure that only the token is printed and no other messages interfere with the mbsync operations.

## Configuration

MSOAuth requires a configuration file `config.toml` located in the `msoauth` directory within your system's config directory. The configuration file should look like this:

```toml
client_id = "your_client_id"
client_secret = "your_client_secret"
tenant_id = "your_tenant_id"
scope = "your_scope"
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.

