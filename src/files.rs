use std::{ffi::OsString, fs::{DirEntry, File}, io::Cursor, path::{Path, PathBuf}};

use rocket::{Request, http::{ContentType, Status}, response::{DEFAULT_CHUNK_SIZE, Responder, Response}};

use crate::users::LoginedUser;
pub struct FileServer(PathBuf);

fn entry_to_line(entry: DirEntry) -> Result<String, OsString> {
    let filename = entry.file_name().into_string();
    let metadata = entry.metadata().unwrap();

    match filename {
        Ok(string) => {
            let mut result = String::from("<li href=");

            if metadata.is_dir() {
                result += &string
            } else {
                result += &string
            }

            result.push('>');
            result += &string;
            result += "</li>";

            Ok(result)
        }
        Err(osstring) => {
            Err(osstring)
        }
    }
}

impl<'r> Responder<'r> for FileServer {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        // Different behavior depending if path is a directory
        let path = Path::new(&self.0);

        if path.exists() {
            if path.is_dir() {
                if let Ok(entries) = path.read_dir() {
                    let names = entries
                        .filter_map(|entry| entry.ok())
                        .filter_map(|entry| entry_to_line(entry).ok())
                        .fold(String::new(), |a, b| a + &b + "\n");

                    Response::build()
                        .header(ContentType::Plain)
                        .sized_body(Cursor::new(names))
                        .ok()
                } else {
                    Err(Status::NotFound)
                }
            } else {
                Response::build()
                    .chunked_body(File::open(path).unwrap(), DEFAULT_CHUNK_SIZE)
                    .raw_header("Content-Disposition", "attachment")
                    .ok()
            }
        } else {
            Err(Status::NotFound)
        }
    }
}

#[get("/public")]
pub fn public_root() -> FileServer {
    let path = Path::new("storage/public/").to_path_buf();
    FileServer(path)
}

#[get("/public/<file..>")]
pub fn public(file: PathBuf) -> FileServer {
    let path = Path::new("storage/public/").join(file);
    FileServer(path)
}

#[get("/private")]
pub fn private_root(_user: &LoginedUser) -> FileServer {
    let path = Path::new("storage/private/").to_path_buf();
    FileServer(path)
}

#[get("/private/<file..>")]
pub fn private(_user: &LoginedUser, file: PathBuf) -> FileServer {
    let path = Path::new("storage/private/").join(file);
    FileServer(path)
}
