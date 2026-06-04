.PHONY: format

# Format all markdown files in the repository.
format:
	~/bin/format_markdown.sh $(shell git ls-files '*.markdown' '*.md')
