use std::{fs::{DirEntry, File}, io::Cursor, path::{Path, PathBuf}};

use rocket::{Data, Request, http::{ContentType, Status}, response::{DEFAULT_CHUNK_SIZE, Responder, Response}};

use crate::users::LoginedUser;
pub struct FileServer(PathBuf);

fn entry_to_line(entry: DirEntry) -> String {
    let filename = entry.file_name().into_string();
    let metadata = entry.metadata().unwrap();

    match filename {
        Ok(mut string) => {
            let mut result = String::from("<a href=");

            if metadata.is_dir() {
                result += &string;
                string.push('/');
            } else {
                result += &string;
            }

            result += "><li>";
            result += &string;
            result += "</li></a>";

            result
        }
        Err(_osstring) => {
            String::from("")
        }
    }
}

impl<'r> Responder<'r> for FileServer {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        // Different behavior depending if path is a directory
        let path = Path::new(&self.0);

        if path.exists() {
            if path.is_dir() {
                if let Ok(entries) = path.read_dir() {
                    let mut body = String::from("<html><head>");
                    body += "<base href=";
                    body += &req.uri().to_string();
                    body += "/></head><body>";

                    body += "<h2> Index for ";
                    body += &req.uri().to_string();
                    body += "</h2>";

                    if &req.uri().to_string() != "/public" {
                        body += "<li> <a href='..'>..</a> </li>"; 
                    }

                    entries.filter_map(|res| res.ok())
                        .for_each(|entry| body += &entry_to_line(entry));
                    
                    body += "</body></html>";

                    Response::build()
                        .sized_body(Cursor::new(body))
                        .header(ContentType::HTML)
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

#[post("/upload/public/<file..>?", format = "any", data = "<data>")]
pub fn public_upload(_user: &LoginedUser, file: PathBuf, data: Data) -> Result<String, std::io::Error> {
    println!("public");
    let path = Path::new("storage/public/").join(file);
    data.stream_to_file(path).map(|n| n.to_string())
}

#[post("/upload/private/<file..>?", format = "any", data = "<data>")]
pub fn private_upload(_user: &LoginedUser, file: PathBuf, data: Data) -> Result<String, std::io::Error> {
    println!("private");
    let path = Path::new("storage/private/").join(file);
    data.stream_to_file(path).map(|n| n.to_string())
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
