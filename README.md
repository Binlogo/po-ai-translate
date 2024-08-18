# Gettext PO File Translator

This Rust project is designed to automate the translation of gettext PO files using the Moonshot API. It's particularly useful for translating internationalization files in software projects.

## Features

- Loads and parses PO files
- Translates untranslated messages using the Moonshot API
- Supports batch translation to optimize API usage
- Preserves existing translations and file structure
- Saves the translated PO file back to disk

## Prerequisites

- Rust programming environment
- Moonshot API key (set as an environment variable)

## Usage

1. Set your Moonshot API key as an environment variable:

   ```
   export MOONSHOT_API_KEY=your_api_key_here
   ```

2. Run the program with the path to your PO file:

   ```
   cargo run -- path/to/your/file.po
   ```
