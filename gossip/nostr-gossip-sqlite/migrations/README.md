# Migrations

## Notes

SQLx creates a checksum of the migrations and compares it to the database.
This means that also comments are included in the checksum. If you change
comments, the hash will change and will break the migrations!

## SQL file format

- Use a tab for indentation
- Leave an empty line at the end of the file
- **DON'T use `--` comments** (schema comments are documented [here](../doc/database.md))
