use std::error::Error;
use std::sync::Arc;

pub trait EyreToAnyhow<T> {
    fn into_anyhow(self) -> anyhow::Result<T>;
}

impl<T> EyreToAnyhow<T> for epub_builder::Result<T> {
    fn into_anyhow(self) -> anyhow::Result<T> {
        match self {
            Ok(it) => Ok(it),
            Err(e) => {
                let err: Box<dyn Error + Send + Sync + 'static> = e.into();
                let err: Arc<dyn Error + Send + Sync + 'static> = Arc::from(err);
                Err(anyhow::Error::new(err))
            }
        }
    }
}
