//! Shell execution engine
//!
//! Executes parsed shell commands using the ShellState and command registry.

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use futures::future::LocalBoxFuture;

use crate::parser::{
    BooleanListOp, Command, GlobPart, Pipeline, PipelineCommand, Redirect, RedirectOp,
    RedirectTarget, Sequence, SequentialList, SimpleCommand, Word, WordPart,
};
use crate::shell::commands::{
    resolve_command, ExecutableCommand, ShellCommand, ShellCommandContext,
};
use crate::shell::types::{
    pipe, EnvChange, ExecuteResult, ShellPipeReader, ShellPipeWriter, ShellState,
};

// ============================================================================
// Public API
// ============================================================================

/// Execute a parsed command list.
pub async fn execute(list: SequentialList, state: Rc<ShellState>) -> i32 {
    execute_sequential_list(list, state).await
}

/// Execute a command string.
pub async fn execute_str(command: &str, state: Rc<ShellState>) -> Result<i32, String> {
    let list = crate::parser::parse(command).map_err(|e| format!("Parse error: {:?}", e))?;
    Ok(execute(list, state).await)
}

/// Execute with custom stdin/stdout/stderr.
pub async fn execute_with_pipes(
    list: SequentialList,
    state: Rc<ShellState>,
    stdin: ShellPipeReader,
    stdout: ShellPipeWriter,
    stderr: ShellPipeWriter,
) -> i32 {
    execute_sequential_list_with_pipes(list, state, stdin, stdout, stderr).await
}

// ============================================================================
// Sequential List Execution
// ============================================================================

async fn execute_sequential_list(list: SequentialList, state: Rc<ShellState>) -> i32 {
    execute_sequential_list_with_pipes(
        list,
        state,
        ShellPipeReader::stdin(),
        ShellPipeWriter::stdout(),
        ShellPipeWriter::stderr(),
    )
    .await
}

async fn execute_sequential_list_with_pipes(
    list: SequentialList,
    state: Rc<ShellState>,
    stdin: ShellPipeReader,
    stdout: ShellPipeWriter,
    stderr: ShellPipeWriter,
) -> i32 {
    let mut last_exit_code = 0;

    for sequence in list.items {
        let result = execute_sequence(
            sequence,
            state.clone(),
            stdin.clone(),
            stdout.clone(),
            stderr.clone(),
        )
        .await;

        last_exit_code = result.exit_code();

        // Apply env changes to state
        if let ExecuteResult::Continue(_, changes, _) = result {
            apply_env_changes(&state, changes);
        }
    }

    last_exit_code
}

// ============================================================================
// Sequence Execution (handles && and ||)
// ============================================================================

fn execute_sequence<'a>(
    sequence: Sequence,
    state: Rc<ShellState>,
    stdin: ShellPipeReader,
    stdout: ShellPipeWriter,
    stderr: ShellPipeWriter,
) -> LocalBoxFuture<'a, ExecuteResult> {
    Box::pin(async move {
        // Execute the current pipeline
        let result = execute_pipeline(
            sequence.current,
            state.clone(),
            stdin.clone(),
            stdout.clone(),
            stderr.clone(),
        )
        .await;

        let exit_code = result.exit_code();

        // Apply any env changes from this pipeline
        let mut all_changes = Vec::new();
        if let ExecuteResult::Continue(_, changes, _) = &result {
            all_changes.extend(changes.clone());
            apply_env_changes(&state, changes.clone());
        }

        // Handle the next part of the sequence
        if let Some(next) = sequence.next {
            let should_continue = match next.op {
                BooleanListOp::And => exit_code == 0,
                BooleanListOp::Or => exit_code != 0,
            };

            if should_continue {
                let next_result =
                    execute_sequence(next.sequence, state, stdin, stdout, stderr).await;

                if let ExecuteResult::Continue(code, changes, handles) = next_result {
                    all_changes.extend(changes);
                    return ExecuteResult::Continue(code, all_changes, handles);
                }

                return next_result;
            }
        }

        // Return with accumulated changes
        match result {
            ExecuteResult::Exit(code, handles) => ExecuteResult::Exit(code, handles),
            ExecuteResult::Continue(code, _, handles) => {
                ExecuteResult::Continue(code, all_changes, handles)
            }
        }
    })
}

