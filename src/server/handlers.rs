use crate::Storage as _;
use warp::{http::StatusCode, Rejection, Reply};

type Result<T> = std::result::Result<T, Rejection>;

pub async fn insert(item: crate::Item) -> Result<impl Reply> {
    log::info!("got an item: {:?}", item);

    if item.version != crate::CURRENT_API_VERSION {
        log::warn!(
            "invalid version: {}. only '{}' is supported",
            item.version,
            crate::CURRENT_API_VERSION
        );
        let err = super::http::user_error(crate::Error::InvalidVersion {
            got: item.version,
            expected: crate::CURRENT_API_VERSION,
        });
        return Err(err);
    }

    crate::Youtube
        .insert(item)
        .await
        .map(|_| StatusCode::ACCEPTED)
        .map_err(|err| {
            if let Ok(err) = err.downcast::<crate::Error>() {
                return super::http::server_error(err);
            }
            panic!("we are in a dengerate state: the db cannot be opened")
        })
}

pub async fn current() -> Result<impl Reply> {
    log::trace!("getting the current song");
    crate::Youtube
        .current()
        .await
        .map(super::http::okay)
        .map_err(|err| {
            if let Ok(err) = err.downcast::<crate::Error>() {
                return super::http::server_error(err);
            }
            panic!("we are in a dengerate state: the db cannot be opened")
        })
}

pub async fn previous() -> Result<impl Reply> {
    log::trace!("getting the previous song");
    crate::Youtube
        .previous()
        .await
        .map(super::http::okay)
        .map_err(|err| {
            if let Ok(err) = err.downcast::<crate::Error>() {
                return super::http::server_error(err);
            }
            panic!("we are in a dengerate state: the db cannot be opened")
        })
}

pub async fn try_list() -> Result<impl Reply> {
    log::trace!("getting all of the songs");
    crate::Youtube
        .all()
        .await
        .map(super::http::okay)
        .map_err(|err| {
            if let Ok(err) = err.downcast::<crate::Error>() {
                return super::http::server_error(err);
            }
            panic!("we are in a dengerate state: the db cannot be opened")
        })
}
