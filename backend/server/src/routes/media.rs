use crate::SinglePath;
use actix_web::http::header;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse, web::Query};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufReader;
use std::io::Seek;
use std::io::{Read, SeekFrom};
use std::path::PathBuf;
use utils::database::establish_connection;
use utils::models::{MediaSource, RecommendedMediaEntry};
use utils::status_com::{ErrorCode, MessageRes};
use utils::visit_dirs::visit_dirs;
use utils::{models, schema};
use utoipa::ToSchema;

fn parse_range(range: actix_web::http::header::HeaderValue) -> (usize, Option<usize>) {
    let range_str = range.to_str().ok().unwrap(); // Safely convert to str, return None if failed
    let range_separated_clear = range_str.replace("bytes=", "");
    let range_separated: Vec<&str> = range_separated_clear.split('-').collect(); // Split the range

    // Parse the start and end values safely
    let start = range_separated.first().unwrap().parse::<usize>().unwrap();

    (
        start,
        match range_separated.get(1) {
            Some(v) => {
                if v == &"" {
                    None
                } else {
                    Some(v.parse::<usize>().unwrap())
                }
            }
            None => None,
        },
    )
}

fn is_media_path_whitelisted(l: Vec<MediaSource>, p: PathBuf) -> bool {
    let mut r = false;

    if !p.exists() {
        return false;
    }

    l.iter().for_each(|le| {
        if !r
            && p.canonicalize()
                .unwrap()
                .starts_with(PathBuf::from(&le.directory_path).canonicalize().unwrap())
            && le.enabled
        {
            r = true
        }
    });
    r
}

#[utoipa::path(
    get,
    path = "/private/media/download",
    responses((status = 200, description = "Binary media file", content_type = "application/octet-stream"), (status = 404, description = "File not found."), (status = 416), (status = 403, description = "Media center may be disabled.")),
    tags = ["media", "private"]
)]
pub async fn media_request(info: Query<SinglePath>, req: HttpRequest) -> HttpResponse {
    use models::MediaSource;
    use models::RecommendedMediaEntry;
    use schema::MediaSources::dsl::*;
    use schema::RecommendedMedia::dsl::*;

    let connection = &mut establish_connection();

    // Determine the requested file path
    let requested_file_path = &info.path;

    let current_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards.");

    if !requested_file_path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let database_insert_execution = diesel::insert_into(RecommendedMedia)
        .values(RecommendedMediaEntry {
            file_path: requested_file_path.to_string_lossy().to_string(),
            last_view: current_ts.as_millis() as i64,
        })
        .on_conflict(file_path)
        .do_update()
        .set(last_view.eq(current_ts.as_millis() as i64))
        .execute(connection);

    if let Err(database_error) = database_insert_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseInsertFailed(database_error.to_string()));
    }

    // Implement HTTP Ranges
    let headers = req.headers();
    let range = headers.get(actix_web::http::header::RANGE);

    let mime = utils::mime::guess_mime(requested_file_path.to_path_buf());

    let whitelist_vector: Vec<MediaSource> = MediaSources
        .select(MediaSource::as_select())
        .get_results(connection)
        .unwrap();

    if !is_media_path_whitelisted(
        whitelist_vector,
        fs::canonicalize(requested_file_path.clone()).unwrap(),
    ) {
        return HttpResponse::Forbidden().json(ErrorCode::MissingApiPermissions.as_error_message());
    }

    if requested_file_path.is_dir() {
        return HttpResponse::BadRequest().json(ErrorCode::FileError.as_error_message());
    }

    match range {
        None => {
            // Does the file even exist
            HttpResponse::Ok()
                .insert_header((
                    header::CONTENT_TYPE,
                    mime.unwrap_or("application/octet-stream".to_string()),
                ))
                .insert_header(header::ContentEncoding::Identity)
                .insert_header((header::ACCEPT_RANGES, "bytes"))
                .body(fs::read(requested_file_path).unwrap())
        }
        Some(e) => {
            let byte_range = parse_range(e.clone());
            let file = File::open(&requested_file_path).unwrap();
            let mut reader = BufReader::new(file);
            let filesize: usize = reader
                .get_ref()
                .metadata()
                .unwrap()
                .len()
                .try_into()
                .unwrap_or(0);

            if byte_range.0 > filesize {
                return HttpResponse::RangeNotSatisfiable()
                    .json(ErrorCode::LeftRangeTooHigh.as_error_message());
            }

            if let Some(right_limit) = byte_range.1 {
                if right_limit > filesize {
                    return HttpResponse::RangeNotSatisfiable()
                        .json(ErrorCode::RightRangeTooHigh.as_error_message());
                }
            }

            let buffer_length = byte_range.1.unwrap_or(filesize) - byte_range.0;
            let _ = reader.seek(SeekFrom::Start(byte_range.0 as u64));
            let mut buf = vec![0; buffer_length]; // A buffer with the length buffer_length
            reader.read_exact(&mut buf).unwrap();

            HttpResponse::PartialContent()
                .insert_header(header::ContentEncoding::Identity)
                .insert_header((header::ACCEPT_RANGES, "bytes"))
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    format!(
                        "inline; filename=\"{}\"",
                        &requested_file_path.file_name().unwrap().to_str().unwrap()
                    ),
                ))
                .insert_header((
                    header::CONTENT_RANGE,
                    format!(
                        "bytes {}-{}/{}",
                        byte_range.0,
                        byte_range.1.unwrap_or(filesize - 1),
                        filesize
                    ), // We HAVE to subtract 1 from the actual file size
                ))
                .insert_header((header::VARY, "*"))
                .insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "Range"))
                .insert_header((header::CONTENT_LENGTH, buf.len()))
                .insert_header((
                    header::CONTENT_TYPE,
                    mime.unwrap_or("application/octet-stream".to_string()),
                ))
                .body(buf)
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MediaSourcesSchema {
    locations: Vec<models::MediaSource>,
}

