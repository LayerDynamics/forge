//! Shell command parser using monch parser combinators
//!
//! Parses shell command strings into an AST for execution.
//! Inspired by deno_task_shell but implemented from scratch.

use monch::{ParseError, ParseErrorFailure, ParseResult};

// ============================================================================
// AST Types
// ============================================================================

/// Top-level parsed shell command: a list of sequential commands.
#[derive(Debug, Clone, PartialEq)]
pub struct SequentialList {
    pub items: Vec<Sequence>,
}

/// A sequence of pipelines connected by && or ||.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub current: Pipeline,
    pub next: Option<Box<SequenceNext>>,
}

/// The next part of a sequence with its boolean operator.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceNext {
    pub op: BooleanListOp,
    pub sequence: Sequence,
}

/// Boolean operators in command sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanListOp {
    /// && - execute next only if previous succeeded
    And,
    /// || - execute next only if previous failed
    Or,
}

/// A pipeline of commands connected by |.
#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    pub commands: Vec<PipelineCommand>,
    pub negated: bool,
}

/// A single command in a pipeline with optional stdin/stdout.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineCommand {
    pub inner: Command,
}

/// The different types of commands.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Simple command with optional env vars, name, and args
    Simple(SimpleCommand),
    /// Subshell: ( commands )
    Subshell(Box<SequentialList>),
    /// If statement (not yet implemented)
    If(IfClause),
}

/// A simple command with optional assignments, command name, and args.
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleCommand {
    /// Environment variable assignments before command
    pub env_vars: Vec<EnvVar>,
    /// The command parts (name + args as Word expressions)
    pub args: Vec<Word>,
    /// Redirections
    pub redirects: Vec<Redirect>,
}

/// Environment variable assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvVar {
    pub name: String,
    pub value: Word,
}

/// A word in shell (can be composed of multiple parts).
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub parts: Vec<WordPart>,
}

impl Word {
    /// Create a word from a single string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Word {
            parts: vec![WordPart::Text(s.into())],
        }
    }

    /// Check if this word is empty.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Evaluate word to string (basic evaluation without expansion).
    pub fn to_string_lossy(&self) -> String {
        let mut result = String::new();
        for part in &self.parts {
            match part {
                WordPart::Text(s) => result.push_str(s),
                WordPart::SingleQuoted(s) => result.push_str(s),
                WordPart::DoubleQuoted(parts) => {
                    for p in parts {
                        if let WordPart::Text(s) = p {
                            result.push_str(s);
                        }
                        // Skip complex parts in lossy conversion
                    }
                }
                WordPart::Variable(name) => {
                    result.push('$');
                    result.push_str(name);
                }
                WordPart::CommandSubstitution(_) => {
                    result.push_str("$(...)");
                }
                WordPart::Tilde(path) => {
                    result.push('~');
                    if let Some(p) = path {
                        result.push_str(p);
                    }
                }
                _ => {}
            }
        }
        result
    }
}

/// Parts of a word (text, variables, command substitution, etc.).
#[derive(Debug, Clone, PartialEq)]
pub enum WordPart {
    /// Literal text
    Text(String),
    /// Single-quoted string (no expansion)
    SingleQuoted(String),
    /// Double-quoted string (allows variable expansion)
    DoubleQuoted(Vec<WordPart>),
    /// Variable reference: $VAR or ${VAR}
    Variable(String),
    /// Command substitution: $(cmd) or `cmd`
    CommandSubstitution(Box<SequentialList>),
    /// Tilde expansion: ~, ~/path, ~user
    Tilde(Option<String>),
    /// Glob pattern part: *, ?, [...]
    Glob(GlobPart),
    /// Arithmetic expansion: $((expr))
    Arithmetic(String),
}

/// Glob pattern components.
#[derive(Debug, Clone, PartialEq)]
pub enum GlobPart {
    /// * - matches any string
    Star,
    /// ? - matches any single character
    Question,
    /// [...] - character class
    CharClass(String),
    /// ** - recursive glob
    DoubleStar,
}

