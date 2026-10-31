//! Readers for what the tools write: TRX, list output, settings.

mod bb_trx;
mod jsonc;
mod output;
mod settings;

pub use bb_trx::{parse_duration, parse_trx, TrxError};
pub use jsonc::strip_jsonc;
pub use output::{
    parse_apex_list, parse_fqn_lines, parse_host_pid, parse_mtp_list, ApexList, HostPid,
    OutputError,
};
pub use settings::{ApexSettings, CommandLine, ParseFailure, SettingsError, SETTINGS_FILE};
