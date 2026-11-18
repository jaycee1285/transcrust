#!/usr/bin/env bash
# Fetch the external word lists into data/. Hashes pinned in report.md.
set -euo pipefail
cd "$(dirname "$0")/data"
curl -sfL https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict -o cmudict.dict
curl -sfL https://raw.githubusercontent.com/cmusphinx/cmudict/master/LICENSE -o cmudict.LICENSE
curl -sfL https://raw.githubusercontent.com/first20hours/google-10000-english/master/20k.txt -o 20k.txt
curl -sfL https://raw.githubusercontent.com/first20hours/google-10000-english/master/LICENSE.md -o 20k.LICENSE.md
sha256sum cmudict.dict 20k.txt