/// I/O redirection.
#[derive(Debug, Clone, PartialEq)]
pub struct Redirect {
    /// File descriptor (None = stdout for >, stdin for <)
    pub fd: Option<u32>,
    /// Type of redirection
    pub op: RedirectOp,
    /// Target (file path or fd number for >&)
    pub target: RedirectTarget,
}

/// Redirect operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectOp {
    /// < - input from file
    Input,
    /// > - output to file (truncate)
    Output,
    /// >> - output to file (append)
    Append,
    /// << - here-document
    HereDoc,
    /// <<< - here-string
    HereString,
    /// &> or >& - redirect both stdout and stderr
    OutputBoth,
    /// 2>&1 - redirect fd to another fd
    DupOutput,
    /// <& - duplicate input
    DupInput,
}

/// Target of a redirection.
#[derive(Debug, Clone, PartialEq)]
pub enum RedirectTarget {
    /// Redirect to/from file
    File(Word),
    /// Redirect to/from file descriptor
    Fd(u32),
    /// Here-document content
    HereDoc(String),
}

/// If clause (basic support).
#[derive(Debug, Clone, PartialEq)]
pub struct IfClause {
    pub condition: Box<SequentialList>,
    pub then_part: Box<SequentialList>,
    pub else_part: Option<Box<SequentialList>>,
}

// ============================================================================
// Parser Implementation
// ============================================================================

/// Parse a shell command string.
pub fn parse(input: &str) -> Result<SequentialList, ParseError<'_>> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(SequentialList { items: vec![] });
    }

    match parse_sequential_list(input) {
        Ok(("", result)) => Ok(result),
        Ok((remaining, _)) => Err(ParseError::Failure(ParseErrorFailure::new(
            remaining,
            format!("Unexpected trailing input: '{}'", remaining),
        ))),
        Err(e) => Err(e),
    }
}

/// Parse a sequential list (commands separated by ; or newlines, or connected by && ||).
fn parse_sequential_list(input: &str) -> ParseResult<'_, SequentialList> {
    let input = skip_whitespace(input);
    if input.is_empty() {
        return Ok((input, SequentialList { items: vec![] }));
    }

    let mut items = Vec::new();
    let mut remaining = input;

    loop {
        let remaining_trimmed = skip_whitespace(remaining);
        if remaining_trimmed.is_empty() {
            break;
        }

        // Skip leading semicolons or newlines
        if remaining_trimmed.starts_with(';') || remaining_trimmed.starts_with('\n') {
            remaining = &remaining_trimmed[1..];
            continue;
        }

        // Try to parse a sequence
        match parse_sequence(remaining_trimmed) {
            Ok((rest, seq)) => {
                items.push(seq);
                remaining = skip_whitespace(rest);

                // Check for separator
                if remaining.starts_with(';') || remaining.starts_with('\n') {
                    remaining = &remaining[1..];
                } else if remaining.is_empty()
                    || remaining.starts_with(')')
                    || remaining.starts_with('}')
                {
                    break;
                }
            }
            Err(_) if items.is_empty() => {
                return Err(ParseError::Failure(ParseErrorFailure::new(
                    remaining_trimmed,
                    "Expected command",
                )));
            }
            Err(_) => break,
        }
    }

    Ok((remaining, SequentialList { items }))
}

