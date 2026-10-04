use futures::TryStreamExt;
use reqwest::Response;
use rodio::Decoder;
use stream_download::{
    StreamDownload,
    async_read::AsyncReadStreamParams,
    storage::{StorageProvider, temp::TempStorageProvider},
};
use tokio_util::io::StreamReader;
use tracing::warn;

use crate::audio_player::{AudioPlayerError, AudioPlayerResult};

pub(crate) async fn convert(
    response: Response,
) -> AudioPlayerResult<Decoder<StreamDownload<impl StorageProvider>>> {
    let stream = StreamReader::new(response.bytes_stream().map_err(std::io::Error::other));

    warn!("Lock here -> build stream");

    let reader = match StreamDownload::new_async_read(
        AsyncReadStreamParams::new(stream),
        TempStorageProvider::new(),
        stream_download::Settings::default(),
    )
    .await
    {
        Ok(reader) => Ok(reader),
        Err(err) => Err(AudioPlayerError::FailedBuildStreamReaderError(
            err.to_string(),
        )),
    }?;

    warn!("actually lock here -> build stream download");

    let decoder = match Decoder::new(reader) {
        Ok(decoder) => Ok(decoder),
        Err(err) => Err(AudioPlayerError::FailedBuildDecoderError(err.to_string())),
    }?;

    Ok(decoder)
}
