# Contributing to Terrier

## Prerequisites

Terrier uses [devenv](https://devenv.sh) for development environment management, which provides:

- Reproducible development environments
- Automatic service management (PostgreSQL, Redis, MinIO)
- Pre-commit hooks

## Required Tools

1. Nix (package manager)

    Install Nix using the Determinate Systems installer (recommended for macOS/Linux):

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
    ```

    Follow https://github.com/nix-darwin/nix-darwin to install nix-darwin (for macOS), which the rest of this doc uses.
    Then run the below to update when you make a change to the flake:

    ```bash
    sudo darwin-rebuild switch --flake /etc/nix-darwin
    ```

    Or see [nixos.org/download](https://nixos.org/download/) for other options, including for WSL.

2. [devenv](https://devenv.sh/) (development environment manager)

    It's recommended to add `pkgs.devenv` to your Nix environment. On nix-darwin, you can find the relevant file in `/etc/nix-darwin/`.

    ```nix
    environment.systemPackages = with pkgs; [
      devenv
    ];
    ```

    Or see [devenv.sh/getting-started](https://devenv.sh/getting-started/#2-install-devenv) for all of the installation methods.

3. [direnv](https://direnv.net/) (automatically load the environment)

    Once again, it's recommended to do this in your Nix environment:

    ```nix
    programs.direnv = {
      enable = true;
      nix-direnv.enable = true;
    };
    ```

    Or see [direnv.net/getting-started](https://direnv.net/#getting-started) for all of the installation methods.

## Getting Started

1. Install the recommended VS Code extensions.

    You may have a prompt from the `direnv` extension asking you to allow the environment, which you should accept.

2. Allow `direnv` in your terminal:

    ```bash
    direnv allow
    ```

    This will download and build the necessary development environment. It may take a while.

3. Create the environment variables:

    ```bash
    cp example/.env.dev .env
    ```

    Then, edit the `.env` file to set the necessary environment variables. You need to set `OIDC_CLIENT_SECRET` and change `ADMIN_EMAILS` to your Andrew email.

    Configure your development OIDC client with the following redirect URIs:
    - Web: `http://localhost:8080/auth/callback`
    - Mobile: `terrier://auth/callback`

4. Start the development server:

    ```bash
    just dev
    ```

    This starts all services (PostgreSQL, Redis, MinIO) in the background and launches the Dioxus development server with hot reloading. Visit [http://localhost:8080](http://localhost:8080) to see the application.

## Available Commands

Run `just` to see all available commands:

```bash
Available recipes:
    attach             # Display service logs
    clean              # Clean devenv state (removes all service data)
    dev                # Start development server
    down               # Stop development server
    fresh              # Fresh database (drop all tables and reapply migrations)
    generate-entities  # Generate entities from database
    help               # Show this help message
    init               # Start database, run migrations, and generate entities
    migrate            # Run database migrations
    new-migration NAME # Create new migration
    status             # Check migration status
```

Every time you make a change to [devenv.nix](./nix/devenv.nix), you should rebuild the development environment:

```bash
direnv reload
```
