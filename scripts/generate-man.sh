#!/bin/bash
# Generate man page for commit-wizard

set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
DATE=$(date +"%B %Y")

cat <<EOF
.TH COMMIT-WIZARD 1 "$DATE" "commit-wizard $VERSION" "User Commands"
.SH NAME
commit-wizard \- A CLI tool to help create better commit messages
.SH SYNOPSIS
.B commit-wizard
[\fIOPTIONS\fR]
.br
.B commit-wizard
[\fICOMMAND\fR]
.SH DESCRIPTION
.B commit-wizard
is an interactive CLI tool that helps developers create better commit messages
following the Conventional Commits specification. It intelligently groups staged
files by commit type and scope, and optionally uses AI to generate commit messages.

Features include:
.IP \[bu] 2
Interactive TUI with keyboard navigation
.IP \[bu]
Automatic conventional commit formatting
.IP \[bu]
Smart file grouping by type and scope
.IP \[bu]
AI-powered message generation (optional)
.IP \[bu]
Editor integration for manual editing
.IP \[bu]
Automatic ticket number detection from branch names

.SH OPTIONS
.TP
.BR \-\-ai
Enable AI-powered commit message generation using GitHub Models or OpenAI API.
Requires GITHUB_TOKEN or OPENAI_API_KEY environment variable.

.TP
.BR \-r ", " \-\-repo " \fIPATH\fR"
Specify the git repository path. Defaults to current directory.

.TP
.BR \-v ", " \-\-verbose
Enable verbose output for debugging.

.TP
.BR \-\-env\-override
Override existing environment variables with values from .env file.

.TP
.BR \-h ", " \-\-help
Print help information.

.TP
.BR \-V ", " \-\-version
Print version information.

.SH COMMANDS
.TP
.B test-token
Test GitHub and OpenAI API tokens for AI functionality.
Validates token format, API authentication, and endpoint accessibility.

.SH KEYBOARD CONTROLS
When running the interactive TUI:

.TP
.BR "↑/↓ or k/j"
Navigate between commit groups

.TP
.BR e
Edit commit message in external editor

.TP
.BR a
Generate commit message with AI (requires --ai flag)

.TP
.BR c
Commit selected group

.TP
.BR C
Commit all groups

.TP
.BR Ctrl+L
Clear status message

.TP
.BR "q or Esc"
Quit application

.SH ENVIRONMENT VARIABLES
.TP
.B GITHUB_TOKEN, GH_TOKEN
GitHub Personal Access Token for AI features (GitHub Models API).
Requires 'read:user' scope. Create at: https://github.com/settings/tokens

.TP
.B OPENAI_API_KEY
OpenAI API key for AI features (fallback option).
Create at: https://platform.openai.com/api-keys

.TP
.B EDITOR, VISUAL
Preferred text editor for editing commit messages.
Defaults to vi/vim if not set.

.SH FILES
.TP
.B .env
Optional environment file for storing API tokens.
Use --env-override flag to load this file.

.TP
.B .git
Git repository directory. commit-wizard operates on staged changes
in the current git repository.

.SH EXAMPLES
.TP
Stage files and run the wizard:
.B git add .
.br
.B commit-wizard

.TP
Use AI-powered commit message generation:
.B export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
.br
.B commit-wizard --ai

.TP
Specify a different repository:
.B commit-wizard --repo /path/to/repo

.TP
Test API tokens before use:
.B commit-wizard test-token

.TP
Use with .env file:
.B echo 'GITHUB_TOKEN=ghp_xxx' > .env
.br
.B commit-wizard --env-override --ai

.SH EXIT STATUS
.TP
.B 0
Success

.TP
.B 1
General error (no staged files, git error, API error, etc.)

.SH CONVENTIONAL COMMITS
commit-wizard follows the Conventional Commits specification (https://www.conventionalcommits.org/).

Commit message format:
.br
.B <type>(<scope>): <description>
.br
.br
[optional body]
.br
.br
[optional footer(s)]

Common types:
.br
.B feat:
New feature
.br
.B fix:
Bug fix
.br
.B docs:
Documentation changes
.br
.B style:
Code style changes (formatting, etc.)
.br
.B refactor:
Code refactoring
.br
.B test:
Test changes
.br
.B chore:
Build process or auxiliary tool changes

.SH BUGS
Report bugs at: https://github.com/jfheinrich-eu/commit-wizard/issues

.SH AUTHOR
Written by jfheinrich <joerg@jfheinrich.eu>

.SH COPYRIGHT
Copyright © 2025 jfheinrich. License: MIT
.br
This is free software: you are free to change and redistribute it.

.SH SEE ALSO
.BR git (1),
.BR git-commit (1),
.BR git-add (1)

Full documentation: https://github.com/jfheinrich-eu/commit-wizard
EOF