// ============================================================================
// Pipeline Execution (handles |)
// ============================================================================

async fn execute_pipeline(
    pipeline: Pipeline,
    state: Rc<ShellState>,
    stdin: ShellPipeReader,
    stdout: ShellPipeWriter,
    stderr: ShellPipeWriter,
) -> ExecuteResult {
    if pipeline.commands.is_empty() {
        return ExecuteResult::Continue(0, vec![], vec![]);
    }

    // Single command (no pipes)
    if pipeline.commands.len() == 1 {
        let result = execute_pipeline_command(
            pipeline.commands.into_iter().next().unwrap(),
            state,
            stdin,
            stdout,
            stderr,
        )
        .await;

        // Handle negation
        if pipeline.negated {
            let code = if result.exit_code() == 0 { 1 } else { 0 };
            return match result {
                ExecuteResult::Exit(_, handles) => ExecuteResult::Exit(code, handles),
                ExecuteResult::Continue(_, changes, handles) => {
                    ExecuteResult::Continue(code, changes, handles)
                }
            };
        }

        return result;
    }

    // Multiple commands connected by pipes
    let mut commands = pipeline.commands.into_iter();
    let mut prev_stdout = stdin;
    let mut handles = Vec::new();

    // Execute all but the last command
    let last_cmd = commands.next_back().unwrap();

    for cmd in commands {
        let (reader, writer) = pipe();

        let result =
            execute_pipeline_command(cmd, state.clone(), prev_stdout, writer, stderr.clone()).await;

        if let ExecuteResult::Exit(code, h) = result {
            handles.extend(h);
            return ExecuteResult::Exit(code, handles);
        }

        if let ExecuteResult::Continue(_, _, h) = result {
            handles.extend(h);
        }

        prev_stdout = reader;
    }

    // Execute the last command with the original stdout
    let result = execute_pipeline_command(last_cmd, state, prev_stdout, stdout, stderr).await;

    // Handle negation
    let exit_code = if pipeline.negated {
        if result.exit_code() == 0 {
            1
        } else {
            0
        }
    } else {
        result.exit_code()
    };

    match result {
        ExecuteResult::Exit(_, h) => {
            handles.extend(h);
            ExecuteResult::Exit(exit_code, handles)
        }
        ExecuteResult::Continue(_, changes, h) => {
            handles.extend(h);
            ExecuteResult::Continue(exit_code, changes, handles)
        }
    }
}

// ============================================================================
// Pipeline Command Execution
// ============================================================================

async fn execute_pipeline_command(
    cmd: PipelineCommand,
    state: Rc<ShellState>,
    stdin: ShellPipeReader,
    stdout: ShellPipeWriter,
    stderr: ShellPipeWriter,
) -> ExecuteResult {
    match cmd.inner {
        Command::Simple(simple) => {
            execute_simple_command(simple, state, stdin, stdout, stderr).await
        }
        Command::Subshell(list) => {
            // Create a cloned state for subshell
            let sub_state = Rc::new(state.clone_for_subshell());
            let code =
                execute_sequential_list_with_pipes(*list, sub_state, stdin, stdout, stderr).await;
            ExecuteResult::Continue(code, vec![], vec![])
        }
        Command::If(if_clause) => {
            // Execute condition
            let cond_state = Rc::new(state.clone_for_subshell());
            let cond_code = execute_sequential_list(*if_clause.condition, cond_state).await;

            if cond_code == 0 {
                // Execute then part
                let code = execute_sequential_list_with_pipes(
                    *if_clause.then_part,
                    state,
                    stdin,
                    stdout,
                    stderr,
                )
                .await;
                ExecuteResult::Continue(code, vec![], vec![])
            } else if let Some(else_part) = if_clause.else_part {
                let code =
                    execute_sequential_list_with_pipes(*else_part, state, stdin, stdout, stderr)
                        .await;
                ExecuteResult::Continue(code, vec![], vec![])
            } else {
                ExecuteResult::Continue(0, vec![], vec![])
            }
        }
    }
}