#[utoipa::path(
    post,
    path = "/private/media/sources",
    request_body = MediaSourcesSchema,
    responses((status = 200)),
    tags = ["media", "private"]
)]
/// Update media sources
///
/// Media sources control what content is shown to the user in Media Center and to which files the
/// user has access.
pub async fn update_media_source_list(json: Json<MediaSourcesSchema>) -> HttpResponse {
    use models::MediaSource;
    use schema::MediaSources::dsl::*;

    let connection = &mut establish_connection();

    let locations = &json.locations;

    // The frontend only sends an updated array of all resources.
    // It is easier to truncate the entire table and then rewrite its' contents.

    if let Err(database_error) = diesel::delete(MediaSources).execute(connection) {
        return HttpResponse::InternalServerError().json(
            ErrorCode::DatabaseTruncateFailed(database_error.to_string()).as_error_message(),
        );
    }

    let deserialized_locations: Vec<&MediaSource> = locations
        .iter()
        .filter(|&e| PathBuf::from(&e.directory_path).exists())
        .collect();

    for location in deserialized_locations {
        let database_update_execution = diesel::insert_into(MediaSources)
            .values(location)
            .on_conflict(directory_path)
            .do_update()
            .set(location)
            .execute(connection);

        if let Err(database_error) = database_update_execution {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    }

    HttpResponse::Ok().json(MessageRes::from("The media source have been updated."))
}

#[utoipa::path(
    get,
    path = "/private/media/sources",
    responses((status = 200, body = MediaSourcesSchema)),
    tags = ["media", "private"]
)]
/// List of media sources.
///
/// See [`update_media_source_list`] for reference.
pub async fn get_media_source_list() -> HttpResponse {
    use models::MediaSource;
    use schema::MediaSources::dsl::*;

    let locations: Vec<MediaSource> = MediaSources
        .select(MediaSource::as_select())
        .get_results(&mut establish_connection())
        .unwrap();

    HttpResponse::Ok().json(MediaSourcesSchema { locations })
}

#[derive(Serialize, ToSchema)]
struct MediaListRes {
    media: Vec<models::MediaEntry>,
}

#[utoipa::path(
    get,
    path = "/private/media/files",
    responses((status = 200, body = MediaListRes), (status = 403, description = "Media center may be disabled.")),
    tags = ["media", "private"]
)]
/// List of media files
///
/// This list is controlled by the active media sources.
pub async fn get_media_list() -> HttpResponse {
    use schema::Media::dsl::*;
    use schema::MediaSources::dsl::*;

    use models::MediaEntry;

    let connection = &mut establish_connection();

    let sources: Vec<PathBuf> = MediaSources
        .select(MediaSource::as_select())
        .get_results(connection)
        .unwrap()
        .into_iter()
        .filter(|source| source.enabled)
        .map(|source| PathBuf::from(source.directory_path))
        .filter(|path| path.exists())
        .collect();

    let mut all_media_file_paths: Vec<PathBuf> = vec![];

    for source in sources {
        let source_specific_contents = visit_dirs(source).unwrap();
        source_specific_contents
            .map(|file| file.path())
            .filter(|path| path.is_file())
            .for_each(|path| all_media_file_paths.push(path));
    }

    let mut media_metadata = Media
        .select(MediaEntry::as_select())
        .get_results(connection)
        .unwrap()
        .into_iter();

    let mut completed_media_entries: Vec<MediaEntry> = Vec::new();

    for media_file_path in all_media_file_paths {
        let search = media_metadata
            .find(|entry: &MediaEntry| PathBuf::from(entry.file_path.clone()) == media_file_path);
        if let Some(defined_metadata) = search {
            completed_media_entries.push(defined_metadata);
        } else {
            completed_media_entries.push(MediaEntry::default_with_file_path(media_file_path));
        }
    }

    return HttpResponse::Ok().json(MediaListRes {
        media: completed_media_entries,
    });
}

