# Contributing

Thank you for showing interests in contributing to this project. Please follow this guide before contributing to this project.

## Don't know where to start?

If you dont know from where to start, check the issue tracker.

## Check for existing PRs

Before making a pull request, make sure to check if someone else has already made a PR for that specific topic. Avoid duplicated PRs.

## Commits

Follow [conventional commits](https://www.conventionalcommits.org/en) format while committing. Conventional commit dovetails with semver, by describing the features, fixes, and breaking changes made in commit messages. The following specification is adapted from [conventionalcommits.org](https://www.conventionalcommits.org/en).

The commit message should be structured as follows:

```text
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

##### The commit contains the following structural elements, to communicate intent to the consumers of your library:

1. `fix`: a commit of the type fix patches a bug in your codebase (this correlates with `PATCH` in semantic versioning).
2. `feat`: a commit of the type feat introduces a new feature to the codebase (this correlates with `MINOR` in semantic versioning).
3. `BREAKING CHANGE`: a commit that has the text `BREAKING CHANGE`: at the beginning of its optional body or footer section introduces a breaking API change (correlating with MAJOR in semantic versioning). A `BREAKING CHANGE` can be part of commits of any type.
4. Others: commit types other than `fix:` and `feat:` are allowed, for example [@commitlint/config-conventional](https://npm.im/@commitlint/config-conventional) (based on the [Angular convention](https://github.com/angular/angular/blob/68a6a07/CONTRIBUTING.md#commit)) recommends `chore:`, `docs:`, `style:`, `refactor:`, `perf:`, `test:`, and others.

### Examples
#### Commit message with description and breaking change in body

```text
feat: allow provided config object to extend other configs

BREAKING CHANGE: `extends` key in config file is now used for extending other config files
```

#### Commit message with optional `!` to draw attention to breaking change

```text
chore!: drop Node 6 from testing matrix

BREAKING CHANGE: dropping Node 6 which hits end of life in April
```

#### Commit message with no body

```text
docs: correct spelling of CHANGELOG
```

#### Commit message with scope

```text
feat(lang): add polish language
```

#### Commit message for a fix using an (optional) issue number.

```text
fix: correct minor typos in code

see the issue for details on the typos fixed

closes issue #12
```

### Specification

The key words `“MUST”`, `“MUST NOT”`, `“REQUIRED”`, `“SHALL”`, `“SHALL NOT”`, `“SHOULD”`, `“SHOULD NOT”`, `“RECOMMENDED”`, `“MAY”`, and `“OPTIONAL”` in this document are to be interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

1. Commits MUST be prefixed with a type, which consists of a noun, `feat`, `fix`, etc., followed by an OPTIONAL scope, and a REQUIRED terminal colon and space.
2. The type `feat` MUST be used when a commit adds a new feature to your application or library.
3. The type `fix` MUST be used when a commit represents a bug fix for your application.
4. A scope MAY be provided after a type. A scope MUST consist of a noun describing a section of the codebase surrounded by parenthesis, e.g., `fix(parser):`
5. A description MUST immediately follow the space after the type/scope prefix. The description is a short summary of the code changes, e.g., `fix: array parsing issue when multiple spaces were contained in string`.
6. A longer commit body MAY be provided after the short description, providing additional contextual information about the code changes. The body MUST begin one blank line after the description.
7. A footer of one or more lines MAY be provided one blank line after the body. The footer MUST contain meta-information about the commit, e.g., related pull-requests, reviewers, breaking changes, with one piece of meta-information per-line.
8. Breaking changes MUST be indicated at the very beginning of the body section, or at the beginning of a line in the footer section. A breaking change MUST consist of the uppercase text `BREAKING CHANGE`, followed by a colon and a space.
9. A description MUST be provided after the `BREAKING CHANGE:`, describing what has changed about the API, e.g., `BREAKING CHANGE: environment variables now take precedence over config files`.
10. Types other than `feat` and `fix` MAY be used in your commit messages.
11. The units of information that make up conventional commits MUST NOT be treated as case sensitive by implementors, with the exception of `BREAKING CHANGE` which MUST be uppercase.
12. A `!` MAY be appended prior to the `:` in the type/scope prefix, to further draw attention to breaking changes. `BREAKING CHANGE: description` MUST also be included in the body or footer, along with the `!` in the prefix.

## Formatting, Linting, Testing

Make sure to properly format the source code, check for linter errors and test the code before pushing.

## File Names

1. Use `PascalCase` format if the file belongs to a class. Ex: `NobuBrowser.ts`, `ProtocolService.ts`, etc.
2. Use `camelCase` format for the files that belong to functions. Ex: `contextMenu`, `appMenu`, etc.
3. Use `lowercase` format for other cases.

> 🎉 Happy coding!