// ============================================================================
// Simple Command Execution
// ============================================================================

async fn execute_simple_command(
    cmd: SimpleCommand,
    state: Rc<ShellState>,
    stdin: ShellPipeReader,
    stdout: ShellPipeWriter,
    stderr: ShellPipeWriter,
) -> ExecuteResult {
    // Handle variable-only commands (no command name, just assignments)
    if cmd.args.is_empty() {
        let changes: Vec<EnvChange> = cmd
            .env_vars
            .iter()
            .map(|e| {
                EnvChange::SetEnvVar(e.name.clone().into(), expand_word(&e.value, &state).into())
            })
            .collect();
        return ExecuteResult::Continue(0, changes, vec![]);
    }

    // Expand all arguments
    let mut expanded_args: Vec<OsString> = Vec::new();
    for arg in &cmd.args {
        let expanded = expand_word_to_args(arg, &state);
        expanded_args.extend(expanded);
    }

    if expanded_args.is_empty() {
        return ExecuteResult::Continue(0, vec![], vec![]);
    }

    // Get command name
    let cmd_name = expanded_args[0].clone();

    // Set up redirections
    let (final_stdin, final_stdout, final_stderr) =
        apply_redirects(stdin, stdout, stderr, &cmd.redirects, &state);

    // Create state with temporary env vars for this command
    let cmd_state = if cmd.env_vars.is_empty() {
        (*state).clone()
    } else {
        let cloned = (*state).clone();
        for env_var in &cmd.env_vars {
            let value = expand_word(&env_var.value, &state);
            cloned.apply_env_var(
                std::ffi::OsStr::new(&env_var.name),
                std::ffi::OsStr::new(&value),
            );
        }
        cloned
    };

    // Look up the command as a builtin first
    if let Some(builtin) = state.resolve_custom_command(&cmd_name) {
        // Execute builtin command
        let context = ShellCommandContext {
            args: expanded_args,
            state: cmd_state,
            stdin: final_stdin,
            stdout: final_stdout,
            stderr: final_stderr,
            execute_command_args: Box::new(execute_command_args),
        };

        return builtin.execute(context).await;
    }

    // Try to find external command
    if let Some(path) = resolve_command(cmd_name.as_os_str(), &state) {
        let executable = ExecutableCommand::new(path);

        let context = ShellCommandContext {
            args: expanded_args,
            state: cmd_state,
            stdin: final_stdin,
            stdout: final_stdout,
            stderr: final_stderr,
            execute_command_args: Box::new(execute_command_args),
        };

        return executable.execute(context).await;
    }

    // Command not found
    let _ = writeln!(
        std::io::stderr(),
        "{}: command not found",
        cmd_name.to_string_lossy()
    );
    ExecuteResult::Continue(127, vec![], vec![])
}

/// Callback for nested command execution (used by xargs, etc.)
fn execute_command_args(context: ShellCommandContext) -> LocalBoxFuture<'static, ExecuteResult> {
    Box::pin(async move {
        // Build a simple command from args
        let args: Vec<Word> = context
            .args
            .iter()
            .map(|a| Word::from_string(a.to_string_lossy().to_string()))
            .collect();

        let cmd = SimpleCommand {
            env_vars: vec![],
            args,
            redirects: vec![],
        };

        let state = Rc::new(context.state);
        execute_simple_command(cmd, state, context.stdin, context.stdout, context.stderr).await
    })
}

// ============================================================================
// Word Expansion
// ============================================================================

/// Expand a word to a string (single result).
fn expand_word(word: &Word, state: &ShellState) -> String {
    let mut result = String::new();

    for part in &word.parts {
        result.push_str(&expand_word_part(part, state));
    }

    result
}

