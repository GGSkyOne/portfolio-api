use std::path::PathBuf;

use actix_files as fs;
use actix_web::{web, Error};

pub fn projects_config(config: &mut web::ServiceConfig) {
    config.service(
        web::resource("/projects")
            .route(web::get().to(get_projects))
    );
}

async fn get_projects() -> Result<fs::NamedFile, Error> {
    let path: PathBuf = "static/projects.json".parse().unwrap();
    let file = fs::NamedFile::open(path).unwrap();
    Ok(file)
}