/// Parse a sequence (pipelines connected by && or ||).
fn parse_sequence(input: &str) -> ParseResult<'_, Sequence> {
    let input = skip_whitespace(input);
    let (remaining, pipeline) = parse_pipeline(input)?;

    let remaining = skip_whitespace(remaining);

    // Check for && or ||
    let (remaining, next) = if let Some(stripped) = remaining.strip_prefix("&&") {
        let remaining = skip_whitespace(stripped);
        let (remaining, next_seq) = parse_sequence(remaining)?;
        (
            remaining,
            Some(Box::new(SequenceNext {
                op: BooleanListOp::And,
                sequence: next_seq,
            })),
        )
    } else if let Some(stripped) = remaining.strip_prefix("||") {
        let remaining = skip_whitespace(stripped);
        let (remaining, next_seq) = parse_sequence(remaining)?;
        (
            remaining,
            Some(Box::new(SequenceNext {
                op: BooleanListOp::Or,
                sequence: next_seq,
            })),
        )
    } else {
        (remaining, None)
    };

    Ok((
        remaining,
        Sequence {
            current: pipeline,
            next,
        },
    ))
}

/// Parse a pipeline (commands connected by |).
fn parse_pipeline(input: &str) -> ParseResult<'_, Pipeline> {
    let input = skip_whitespace(input);

    // Check for negation
    let (input, negated) = if let Some(stripped) = input.strip_prefix('!') {
        let rest = skip_whitespace(stripped);
        (rest, true)
    } else {
        (input, false)
    };

    let mut commands = Vec::new();
    let mut remaining = input;

    loop {
        let (rest, cmd) = parse_pipeline_command(remaining)?;
        commands.push(cmd);

        let rest = skip_whitespace(rest);
        if let Some(stripped) = rest.strip_prefix('|') {
            if !stripped.starts_with('|') {
                remaining = skip_whitespace(stripped);
            } else {
                remaining = rest;
                break;
            }
        } else {
            remaining = rest;
            break;
        }
    }

    Ok((remaining, Pipeline { commands, negated }))
}

/// Parse a single command in a pipeline.
fn parse_pipeline_command(input: &str) -> ParseResult<'_, PipelineCommand> {
    let input = skip_whitespace(input);

    // Check for subshell
    if input.starts_with('(') {
        let (remaining, cmd) = parse_subshell(input)?;
        return Ok((remaining, PipelineCommand { inner: cmd }));
    }

    // Parse simple command
    let (remaining, simple) = parse_simple_command(input)?;
    Ok((
        remaining,
        PipelineCommand {
            inner: Command::Simple(simple),
        },
    ))
}

/// Parse a subshell: ( commands )
fn parse_subshell(input: &str) -> ParseResult<'_, Command> {
    if !input.starts_with('(') {
        return Err(ParseError::Backtrace);
    }

    let inner = &input[1..];
    let (remaining, list) = parse_sequential_list(inner)?;
    let remaining = skip_whitespace(remaining);

    if !remaining.starts_with(')') {
        return Err(ParseError::Failure(ParseErrorFailure::new(
            remaining,
            "Expected ')' to close subshell",
        )));
    }

    Ok((&remaining[1..], Command::Subshell(Box::new(list))))
}

/// Parse a simple command.
fn parse_simple_command(input: &str) -> ParseResult<'_, SimpleCommand> {
    let mut env_vars = Vec::new();
    let mut args = Vec::new();
    let mut redirects = Vec::new();
    let mut remaining = skip_whitespace(input);

    // Parse leading env var assignments
    loop {
        let trimmed = skip_whitespace(remaining);
        if let Ok((rest, env)) = parse_env_assignment(trimmed) {
            env_vars.push(env);
            remaining = rest;
        } else {
            remaining = trimmed;
            break;
        }
    }

    // Parse command name and arguments, mixed with redirects
    loop {
        remaining = skip_whitespace(remaining);

        if remaining.is_empty() || is_command_terminator(remaining) {
            break;
        }

        // Try redirect first
        if let Ok((rest, redirect)) = parse_redirect(remaining) {
            redirects.push(redirect);
            remaining = rest;
            continue;
        }

        // Parse a word
        if let Ok((rest, word)) = parse_word(remaining) {
            if !word.is_empty() {
                args.push(word);
            }
            remaining = rest;
        } else {
            break;
        }
    }

    if args.is_empty() && env_vars.is_empty() {
        return Err(ParseError::Backtrace);
    }

    Ok((
        remaining,
        SimpleCommand {
            env_vars,
            args,
            redirects,
        },
    ))
}

