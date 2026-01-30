//! Ledger API client — connects to Canton/Daml participant and exposes Ledger API v2 services.
//! Only compiled when proto files are present (see proto/README.md).

use canton_core::{error::*, types::LedgerOffset};
use tonic::transport::Channel;
use tonic::Status;

use crate::generated::com::daml::ledger::api::v2::{
    command_submission_service_client::CommandSubmissionServiceClient,
    state_service_client::StateServiceClient,
    GetLedgerEndRequest, SubmitRequest,
};

/// Ledger API v2 client. Holds gRPC channel and service stubs.
#[derive(Clone)]
pub struct LedgerClient {
    #[allow(dead_code)] // kept for future service clients and Clone
    channel: Channel,
    ledger_id: String,
    state: StateServiceClient<Channel>,
    command_submission: CommandSubmissionServiceClient<Channel>,
}

impl LedgerClient {
    /// Connect to a participant and create a Ledger API client.
    /// `endpoint`: e.g. `"http://localhost:5011"` or `"https://participant.example.com"`.
    /// `ledger_id`: ledger identifier (e.g. from participant config or VersionService).
    pub async fn connect(
        endpoint: impl AsRef<str>,
        ledger_id: impl Into<String>,
    ) -> SdkResult<Self> {
        let endpoint = endpoint.as_ref();
        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| SdkError::Config(format!("invalid endpoint {:?}: {}", endpoint, e)))?
            .connect()
            .await
            .map_err(|e| SdkError::Connection {
                message: format!("failed to connect to {}: {}", endpoint, e),
                cause: Some(Box::new(e)),
                backtrace: std::backtrace::Backtrace::capture(),
            })?;
        let ledger_id = ledger_id.into();
        let state = StateServiceClient::new(channel.clone());
        let command_submission = CommandSubmissionServiceClient::new(channel.clone());
        Ok(Self {
            channel,
            ledger_id,
            state,
            command_submission,
        })
    }

    /// Ledger identifier for this connection.
    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }

    /// Get the current ledger end offset. Subscriptions started with this offset
    /// will receive events after this call.
    pub async fn get_ledger_end(&mut self) -> SdkResult<LedgerOffset> {
        let request = GetLedgerEndRequest {};
        let response = self
            .state
            .get_ledger_end(request)
            .await
            .map_err(grpc_status_to_sdk_error)?;
        let offset = response.into_inner().offset;
        Ok(LedgerOffset::absolute(offset.to_string()))
    }

    /// Submit commands to the ledger. Uses proto Commands (built from canton_core::types::Commands
    /// via a conversion layer when needed).
    pub async fn submit(&mut self, commands: crate::generated::com::daml::ledger::api::v2::Commands) -> SdkResult<()> {
        let request = SubmitRequest {
            commands: Some(commands),
        };
        self.command_submission
            .submit(request)
            .await
            .map_err(grpc_status_to_sdk_error)?;
        Ok(())
    }
}

fn grpc_status_to_sdk_error(status: Status) -> SdkError {
    let message = status.message().to_string();
    let code = status.code();
    if code == tonic::Code::Unavailable {
        SdkError::Connection {
            message,
            cause: None,
            backtrace: std::backtrace::Backtrace::capture(),
        }
    } else if code == tonic::Code::DeadlineExceeded {
        SdkError::Timeout {
            duration: std::time::Duration::from_secs(0),
            operation: message,
        }
    } else {
        SdkError::Internal {
            message: format!("grpc {}: {}", code, message),
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canton_core::types::OffsetValue;

    #[test]
    fn grpc_unavailable_maps_to_connection_error() {
        let status = Status::unavailable("gone");
        let err = grpc_status_to_sdk_error(status);
        assert!(matches!(err, SdkError::Connection { .. }));
    }

    #[test]
    fn grpc_deadline_exceeded_maps_to_timeout() {
        let status = Status::deadline_exceeded("took too long");
        let err = grpc_status_to_sdk_error(status);
        assert!(matches!(err, SdkError::Timeout { .. }));
    }

    #[test]
    fn ledger_offset_absolute_from_get_ledger_end_response() {
        let offset = LedgerOffset::absolute(42_i64.to_string());
        assert!(matches!(offset.value, OffsetValue::Absolute(ref s) if s == "42"));
    }
}
