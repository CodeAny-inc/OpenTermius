use crate::state::AppState;
use std::path::Path;
use std::sync::Arc;
use tauri::State;

type ApiResult<T> = std::result::Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Stream a remote SFTP file directly to the local path selected by the native
/// save dialog. File bytes never cross the frontend IPC boundary.
#[tauri::command]
pub async fn sftp_download_to_local(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    remote_path: String,
    local_path: String,
) -> ApiResult<()> {
    state
        .sftp
        .download_to_local(&session_id, &remote_path, Path::new(&local_path))
        .await
        .map_err(err)
}

/// Stream a local file to the remote SFTP server. Overwrite must be an explicit
/// user choice; otherwise an existing remote file is rejected by the core.
#[tauri::command]
pub async fn sftp_upload_from_local(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    local_path: String,
    remote_path: String,
    overwrite: bool,
) -> ApiResult<()> {
    state
        .sftp
        .upload_from_local(
            &session_id,
            Path::new(&local_path),
            &remote_path,
            overwrite,
        )
        .await
        .map_err(err)
}