/// Parse an environment variable assignment (VAR=value).
fn parse_env_assignment(input: &str) -> ParseResult<'_, EnvVar> {
    // First character must be a letter or underscore
    let first = input.chars().next().ok_or(ParseError::Backtrace)?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(ParseError::Backtrace);
    }

    // Find the =
    let eq_pos = input.find('=').ok_or(ParseError::Backtrace)?;

    let name = &input[..eq_pos];

    // Validate name
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(ParseError::Backtrace);
    }

    // Check that the next non-whitespace after name is =
    // and there's no space before =
    if name.ends_with(char::is_whitespace) {
        return Err(ParseError::Backtrace);
    }

    let value_start = &input[eq_pos + 1..];

    // Parse value as a word (or empty)
    let (remaining, value) = if value_start.is_empty()
        || value_start.starts_with(char::is_whitespace)
        || is_command_terminator(value_start)
    {
        (value_start, Word { parts: vec![] })
    } else {
        parse_word(value_start)?
    };

    Ok((
        remaining,
        EnvVar {
            name: name.to_string(),
            value,
        },
    ))
}

/// Parse a word (sequence of word parts).
fn parse_word(input: &str) -> ParseResult<'_, Word> {
    let mut parts = Vec::new();
    let mut remaining = input;

    loop {
        if remaining.is_empty()
            || remaining.starts_with(char::is_whitespace)
            || is_command_terminator(remaining)
        {
            break;
        }

        let first = remaining.chars().next().unwrap();

        match first {
            // Single quote
            '\'' => {
                let (rest, text) = parse_single_quoted(remaining)?;
                parts.push(WordPart::SingleQuoted(text));
                remaining = rest;
            }
            // Double quote
            '"' => {
                let (rest, inner_parts) = parse_double_quoted(remaining)?;
                parts.push(WordPart::DoubleQuoted(inner_parts));
                remaining = rest;
            }
            // Variable or special
            '$' => {
                let (rest, part) = parse_dollar(remaining)?;
                parts.push(part);
                remaining = rest;
            }
            // Tilde expansion
            '~' if parts.is_empty() => {
                let (rest, part) = parse_tilde(remaining)?;
                parts.push(part);
                remaining = rest;
            }
            // Backslash escape
            '\\' => {
                if remaining.len() > 1 {
                    let escaped = remaining.chars().nth(1).unwrap();
                    parts.push(WordPart::Text(escaped.to_string()));
                    remaining = &remaining[2..];
                } else {
                    break;
                }
            }
            // Glob patterns
            '*' => {
                if remaining.starts_with("**") {
                    parts.push(WordPart::Glob(GlobPart::DoubleStar));
                    remaining = &remaining[2..];
                } else {
                    parts.push(WordPart::Glob(GlobPart::Star));
                    remaining = &remaining[1..];
                }
            }
            '?' => {
                parts.push(WordPart::Glob(GlobPart::Question));
                remaining = &remaining[1..];
            }
            '[' => {
                let (rest, class) = parse_char_class(remaining)?;
                parts.push(WordPart::Glob(GlobPart::CharClass(class)));
                remaining = rest;
            }
            // Backtick command substitution
            '`' => {
                let (rest, cmd) = parse_backtick_substitution(remaining)?;
                parts.push(WordPart::CommandSubstitution(Box::new(cmd)));
                remaining = rest;
            }
            // Regular text
            _ => {
                let (rest, text) = parse_unquoted_text(remaining)?;
                if !text.is_empty() {
                    parts.push(WordPart::Text(text));
                }
                remaining = rest;
            }
        }
    }

    if parts.is_empty() {
        return Err(ParseError::Backtrace);
    }

    Ok((remaining, Word { parts }))
}

