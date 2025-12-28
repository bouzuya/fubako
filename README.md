# fubako

A personal wiki/knowledge base system with markdown support and bi-directional linking.

> **Note:** fubako is designed to work with a local filesystem. All pages are stored as markdown files in a directory you specify.

## Features

- **Markdown-based pages**: Store your knowledge in simple markdown files
- **Bi-directional linking**: Track both outgoing links and backlinks between pages
- **Live preview server**: View and navigate your wiki with a local web server
- **Auto-reload**: File watcher automatically updates the index when pages change
- **Image management**: Add, pull, and push images to Google Cloud Storage
- **Full-text search**: Search across all pages
- **Page editing**: Edit pages directly from the command line

## Requirements

- Rust 1.83.0 or later (edition 2024)
- XDG-compliant configuration file (see Configuration section)

## Installation

```bash
cargo install --git https://github.com/bouzuya/fubako
```

## Configuration

Create a `config.json` file in the XDG config directory (`~/.config/fubako/config.json` on Linux):

```json
{
  "data_dir": "/path/to/your/wiki/pages",
  "image_sync": {
    "bucket_name": "your-gcs-bucket-name",
    "google_application_credentials": "/path/to/credentials.json",
    "object_prefix": "images/"
  },
  "port": 3000
}
```

Configuration options:
- `data_dir` (required): Directory where markdown files are stored
- `image_sync` (optional): Google Cloud Storage configuration for image synchronization
  - `bucket_name`: GCS bucket name for images
  - `google_application_credentials`: Path to GCS credentials JSON file
  - `object_prefix`: Prefix for uploaded image objects
- `port` (optional): Port for the local server (default: 3000)

## Usage

### Create a new page

```bash
fubako new
```

Creates a new markdown file with a timestamp-based ID (e.g., `20240620T123456Z.md`) and opens it in your default editor.

### Edit a page

```bash
fubako edit <page_id>
```

Opens the specified page in your default editor.

### Get page content

```bash
fubako get <page_id>
```

Displays the content of the specified page.

### Start the web server

```bash
fubako serve
```

Opens a local web server at `http://127.0.0.1:3000` (or your configured port) where you can:
- Browse all pages at `/`
- Search pages using the search box
- View individual pages at `/{page_id}`
- See links and backlinks for each page

The server automatically reloads when files change.

### Manage images

#### Add an image

```bash
fubako image add <path/to/image.png>
```

Copies an image to the `images` subdirectory in your data directory.

#### Push images to remote storage

```bash
fubako image push [image_name]
```

Uploads a local image to Google Cloud Storage. Requires `image_sync` configuration.

#### Pull images from remote storage

```bash
fubako image pull [image_name]
```

Downloads an image from Google Cloud Storage to your local images directory. Requires `image_sync` configuration.

## Page Format

Pages are markdown files with timestamps as IDs. The format is `YYYYMMDDTHHMMSSµ.md` (where µ is microseconds with 'Z' suffix for UTC).

### Linking between pages

Use markdown links with the page ID:

```markdown
# My Page

Link to another page: [20240620T123456Z]

Or with custom text: [link text](20240620T123456Z)
```

Both shortcut links `[PageID]` and explicit links are supported.

### Page title

The first H1 heading in the markdown file is used as the page title:

```markdown
# This is the Page Title

Content goes here...
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── config.rs            # Configuration loading (XDG-compliant)
├── page_id.rs           # Page ID type and validation
├── page_io.rs           # File I/O operations
├── page_meta.rs         # Metadata extraction from markdown
├── util.rs              # Utility functions
└── subcommand/          # CLI subcommands
    ├── mod.rs
    ├── new.rs           # Create new pages
    ├── edit.rs          # Edit pages
    ├── get.rs           # Get page content
    ├── serve.rs         # Local web server
    │   ├── handler.rs   # HTTP request handlers
    │   └── index.rs     # Index building and search
    └── image/           # Image management
        ├── add.rs       # Add images locally
        ├── pull.rs      # Pull from GCS
        └── push.rs      # Push to GCS

templates/
├── list.html            # Index page template
├── list_titles.html     # Title list template
└── get.html             # Individual page template

public/
├── scripts/
│   └── index.js         # Client-side JavaScript
└── styles/
    └── index.css        # Styles
```

## License

This project is dual-licensed under either:

- MIT License (see <https://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 (see <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.