/// Expand a word to multiple arguments (for glob expansion).
fn expand_word_to_args(word: &Word, state: &ShellState) -> Vec<OsString> {
    let mut has_glob = false;
    let mut pattern = String::new();

    for part in &word.parts {
        match part {
            WordPart::Glob(_) => {
                has_glob = true;
                pattern.push_str(&expand_word_part(part, state));
            }
            _ => {
                pattern.push_str(&expand_word_part(part, state));
            }
        }
    }

    if has_glob {
        // Try glob expansion
        match glob::glob(&pattern) {
            Ok(paths) => {
                let matches: Vec<OsString> = paths
                    .filter_map(|p| p.ok())
                    .map(|p| p.into_os_string())
                    .collect();

                if matches.is_empty() {
                    // No matches, return pattern as-is
                    vec![OsString::from(pattern)]
                } else {
                    matches
                }
            }
            Err(_) => vec![OsString::from(pattern)],
        }
    } else {
        vec![OsString::from(pattern)]
    }
}

/// Expand a single word part.
fn expand_word_part(part: &WordPart, state: &ShellState) -> String {
    match part {
        WordPart::Text(s) => s.clone(),
        WordPart::SingleQuoted(s) => s.clone(),
        WordPart::DoubleQuoted(parts) => {
            let mut result = String::new();
            for p in parts {
                result.push_str(&expand_word_part(p, state));
            }
            result
        }
        WordPart::Variable(name) => {
            // Handle special variables
            match name.as_str() {
                "?" => state.last_exit_code().to_string(),
                "$" => std::process::id().to_string(),
                "HOME" => state
                    .home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                "PWD" => state.cwd().to_string_lossy().to_string(),
                "OLDPWD" => state.get_env_var("OLDPWD").unwrap_or_default(),
                _ => state.get_var_str(name).unwrap_or_default(),
            }
        }
        WordPart::Tilde(suffix) => {
            let home = state
                .home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "~".to_string());
            match suffix {
                None => home,
                Some(s) => format!("{}/{}", home, s),
            }
        }
        WordPart::Glob(glob) => match glob {
            GlobPart::Star => "*".to_string(),
            GlobPart::Question => "?".to_string(),
            GlobPart::DoubleStar => "**".to_string(),
            GlobPart::CharClass(class) => format!("[{}]", class),
        },
        WordPart::CommandSubstitution(list) => {
            // Execute command substitution and capture output
            // For complex substitutions, we serialize the AST and run through sh
            // For simple cases, we can execute directly
            execute_command_substitution(list, state)
        }
        WordPart::Arithmetic(expr) => {
            // Simple arithmetic evaluation
            evaluate_arithmetic(expr, state).unwrap_or_else(|| "0".to_string())
        }
    }
}

/// Simple arithmetic evaluation.
fn evaluate_arithmetic(expr: &str, state: &ShellState) -> Option<String> {
    // Very basic: just handle simple integer expressions
    let expr = expr.trim();

    // Handle variable references
    let expanded = if let Some(var_name) = expr.strip_prefix('$') {
        state.get_var_str(var_name).unwrap_or_default()
    } else {
        expr.to_string()
    };

    // Try to parse as number
    expanded.parse::<i64>().ok().map(|n| n.to_string())
}

/// Execute a command substitution and return its output.
///
/// This runs the command synchronously and captures stdout.
/// The output is trimmed of trailing newlines (standard shell behavior).
fn execute_command_substitution(list: &SequentialList, state: &ShellState) -> String {
    // Serialize the command to a string for shell execution
    let command_str = serialize_sequential_list(list);

    // Use the system shell to execute (handles complex cases like pipes)
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command_str)
        .current_dir(state.cwd())
        .envs(state.env_vars().iter().map(|(k, v)| {
            (
                k.to_string_lossy().to_string(),
                v.to_string_lossy().to_string(),
            )
        }))
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Shell convention: strip trailing newlines
            stdout.trim_end_matches('\n').to_string()
        }
        Err(_) => String::new(),
    }
}

/// Serialize a SequentialList back to shell command string.
fn serialize_sequential_list(list: &SequentialList) -> String {
    list.items
        .iter()
        .map(serialize_sequence)
        .collect::<Vec<_>>()
        .join("; ")
}

fn serialize_sequence(seq: &Sequence) -> String {
    let mut result = serialize_pipeline(&seq.current);
    if let Some(next) = &seq.next {
        let op_str = match next.op {
            BooleanListOp::And => " && ",
            BooleanListOp::Or => " || ",
        };
        result.push_str(op_str);
        result.push_str(&serialize_sequence(&next.sequence));
    }
    result
}

