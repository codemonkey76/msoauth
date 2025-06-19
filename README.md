# MSOAuth

MSOAuth is a simple command-line tool for obtaining and refreshing Microsoft OAuth2 tokens.

## Installation

To install the MSOAuth tool, you'll need to have Rust and Cargo installed on your system. Once you have them, you can build and install the project using:

```bash
cargo install --path .
```

This will create an executable in the `target/release` directory.

## Configuration

Create a configuration file `config.toml` located in the `msoauth` directory within your system's config directory, e.g., `~/.config/msoauth/config.toml`. The configuration file should look like this:

```toml
client_id = "YOUR_CLIENT_ID"
client_secret = "YOUR_CLIENT_SECRET"
tenant_id = "YOUR_TENANT_ID"
scope = "https://graph.microsoft.com/.default"
```

To obtain these values:

1. Go to [https://portal.azure.com](https://portal.azure.com)
2. Navigate to **Azure Active Directory > App Registrations**
3. Register a new app
4. Under **Overview**, copy the `Application (client) ID` and `Directory (tenant) ID`.
5. Under **Certificates & secrets**, create a new client secret.
6. Under **API Permissions**, add `Microsoft Graph > Delegated | User.Read` or other needed scopes.

## Usage

The MSOAuth tool provides several command-line options:

- `--print-token`: Prints the current access token if it's valid. If the token is expired or close to expiring, it will attempt to refresh it.
- `--refresh`: Forces a token refresh.
- `--login`: Initiates a device login flow to obtain a new token.
- `--clear-token`: Deletes the saved token file.

Example usage:

```bash
msoauth --login         # Start device login flow
msoauth --refresh       # Refresh the token if expired
msoauth --print-token   # Print current access token (refresh if needed)
msoauth --clear-token   # Delete the saved token file
msoauth                 # Default, try refresh, fallback to login
```

### Integration Example (mbsync)

You can use MSOAuth in your `mbsyncrc` file with:

```plaintext
PassCmd "msoauth --print-token"
```

This will ensure that only the token is printed and no other messages interfere with the mbsync operations.

### Integration Example (NeoMutt)

```bash
set imap_pass="`msoauth --print-token`"
```

## Features

- Authenticates using Microsoft OAuth2 Device Code flow
- Automatically saves/refreshes access tokens
- Prints access token for use in scripts or email clients
- Logs activity via tracing
- Friendly error messages and self-healing default mode

## Requirements

- Rust (use rustup to install)
- A registered Azure AD app with the following:
  - client_id
  - tenant_id
  - client_secret
  - Scopes (e.g., <https://graph.microsoft.com/.default>)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.
