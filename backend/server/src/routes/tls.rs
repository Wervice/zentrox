use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::HttpResponse;
use diesel::prelude::*;
use log::error;
use serde::Serialize;
use std::{fs, path};
use utils::status_com::ErrorCode;
use utils::{database::establish_connection, models, schema, status_com::MessageRes};
use utoipa::ToSchema;

#[derive(MultipartForm, ToSchema)]
pub struct TlsUploadForm {
    #[multipart(limit = "1GB")]
    #[schema(value_type = Vec<u8>)]
    file: TempFile,
}

/// Upload new TLS certificate
///
/// This certificate is used to encrypt the connection between your browser and Zentrox.
/// The name of then new certificate is stored in the configuration file.
#[utoipa::path(
    post,
    path = "/private/tls/upload",
    request_body(content = TlsUploadForm, content_type = "multipart/form-data"),
    responses((status = 200)),
    tags = ["private", "tls"]
)]
pub async fn upload_tls(MultipartForm(form): MultipartForm<TlsUploadForm>) -> HttpResponse {
    use schema::Configuration::dsl::*;

    let file_name = form
        .file
        .file_name
        .unwrap_or_else(|| "tls".to_string())
        .replace("..", "")
        .replace("/", "");

    let base_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join(&file_name);

    let database_update_execution = diesel::update(Configuration)
        .set(tls_cert.eq(&file_name))
        .execute(&mut establish_connection());

    if let Err(database_error) = database_update_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message());
    }

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &base_path);

    match fs::remove_file(&tmp_file_path) {
        Ok(_) => {}
        Err(_) => {
            error!(
                "Unable to remove temporary file at {}",
                tmp_file_path.to_string_lossy()
            );
            return HttpResponse::InternalServerError()
                .json(ErrorCode::FileError.as_error_message());
        }
    };

    HttpResponse::Ok().json(MessageRes::from("Certificate uploaded and stored."))
}

#[derive(Serialize, ToSchema)]
struct CertNameRes {
    name: String,
}

/// Name of active certificate.
#[utoipa::path(
    get,
    path = "/private/tls/name",
    responses((status = 200, body = CertNameRes)),
    tags = ["private", "tls"]
)]
pub async fn cert_names() -> HttpResponse {
    use models::Configurations;
    use schema::Configuration::dsl::*;
    let name = Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .tls_cert;

    HttpResponse::Ok().json(CertNameRes { name })
}