fn serialize_pipeline(pipeline: &Pipeline) -> String {
    pipeline
        .commands
        .iter()
        .map(serialize_pipeline_command)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn serialize_pipeline_command(cmd: &PipelineCommand) -> String {
    match &cmd.inner {
        Command::Simple(simple) => serialize_simple_command(simple),
        Command::Subshell(list) => format!("({})", serialize_sequential_list(list)),
        Command::If(if_clause) => serialize_if_clause(if_clause),
    }
}

fn serialize_if_clause(if_clause: &crate::parser::IfClause) -> String {
    let mut result = format!(
        "if {}; then {}",
        serialize_sequential_list(&if_clause.condition),
        serialize_sequential_list(&if_clause.then_part)
    );
    if let Some(else_part) = &if_clause.else_part {
        result.push_str(&format!("; else {}", serialize_sequential_list(else_part)));
    }
    result.push_str("; fi");
    result
}

fn serialize_simple_command(cmd: &SimpleCommand) -> String {
    let mut parts = Vec::new();

    // Add env vars
    for env in &cmd.env_vars {
        parts.push(format!("{}={}", env.name, serialize_word(&env.value)));
    }

    // Add args
    for arg in &cmd.args {
        parts.push(serialize_word(arg));
    }

    // Add redirects
    for redirect in &cmd.redirects {
        parts.push(serialize_redirect(redirect));
    }

    parts.join(" ")
}

fn serialize_word(word: &Word) -> String {
    let mut result = String::new();
    for part in &word.parts {
        result.push_str(&serialize_word_part(part));
    }
    result
}

fn serialize_word_part(part: &WordPart) -> String {
    match part {
        WordPart::Text(s) => s.clone(),
        WordPart::SingleQuoted(s) => format!("'{}'", s),
        WordPart::DoubleQuoted(parts) => {
            let inner: String = parts.iter().map(serialize_word_part).collect();
            format!("\"{}\"", inner)
        }
        WordPart::Variable(name) => format!("${}", name),
        WordPart::Tilde(suffix) => match suffix {
            Some(s) => format!("~{}", s),
            None => "~".to_string(),
        },
        WordPart::Glob(glob) => match glob {
            GlobPart::Star => "*".to_string(),
            GlobPart::Question => "?".to_string(),
            GlobPart::DoubleStar => "**".to_string(),
            GlobPart::CharClass(class) => format!("[{}]", class),
        },
        WordPart::CommandSubstitution(list) => {
            format!("$({})", serialize_sequential_list(list))
        }
        WordPart::Arithmetic(expr) => format!("$(({}))", expr),
    }
}

fn serialize_redirect(redirect: &Redirect) -> String {
    let op_str = match redirect.op {
        RedirectOp::Input => "<",
        RedirectOp::Output => ">",
        RedirectOp::Append => ">>",
        RedirectOp::HereDoc => "<<",
        RedirectOp::HereString => "<<<",
        RedirectOp::OutputBoth => "&>",
        RedirectOp::DupOutput => ">&",
        RedirectOp::DupInput => "<&",
    };

    let target_str = match &redirect.target {
        RedirectTarget::File(word) => serialize_word(word),
        RedirectTarget::Fd(fd) => format!("{}", fd),
        RedirectTarget::HereDoc(content) => content.clone(),
    };

    if let Some(fd) = redirect.fd {
        format!("{}{}{}", fd, op_str, target_str)
    } else {
        format!("{}{}", op_str, target_str)
    }
}

// ============================================================================
// Redirection Handling
// ============================================================================

fn apply_redirects(
    stdin: ShellPipeReader,
    stdout: ShellPipeWriter,
    stderr: ShellPipeWriter,
    redirects: &[Redirect],
    state: &ShellState,
) -> (ShellPipeReader, ShellPipeWriter, ShellPipeWriter) {
    let mut final_stdin = stdin;
    let mut final_stdout = stdout;
    let mut final_stderr = stderr;

    for redirect in redirects {
        match redirect.op {
            RedirectOp::Input => {
                if let RedirectTarget::File(word) = &redirect.target {
                    let path = expand_word(word, state);
                    let full_path = if PathBuf::from(&path).is_absolute() {
                        PathBuf::from(&path)
                    } else {
                        state.cwd().join(&path)
                    };

                    if let Ok(file) = File::open(&full_path) {
                        final_stdin = ShellPipeReader::from_file(file);
                    }
                }
            }
            RedirectOp::Output => {
                let fd = redirect.fd.unwrap_or(1);
                if let RedirectTarget::File(word) = &redirect.target {
                    let path = expand_word(word, state);
                    let full_path = if PathBuf::from(&path).is_absolute() {
                        PathBuf::from(&path)
                    } else {
                        state.cwd().join(&path)
                    };

                    if let Ok(file) = File::create(&full_path) {
                        if fd == 1 {
                            final_stdout = ShellPipeWriter::from_file(file);
                        } else if fd == 2 {
                            final_stderr = ShellPipeWriter::from_file(file);
                        }
                    }
                }
            }
            RedirectOp::Append => {
                let fd = redirect.fd.unwrap_or(1);
                if let RedirectTarget::File(word) = &redirect.target {
                    let path = expand_word(word, state);
                    let full_path = if PathBuf::from(&path).is_absolute() {
                        PathBuf::from(&path)
                    } else {
                        state.cwd().join(&path)
                    };

                    if let Ok(file) = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&full_path)
                    {
                        if fd == 1 {
                            final_stdout = ShellPipeWriter::from_file(file);
                        } else if fd == 2 {
                            final_stderr = ShellPipeWriter::from_file(file);
                        }
                    }
                }
            }
            RedirectOp::OutputBoth => {
                if let RedirectTarget::File(word) = &redirect.target {
                    let path = expand_word(word, state);
                    let full_path = if PathBuf::from(&path).is_absolute() {
                        PathBuf::from(&path)
                    } else {
                        state.cwd().join(&path)
                    };

                    if let Ok(file) = File::create(&full_path) {
                        final_stdout = ShellPipeWriter::from_file(file.try_clone().unwrap_or(file));
                        // Note: Can't easily clone to stderr too in this model
                    }
                }
            }
            RedirectOp::DupOutput => {
                // 2>&1 style
                if let RedirectTarget::Fd(fd) = redirect.target {
                    let source_fd = redirect.fd.unwrap_or(1);
                    if source_fd == 2 && fd == 1 {
                        // 2>&1: redirect stderr to stdout
                        final_stderr = final_stdout.clone();
                    } else if source_fd == 1 && fd == 2 {
                        // 1>&2: redirect stdout to stderr
                        final_stdout = final_stderr.clone();
                    }
                }
            }
            RedirectOp::DupInput => {
                // <& style - less common
            }
            RedirectOp::HereDoc | RedirectOp::HereString => {
                // Here documents
                if let RedirectTarget::File(word) = &redirect.target {
                    let content = expand_word(word, state);
                    final_stdin = ShellPipeReader::from_string(content);
                }
            }
        }
    }

    (final_stdin, final_stdout, final_stderr)
}

// ============================================================================
// Environment Changes
// ============================================================================

fn apply_env_changes(state: &ShellState, changes: Vec<EnvChange>) {
    state.apply_changes(&changes);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> Rc<ShellState> {
        Rc::new(ShellState::new_default())
    }

    #[tokio::test]
    async fn test_execute_simple() {
        let state = create_test_state();
        let result = execute_str("echo hello", state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_execute_sequence_and() {
        let state = create_test_state();
        // true && echo yes - should execute echo
        let result = execute_str("exit 0 && exit 5", state).await;
        assert!(result.is_ok());
        // Second command should run and return 5
    }

    #[tokio::test]
    async fn test_execute_sequence_or() {
        let state = create_test_state();
        // false || echo yes - should execute echo
        let result = execute_str("exit 1 || exit 0", state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_variable_expansion() {
        let state = create_test_state();
        state.set_env_var("FOO", "bar");
        let word = Word {
            parts: vec![WordPart::Variable("FOO".to_string())],
        };
        assert_eq!(expand_word(&word, &state), "bar");
    }
}