/// Media cover
///
/// Only media covers that are in an active media source will be shown.
#[utoipa::path(get, path = "/private/media/cover", responses((status = 200, content_type = "image/"), (status = 404, description = "Media not found.")), tags = ["media", "private"], params(("path" = String, Query)))]
pub async fn get_cover(info: Query<SinglePath>) -> HttpResponse {
    use models::MediaSource;
    use schema::MediaSources::dsl::*;

    let sources: Vec<MediaSource> = MediaSources
        .select(MediaSource::as_select())
        .get_results(&mut establish_connection())
        .unwrap();

    let cover_uri = &info.path;

    if cover_uri == &PathBuf::from("/music") {
        let cover = include_str!("../../../assets/music_default.svg");
        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "image/svg+xml".to_string()))
            .body(cover.bytes().collect::<Vec<u8>>())
    } else {
        if !cover_uri.exists() {
            return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
        }

        let cover_path = cover_uri
            .canonicalize()
            .expect("Canonicalizing a path failed.");

        let allowed_cover_file_extensions = ["png", "jpg", "jpeg", "webp", "gif", "tiff"];
        if !allowed_cover_file_extensions
            .contains(&cover_path.extension().unwrap().to_str().unwrap())
        {
            return HttpResponse::UnsupportedMediaType()
                .json(ErrorCode::ProtectedExtension.as_error_message());
        }

        if !is_media_path_whitelisted(sources, cover_uri.to_path_buf()) {
            HttpResponse::Forbidden().json(ErrorCode::MissingApiPermissions.as_error_message())
        } else {
            let fh = fs::read(cover_path).unwrap();
            HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
        }
    }
}

pub fn get_media_enabled_database() -> bool {
    use models::Configurations;
    use schema::Configuration::dsl::*;

    Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .media_enabled
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MediaEnabledSchema {
    enabled: bool,
}

/// Is media center enabled?
#[utoipa::path(get, path = "/private/media/enabled", responses((status = 200, body = MediaEnabledSchema)), tags = ["media", "private"])]
pub async fn get_media_enabled_handler() -> HttpResponse {
    HttpResponse::Ok().json(MediaEnabledSchema {
        enabled: get_media_enabled_database(),
    })
}

/// Set media center activation
#[utoipa::path(post, path = "/private/media/enabled", responses((status = 200)), request_body = MediaEnabledSchema, tags = ["media", "private"])]
pub async fn set_enable_media(e: Json<MediaEnabledSchema>) -> HttpResponse {
    use schema::Configuration::dsl::*;

    let connection = &mut establish_connection();

    let database_update_execution = diesel::update(Configuration)
        .set(media_enabled.eq(e.enabled))
        .execute(connection);

    if let Err(database_error) = database_update_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message());
    }

    return HttpResponse::Ok().json(MessageRes::from(
        "The media center status has been updated.",
    ));
}

#[derive(Serialize, ToSchema)]
pub struct RecommendationsRes {
    recommendations: Vec<RecommendedMediaEntry>,
}

/// Media files history
#[utoipa::path(get, path = "/private/media/history", tags = ["private", "media"], responses((status = 200, body = RecommendationsRes)))]
pub async fn read_full_media_history() -> HttpResponse {
    use models::RecommendedMediaEntry;
    use schema::RecommendedMedia::dsl::*;

    let connection = &mut establish_connection();

    let queried_entries = RecommendedMedia
        .select(RecommendedMediaEntry::as_select())
        .get_results(connection)
        .unwrap();

    let filtered_entries: Vec<RecommendedMediaEntry> = queried_entries
        .into_iter()
        .filter(|e| PathBuf::from(&e.file_path).exists())
        .collect();

    return HttpResponse::Ok().json(RecommendationsRes {
        recommendations: filtered_entries,
    });
}

#[derive(Deserialize)]
pub struct MetadataReq {
    name: Option<String>,
    genre: Option<String>,
    cover: Option<String>,
    artist: Option<String>,
}

#[utoipa::path(get, path = "/private/media/metadata/{file}", params(("file" = String, Path)), tags = ["private", "media"], responses((status = 200, body = RecommendationsRes)))]
/// Update media metadata
pub async fn update_media_metadata(path: Path<PathBuf>, json: Json<MetadataReq>) -> HttpResponse {
    use models::MediaEntry;
    use schema::Media::dsl::*;

    let new_media_entry = MediaEntry {
        file_path: path.into_inner().to_string_lossy().to_string(),
        genre: json.genre.clone(),
        name: json.name.clone(),
        artist: json.artist.clone(),
        cover: json.cover.clone(),
    };

    let connection = &mut establish_connection();

    let wx = diesel::insert_into(Media)
        .values(&new_media_entry)
        .on_conflict(file_path)
        .do_update()
        .set((
            genre.eq(&new_media_entry.genre),
            name.eq(&new_media_entry.name),
            artist.eq(&new_media_entry.artist),
            cover.eq(&new_media_entry.cover),
        ))
        .execute(connection);

    if let Err(database_error) = wx {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseInsertFailed(database_error.to_string()));
    }

    return HttpResponse::Ok().json(MessageRes::from("The media metadata has been updated."));
}