/// Parse single-quoted string.
fn parse_single_quoted(input: &str) -> ParseResult<'_, String> {
    if !input.starts_with('\'') {
        return Err(ParseError::Backtrace);
    }

    let content = &input[1..];
    let end = content.find('\'').ok_or_else(|| {
        ParseError::Failure(ParseErrorFailure::new(input, "Unterminated single quote"))
    })?;

    let text = &content[..end];
    let remaining = &content[end + 1..];

    Ok((remaining, text.to_string()))
}

/// Parse double-quoted string.
fn parse_double_quoted(input: &str) -> ParseResult<'_, Vec<WordPart>> {
    if !input.starts_with('"') {
        return Err(ParseError::Backtrace);
    }

    let mut parts = Vec::new();
    let mut remaining = &input[1..];
    let mut current_text = String::new();

    loop {
        if remaining.is_empty() {
            return Err(ParseError::Failure(ParseErrorFailure::new(
                input,
                "Unterminated double quote",
            )));
        }

        let first = remaining.chars().next().unwrap();

        match first {
            '"' => {
                if !current_text.is_empty() {
                    parts.push(WordPart::Text(current_text));
                }
                return Ok((&remaining[1..], parts));
            }
            '\\' => {
                if remaining.len() > 1 {
                    let escaped = remaining.chars().nth(1).unwrap();
                    // In double quotes, only certain escapes are special
                    match escaped {
                        '"' | '\\' | '$' | '`' | '\n' => {
                            current_text.push(escaped);
                            remaining = &remaining[2..];
                        }
                        _ => {
                            current_text.push('\\');
                            current_text.push(escaped);
                            remaining = &remaining[2..];
                        }
                    }
                } else {
                    current_text.push('\\');
                    remaining = &remaining[1..];
                }
            }
            '$' => {
                if !current_text.is_empty() {
                    parts.push(WordPart::Text(current_text.clone()));
                    current_text.clear();
                }
                let (rest, part) = parse_dollar(remaining)?;
                parts.push(part);
                remaining = rest;
            }
            '`' => {
                if !current_text.is_empty() {
                    parts.push(WordPart::Text(current_text.clone()));
                    current_text.clear();
                }
                let (rest, cmd) = parse_backtick_substitution(remaining)?;
                parts.push(WordPart::CommandSubstitution(Box::new(cmd)));
                remaining = rest;
            }
            _ => {
                current_text.push(first);
                remaining = &remaining[first.len_utf8()..];
            }
        }
    }
}

/// Parse $ prefixed constructs.
fn parse_dollar(input: &str) -> ParseResult<'_, WordPart> {
    if !input.starts_with('$') {
        return Err(ParseError::Backtrace);
    }

    let after_dollar = &input[1..];

    if after_dollar.is_empty() {
        return Ok((after_dollar, WordPart::Text("$".to_string())));
    }

    let first = after_dollar.chars().next().unwrap();

    match first {
        // ${VAR} - braced variable
        '{' => parse_braced_variable(after_dollar),
        // $(cmd) - command substitution
        '(' => {
            if after_dollar.starts_with("((") {
                // $((expr)) - arithmetic
                parse_arithmetic(after_dollar)
            } else {
                parse_command_substitution(after_dollar)
            }
        }
        // Simple variable
        _ if first.is_ascii_alphabetic() || first == '_' => {
            let end = after_dollar
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(after_dollar.len());
            let name = &after_dollar[..end];
            Ok((&after_dollar[end..], WordPart::Variable(name.to_string())))
        }
        // Special variables: $?, $!, $$, etc.
        '?' | '!' | '$' | '#' | '*' | '@' | '-' | '0'..='9' => {
            let name = first.to_string();
            Ok((&after_dollar[1..], WordPart::Variable(name)))
        }
        // Just a literal $
        _ => Ok((after_dollar, WordPart::Text("$".to_string()))),
    }
}

