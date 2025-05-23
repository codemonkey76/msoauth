# msoauth

<!--toc:start-->
- [msoauth](#msoauth)
  - [Features](#features)
  - [Requirements](#requirements)
  - [Installation](#installation)
  - [Configuration](#configuration)
  - [Usage](#usage)
  - [Integration Example (NeoMutt)](#integration-example-neomutt)
  - [License](#license)
<!--toc:end-->

A simple CLI tool for retrieving, refreshing, and printing Microsoft OAuth2
tokens via the Device Code flow. Useful for tools like NeoMutt with
OAuth2-based accounts.

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
  - Scopes (e.g. <https://graph.microsoft.com/.default>)

## Installation

```bash
cargo install --path .
```

## Configuration

Create the file:

```bash
~/.config/msoauth/config.toml
```

With the contents:

```toml
client_id = "YOUR_CLIENT_ID"
client_secret = "YOUR_CLIENT_SECRET"
tenant_id = "YOUR_TENANT_ID"
scope = "https://graph.microsoft.com/.default"
```

To obtain these values:

1. Go to [https://portal.azure.com](https://portal.aszure.com)
2. Navigate to **Azure Active Directory > App Registrations**
3. Register a new app
4. Under **Overview**, copy the `Application (cliend) ID` and
`Directory (tenant) ID`.
5. Under **Certificates & secrets**, create a new client secret.
6. Under **API Permissions**. Add `Microsoft Graph > Delegated | User.Read` or
other needed scopes.

## Usage

```bash
msoauth --login         # Start device login flow
msoauth --refresh       # Refresh the token if expired
msoauth --print-token   # Print current access token (refresh if needed)
msoauth --clear-token   # Delete the saved token file
msoauth                 # Default, try refresh, fallback to login
```

Token is stored at:

```bash
~/.config/neomutt/token.json
```

## Integration Example (NeoMutt)

```bash
set imap_pass="`msoauth --print-token`"
```

## License

MIT
