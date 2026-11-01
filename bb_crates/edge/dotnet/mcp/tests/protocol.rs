//! Stdio protocol checks against small scripted perl servers (no dotnet, no network).

use std::ffi::OsString;
use std::time::{Duration, Instant};

use bb_mcp::{McpError, Session};

const INIT_REPLY: &str = r#"sub init_reply { my ($id) = @_; print qq({"jsonrpc":"2.0","id":$id,"result":{"protocolVersion":"2025-03-26","capabilities":{},"serverInfo":{"name":"p","version":"1"}}}\n); }
sub pong { my ($line) = @_; my ($id) = $line =~ /"id":(\d+)/; print qq({"jsonrpc":"2.0","id":$id,"result":{}}\n); }
$| = 1;
"#;

const PING_DURING_INITIALIZE: &str = r#"
my $init = <STDIN>;
print qq({"jsonrpc":"2.0","id":"srv-1","method":"ping"}\n);
my $reply = <STDIN>;
exit 7 unless $reply =~ /"id":"srv-1"/ && $reply =~ /"result":\{\}/;
my ($id) = $init =~ /"id":(\d+)/;
init_reply($id);
while (my $line = <STDIN>) { pong($line) if $line =~ /"method":"ping"/; }
"#;

const REQUEST_WHILE_IDLE: &str = r#"
my $init = <STDIN>;
my ($id) = $init =~ /"id":(\d+)/;
init_reply($id);
my $initialized = <STDIN>;
print qq({"jsonrpc":"2.0","id":"srv-2","method":"roots/list"}\n);
my $reply = <STDIN>;
exit 9 unless $reply =~ /"id":"srv-2"/ && $reply =~ /"code":-32601/;
while (my $line = <STDIN>) { pong($line) if $line =~ /"method":"ping"/; }
"#;

const GARBAGE_ON_STDOUT: &str = r#"
my $init = <STDIN>;
print "Hello from a server that logs to stdout\n";
sleep 5;
"#;

const EXIT_ON_EOF: &str = r#"
my $init = <STDIN>;
my ($id) = $init =~ /"id":(\d+)/;
init_reply($id);
while (my $line = <STDIN>) { pong($line) if $line =~ /"method":"ping"/; }
exit 0;
"#;

async fn perl(body: &str) -> Result<Session, McpError> {
    let script: String = format!("{INIT_REPLY}{body}");
    let args: Vec<OsString> = vec!["-e".into(), script.into()];
    Session::launch("perl", &args).await
}

#[tokio::test]
async fn server_ping_during_initialize_is_answered_and_handshake_completes() {
    let mut session: Session = perl(PING_DURING_INITIALIZE).await.unwrap();
    session.ping().await.unwrap();
    assert!(session.stop().await.unwrap().success());
}

#[tokio::test]
async fn server_request_while_idle_gets_method_not_found_before_next_client_request() {
    let mut session: Session = perl(REQUEST_WHILE_IDLE).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    session.ping().await.unwrap();
    assert!(session.stop().await.unwrap().success());
}

#[tokio::test]
async fn non_json_stdout_error_names_the_channel_and_first_bytes() {
    let error: McpError = perl(GARBAGE_ON_STDOUT).await.err().unwrap();
    let text: String = error.to_string();
    assert!(text.contains("non-JSON"), "{text}");
    assert!(text.contains("Hello from a server"), "{text}");
}

#[tokio::test]
async fn missing_program_error_names_the_program() {
    let error: McpError = Session::launch("bb-no-such-server", &[])
        .await
        .err()
        .unwrap();
    let text: String = error.to_string();
    assert!(text.contains("bb-no-such-server"), "{text}");
    assert!(text.contains("not found"), "{text}");
}

#[tokio::test]
async fn stop_closes_stdin_so_an_eof_server_exits_before_the_fallback() {
    let session: Session = perl(EXIT_ON_EOF).await.unwrap();
    let started: Instant = Instant::now();
    let status: std::process::ExitStatus = session.stop().await.unwrap();
    let elapsed: Duration = started.elapsed();
    assert!(status.success(), "{status}");
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}