/// Parse ${VAR} or ${VAR:-default} style.
fn parse_braced_variable(input: &str) -> ParseResult<'_, WordPart> {
    if !input.starts_with('{') {
        return Err(ParseError::Backtrace);
    }

    let content = &input[1..];
    let end = find_matching_brace(content, '{', '}')?;
    let var_content = &content[..end];
    let remaining = &content[end + 1..];

    // For now, just extract the variable name (ignoring modifiers)
    let name = var_content
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .map(|pos| &var_content[..pos])
        .unwrap_or(var_content);

    Ok((remaining, WordPart::Variable(name.to_string())))
}

/// Parse $(cmd) command substitution.
fn parse_command_substitution(input: &str) -> ParseResult<'_, WordPart> {
    if !input.starts_with('(') {
        return Err(ParseError::Backtrace);
    }

    let content = &input[1..];
    let end = find_matching_brace(content, '(', ')')?;
    let cmd_content = &content[..end];
    let remaining = &content[end + 1..];

    let cmd = parse(cmd_content).map_err(|_| {
        ParseError::Failure(ParseErrorFailure::new(
            input,
            "Invalid command in substitution",
        ))
    })?;

    Ok((remaining, WordPart::CommandSubstitution(Box::new(cmd))))
}

/// Parse $((expr)) arithmetic expansion.
fn parse_arithmetic(input: &str) -> ParseResult<'_, WordPart> {
    if !input.starts_with("((") {
        return Err(ParseError::Backtrace);
    }

    let content = &input[2..];
    // Find matching ))
    let mut depth = 1;
    let mut end = 0;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '(' if chars.peek() == Some(&'(') => {
                chars.next();
                depth += 1;
                end += 2;
            }
            ')' if chars.peek() == Some(&')') => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                chars.next();
                end += 2;
            }
            _ => {
                end += c.len_utf8();
            }
        }
    }

    if depth != 0 {
        return Err(ParseError::Failure(ParseErrorFailure::new(
            input,
            "Unterminated arithmetic expression",
        )));
    }

    let expr = &content[..end];
    let remaining = &content[end + 2..]; // Skip ))

    Ok((remaining, WordPart::Arithmetic(expr.to_string())))
}

/// Parse `cmd` backtick command substitution.
fn parse_backtick_substitution(input: &str) -> ParseResult<'_, SequentialList> {
    if !input.starts_with('`') {
        return Err(ParseError::Backtrace);
    }

    let content = &input[1..];
    let end = content.find('`').ok_or_else(|| {
        ParseError::Failure(ParseErrorFailure::new(input, "Unterminated backtick"))
    })?;

    let cmd_content = &content[..end];
    let remaining = &content[end + 1..];

    let cmd = parse(cmd_content).map_err(|_| {
        ParseError::Failure(ParseErrorFailure::new(
            input,
            "Invalid command in backticks",
        ))
    })?;

    Ok((remaining, cmd))
}

/// Parse ~ tilde expansion.
fn parse_tilde(input: &str) -> ParseResult<'_, WordPart> {
    if !input.starts_with('~') {
        return Err(ParseError::Backtrace);
    }

    let after_tilde = &input[1..];

    // Find end of tilde path (until /, whitespace, or special char)
    let end = after_tilde
        .find(|c: char| c == '/' || c.is_whitespace() || is_special_char(c))
        .unwrap_or(after_tilde.len());

    let path = if end == 0 {
        None
    } else {
        Some(after_tilde[..end].to_string())
    };

    // Get remaining after tilde path (includes / if present)
    let remaining = &after_tilde[end..];

    Ok((remaining, WordPart::Tilde(path)))
}

/// Parse character class [...].
fn parse_char_class(input: &str) -> ParseResult<'_, String> {
    if !input.starts_with('[') {
        return Err(ParseError::Backtrace);
    }

    let content = &input[1..];
    let mut end = 0;
    let mut chars = content.chars();
    let mut found_end = false;

    // Handle [! or [^ at start
    if content.starts_with('!') || content.starts_with('^') {
        end += 1;
        chars.next();
    }

    // Handle ] as first character (literal)
    if content[end..].starts_with(']') {
        end += 1;
        chars.next();
    }

    for c in chars {
        end += c.len_utf8();
        if c == ']' {
            found_end = true;
            break;
        }
    }

    if !found_end {
        return Err(ParseError::Failure(ParseErrorFailure::new(
            input,
            "Unterminated character class",
        )));
    }

    let class = &content[..end - 1]; // Exclude closing ]
    let remaining = &content[end..];

    Ok((remaining, class.to_string()))
}

