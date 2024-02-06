// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use image::{DynamicImage, ImageFormat};
use ddsfile::{Dds, D3DFormat};
use std::{io::{self, Read, Cursor, Seek, SeekFrom}, path, sync::{Mutex, MutexGuard}};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use tauri::api::{dir::{read_dir, DiskEntry}, file};
use std::fs::File;
use std::fs;
use lazy_static::lazy_static;

pub struct FileData {
    pub handle: File,
    pub contents: Vec<u8>,
}

lazy_static! {
    static ref FILE_DATA: Mutex<Option<FileData>> = Mutex::new(None);
    static ref FACES_SEQUENCE: Vec<u8> = vec![
        0x04, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x04, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00,
    ];
    static ref VERTICES_SEQUENCE: Vec<u8> = vec![
        0x00, 0x00, 0x01, 0x00, 
        0x00, 0x00, 0x00, 0x00, 
        0x00, 0x00, 0x00, 0x00, 
        0x00, 0x00, 0x00, 0x00, 
        0x00, 0x00, 0x00, 0x00, 
    ];
}

fn find_pattern_offset(pattern: &[u8], start_offset: Option<usize>) -> Option<usize> {
    let file_data = FILE_DATA.lock().expect("Lock failed");

    if let Some(ref data) = *file_data {
        let start = start_offset.unwrap_or(0); // Use provided offset, or default to 0.

        // Ensure the start offset is within the bounds of the file data.
        if start >= data.contents.len() {
            return None;
        }

        // Create a subslice of the contents starting from the start_offset.
        let content_slice = &data.contents[start..];

        // Search for the pattern in the subslice.
        content_slice.windows(pattern.len()).position(|window| window == pattern)
                     .map(|pos| pos + start + pattern.len())
    } else {
        None
    }
}

#[tauri::command]
fn get_meshes() -> Vec<DiskEntry> {
    let files = read_dir(".\\resources\\meshes\\all", false).unwrap();
    files
}

#[tauri::command]
fn load_file(file_path: &str) -> Vec<u8> {
    let content = fs::read(file_path).unwrap();
    let mut contents = vec![];
    let mut file_data = FILE_DATA.lock().expect("Lock failed");
    let mut f = File::open(file_path).expect("File not found");

    f.read_to_end(&mut contents).expect("Read failed");

    *file_data = Some(FileData {
        handle: f,
        contents
    });
    content
}

#[tauri::command]
fn load_texture() {
    let mut file_data = FILE_DATA.lock().expect("Lock failed");

    match *file_data {
        Some(ref mut file_data) => {
            let img_width = LittleEndian::read_u32(&file_data.contents[0x20..0x24]);
            let img_height = LittleEndian::read_u32(&file_data.contents[0x24..0x28]);
            let img_format = LittleEndian::read_u32(&file_data.contents[0x38..0x3c]);

            println!("Width: {:?}, Height: {:?}, Format: {:?}", img_width, img_height, img_format);

            let img_data = &file_data.contents[0x40..];

            let dds = Dds::new_d3d(img_width, img_height, Some(128), D3DFormat::DXT1, Some(1), Some(ddsfile::Caps2::CUBEMAP)).unwrap();

            let dds_data = dds.get_data(0).unwrap();
            let img: DynamicImage = match  img_format {
                0x41 => {
                    image::load_from_memory_with_format(&dds_data, ImageFormat::Dds).unwrap()
                },
                _ => panic!("Unsupported format"),
            };

            img.save("test.png").unwrap();

            /* let img = match img_format {
                0x31545844 => {
                    let dds = Dds::read(&mut Cursor::new(img_data)).unwrap();
                    let img = DynamicImage::from_rgba8(dds.data, img_width, img_height).unwrap();
                    img
                },
                _ => {
                    let img = DynamicImage::from_rgba8(img_data.to_vec(), img_width, img_height).unwrap();
                    img
                }
            };

            img.save("test.png").unwrap(); */
        },
        None => {
            return;
        }
    }
}

#[tauri::command]
fn get_vertices(padding: i32, pad_interval: i32) -> Vec<Vec<f32>> {
    let faces_offset =  match find_pattern_offset(&FACES_SEQUENCE, None) {
        Some(offset) => {
            offset
        },
        None => {
            return vec![];
        }
    };

    println!("faces_offset: {:?}", faces_offset);

    let offset = match find_pattern_offset(&VERTICES_SEQUENCE, Some(faces_offset)) {
        Some(offset) => {
            offset
        },
        None => {
            return vec![];
        }
    };

    println!("offset: {:?}", offset);

    let mut file_data = FILE_DATA.lock().expect("Lock failed");

    let data = match *file_data {
        Some(ref mut file_data) => {
            let mut vertices = Vec::new();
            file_data.handle.seek(SeekFrom::Start(offset.try_into().unwrap()));
            let mut num_vertices = file_data.handle.read_i32::<LittleEndian>().unwrap();
            
            num_vertices = file_data.handle.read_i32::<LittleEndian>().unwrap();
            num_vertices = file_data.handle.read_i32::<LittleEndian>().unwrap();
            num_vertices = file_data.handle.read_i32::<LittleEndian>().unwrap();

            println!("num_vertices: {:?}", num_vertices);

            for i in 0..num_vertices {
                let x = file_data.handle.read_f32::<LittleEndian>().unwrap();
                let y = file_data.handle.read_f32::<LittleEndian>().unwrap();
                let z = file_data.handle.read_f32::<LittleEndian>().unwrap();
                let n1 = file_data.handle.read_f32::<LittleEndian>().unwrap();
                let n2 = file_data.handle.read_f32::<LittleEndian>().unwrap();
                let n3 = file_data.handle.read_f32::<LittleEndian>().unwrap();
                let u = file_data.handle.read_f32::<LittleEndian>().unwrap();
                let v = file_data.handle.read_f32::<LittleEndian>().unwrap();

                let xyz = vec![x, y, z, n1, n2, n3, u, v];

                vertices.push(xyz);
            }
            println!("vertices: {:?}", vertices);
            vertices
        },
        None => {
            return vec![];
        }
    };
    data
}

#[tauri::command]
fn get_faces(padding: i32, pad_interval: i32) -> Vec<Vec<i8>> {
    let offset = match find_pattern_offset(&FACES_SEQUENCE, None) {
        Some(offset) => {
            offset
        },
        None => {
            return vec![];
        }
    };
    let mut faces = Vec::new();
    let mut file_data = FILE_DATA.lock().expect("Lock failed");

    println!("offset: {:?}", offset);
    
    let _ = match *file_data {
        Some(ref mut file_data) => {
            file_data.handle.seek(SeekFrom::Start(offset.try_into().unwrap()));

            let num_faces = file_data.handle.read_i32::<LittleEndian>().unwrap();
            println!("num_faces: {:?}", num_faces);

            for _ in 0..num_faces {
                let a = file_data.handle.read_i8().unwrap();
                let b = file_data.handle.read_i8().unwrap();
                let c = file_data.handle.read_i8().unwrap();

                let abc = vec![a, b, c];

                faces.push(abc);
            }
        },
        None => {
            return vec![];
        }
    };

    println!("faces: {:?}", faces);
    faces
}

pub fn get_file_data() -> MutexGuard<'static, Option<FileData>> {
    FILE_DATA.lock().expect("Lock failed")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_meshes,
            load_file,
            get_vertices,
            get_faces,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
