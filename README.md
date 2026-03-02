# iOS Backup Explorer

A Rust toolset for exploring and extracting files from unencrypted iOS backups.
It decodes the `Manifest.mbdb` index and maps hashed filenames back to their original directory structures.

## Project Structure

This project is organized as a Cargo workspace:

- **`ios_backup_core`**: A library to handle MBDB parsing.
- **`ios_backup_cli`**: A command-line utility for batch extraction.
- **`ios_backup_web`**: A WebAssembly-powered [web application](https://calixte.github.io/ios_backup_explorer/) for interactive browsing and extraction.

## Usage

### Command Line Interface

The CLI requires the path to your unencrypted iOS backup directory (the one containing `Manifest.mbdb`).

#### List all file types in a backup:
```bash
cargo run -r -p ios_backup_cli <BACKUP_DIR> list-extensions
```

#### Extract specific files:
Extract only `.jpg` and `.png` files to a `photos` directory:
```bash
cargo run -r -p ios_backup_cli <BACKUP_DIR> extract ./photos -i jpg -i png
```

#### Exclude noisy extensions:
Extract everything except `.plist` and `.db` files:
```bash
cargo run -r -p ios_backup_cli <BACKUP_DIR> extract ./output -e plist -e db
```

### Web Interface

The web application is deployed at [https://calixte.github.io/ios_backup_explorer/](https://calixte.github.io/ios_backup_explorer/).

To run it locally:

1. Start the dev server: `trunk serve --config ios_backup_web`.
2. Open `http://localhost:8080` in your browser.
3. Select your backup folder to begin exploring.

## License

This project is licensed under the [MIT License](LICENSE).