/// Parse unquoted text (until special character).
fn parse_unquoted_text(input: &str) -> ParseResult<'_, String> {
    let end = input
        .find(|c: char| {
            c.is_whitespace()
                || is_special_char(c)
                || c == '\''
                || c == '"'
                || c == '$'
                || c == '\\'
                || c == '`'
                || c == '*'
                || c == '?'
                || c == '['
        })
        .unwrap_or(input.len());

    if end == 0 {
        return Err(ParseError::Backtrace);
    }

    Ok((&input[end..], input[..end].to_string()))
}

/// Parse I/O redirection.
fn parse_redirect(input: &str) -> ParseResult<'_, Redirect> {
    let input = skip_whitespace(input);

    // Check for fd number prefix
    let (fd, remaining) = if input
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        let end = input
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(input.len());
        let fd_str = &input[..end];
        let fd = fd_str.parse().ok();
        (fd, &input[end..])
    } else {
        (None, input)
    };

    // Parse redirect operator (order matters - check longer patterns first)
    let (op, remaining) = if let Some(r) = remaining.strip_prefix("<<<") {
        (RedirectOp::HereString, r)
    } else if let Some(r) = remaining.strip_prefix(">>") {
        (RedirectOp::Append, r)
    } else if let Some(r) = remaining.strip_prefix(">&") {
        (RedirectOp::DupOutput, r)
    } else if let Some(r) = remaining.strip_prefix("&>") {
        (RedirectOp::OutputBoth, r)
    } else if let Some(r) = remaining.strip_prefix("<&") {
        (RedirectOp::DupInput, r)
    } else if let Some(r) = remaining.strip_prefix("<<") {
        (RedirectOp::HereDoc, r)
    } else if let Some(r) = remaining.strip_prefix('>') {
        (RedirectOp::Output, r)
    } else if let Some(r) = remaining.strip_prefix('<') {
        (RedirectOp::Input, r)
    } else {
        return Err(ParseError::Backtrace);
    };

    let remaining = skip_whitespace(remaining);

    // Parse target
    let (remaining, target) = match op {
        RedirectOp::DupOutput | RedirectOp::DupInput => {
            // Expect fd number or -
            if let Some(r) = remaining.strip_prefix('-') {
                (
                    r,
                    RedirectTarget::Fd(u32::MAX), // Special value for close
                )
            } else if remaining
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                let end = remaining
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(remaining.len());
                let fd: u32 = remaining[..end]
                    .parse()
                    .map_err(|_| ParseError::Backtrace)?;
                (&remaining[end..], RedirectTarget::Fd(fd))
            } else {
                // Fall back to file
                let (rest, word) = parse_word(remaining)?;
                (rest, RedirectTarget::File(word))
            }
        }
        RedirectOp::HereDoc => {
            // TODO: Proper here-doc parsing
            let (rest, word) = parse_word(remaining)?;
            (rest, RedirectTarget::File(word))
        }
        RedirectOp::HereString => {
            let (rest, word) = parse_word(remaining)?;
            (rest, RedirectTarget::File(word))
        }
        _ => {
            let (rest, word) = parse_word(remaining)?;
            (rest, RedirectTarget::File(word))
        }
    };

    Ok((remaining, Redirect { fd, op, target }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Skip whitespace (but not newlines in some contexts).
fn skip_whitespace(input: &str) -> &str {
    input.trim_start_matches([' ', '\t'])
}

/// Check if char is a special shell character.
fn is_special_char(c: char) -> bool {
    matches!(c, '|' | '&' | ';' | '(' | ')' | '<' | '>' | '\n' | '#')
}

/// Check if we're at a command terminator.
fn is_command_terminator(input: &str) -> bool {
    input.is_empty()
        || input.starts_with(';')
        || input.starts_with('\n')
        || input.starts_with("&&")
        || input.starts_with("||")
        || input.starts_with('|')
        || input.starts_with(')')
        || input.starts_with('}')
        || input.starts_with('#')
}

/// Find matching brace, accounting for nesting.
fn find_matching_brace(input: &str, open: char, close: char) -> Result<usize, ParseError<'_>> {
    let mut depth = 1;
    let mut pos = 0;
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        // Handle quotes
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
        } else if c == '\\' && !in_single_quote {
            // Skip escaped char
            if chars.next().is_some() {
                pos += 1;
            }
        } else if !in_single_quote && !in_double_quote {
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    return Ok(pos);
                }
            }
        }
        pos += c.len_utf8();
    }

    Err(ParseError::Failure(ParseErrorFailure::new(
        input,
        format!("Unterminated '{}' - expected '{}'", open, close),
    )))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let result = parse("echo hello").unwrap();
        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn test_pipeline() {
        let result = parse("cat file | grep pattern").unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].current.commands.len(), 2);
    }

    #[test]
    fn test_sequence_and() {
        let result = parse("cmd1 && cmd2").unwrap();
        assert!(result.items[0].next.is_some());
        assert_eq!(
            result.items[0].next.as_ref().unwrap().op,
            BooleanListOp::And
        );
    }

    #[test]
    fn test_sequence_or() {
        let result = parse("cmd1 || cmd2").unwrap();
        assert!(result.items[0].next.is_some());
        assert_eq!(result.items[0].next.as_ref().unwrap().op, BooleanListOp::Or);
    }

    #[test]
    fn test_single_quoted() {
        let result = parse("echo 'hello world'").unwrap();
        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn test_double_quoted() {
        let result = parse("echo \"hello $USER\"").unwrap();
        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn test_variable_expansion() {
        let result = parse("echo $HOME").unwrap();
        assert_eq!(result.items.len(), 1);
        let cmd = match &result.items[0].current.commands[0].inner {
            Command::Simple(s) => s,
            _ => panic!("Expected simple command"),
        };
        assert_eq!(cmd.args.len(), 2);
    }

    #[test]
    fn test_redirect_output() {
        let result = parse("echo hello > file.txt").unwrap();
        let cmd = match &result.items[0].current.commands[0].inner {
            Command::Simple(s) => s,
            _ => panic!("Expected simple command"),
        };
        assert_eq!(cmd.redirects.len(), 1);
        assert_eq!(cmd.redirects[0].op, RedirectOp::Output);
    }

    #[test]
    fn test_subshell() {
        let result = parse("(echo hello)").unwrap();
        assert!(matches!(
            &result.items[0].current.commands[0].inner,
            Command::Subshell(_)
        ));
    }

    #[test]
    fn test_env_var_assignment() {
        let result = parse("FOO=bar echo $FOO").unwrap();
        let cmd = match &result.items[0].current.commands[0].inner {
            Command::Simple(s) => s,
            _ => panic!("Expected simple command"),
        };
        assert_eq!(cmd.env_vars.len(), 1);
        assert_eq!(cmd.env_vars[0].name, "FOO");
    }

    #[test]
    fn test_empty_input() {
        let result = parse("").unwrap();
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_multiple_commands() {
        let result = parse("cmd1; cmd2; cmd3").unwrap();
        assert_eq!(result.items.len(), 3);
    }

    #[test]
    fn test_glob_patterns() {
        let result = parse("ls *.rs").unwrap();
        let cmd = match &result.items[0].current.commands[0].inner {
            Command::Simple(s) => s,
            _ => panic!("Expected simple command"),
        };
        assert_eq!(cmd.args.len(), 2);
    }
}
