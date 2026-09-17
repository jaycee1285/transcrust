# Hook contract

On user-prompt submit, pipe the submitted prompt text (or the hook's JSON with a `prompt` field) to `~/.local/bin/corpus-match` on stdin. Ignore output and exit code; never block submission.
