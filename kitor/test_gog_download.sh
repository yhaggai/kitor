#!/bin/bash
# We need the user's token from the db.
sqlite3 src-tauri/kitor.db "SELECT access_token FROM gog_auth;" > token_b64.txt
