use std::{
    fs::File,
    io::{Error, Write},
};

pub enum Format {
    BMP,
    TGA
}

// -- TGA parameters --
// All values little-endian
const TGA_DEFAULT_HEADER: [u8; 18] = [
    0x00, // ID length: no ID field
    0x00, // Color map type: no color map
    0x02, // Image type: uncompressed true color
    0x00, 0x00, 0x00, 0x00, 0x00, // Irrelevant color map stuff
    0x00, 0x00, 0x00, 0x00, // (x, y) origin (should be 0)
    0x00, 0x00, // Width in pixels (we want to change this)
    0x00, 0x00, // Height in pixels (and this)
    0x18, // Bits per pixel: 24
    0x00, // Random stuff we don't care about
];
const TGA_WIDTH_INDEX: usize = 12;
const TGA_HEIGHT_INDEX: usize = 14;

// -- BMP parameters --
// Standard bitmap header followed by BITMAPCOREHEADER
const BMP_DEFAULT_HEADER: [u8; 26] = [
    'B' as u8, 'M' as u8, // BMP identifier
    0x00, 0x00, 0x00, 0x00, // Filesize (we want to change this)
    0x00, 0x00, 0x00, 0x00, // Reserved (can be ignored)
    0x1A, 0x00, 0x00, 0x00, // Image data offset (size of header)
    0x0C, 0x00, 0x00, 0x00, // Header size: 12 bytes
    0x00, 0x00, // Width in pixels (we want to change this)
    0x00, 0x00, // Height in pixels (we want to change this too!)
    0x01, 0x00, // Number of color planes, whatever that is (must be 1 anyway)
    0x18, 0x00, // Bits per pixel: 24
];
const BMP_FILESIZE_INDEX: usize = 2;
const BMP_WIDTH_INDEX: usize = 18;
const BMP_HEIGHT_INDEX: usize = 20;

// Output the generated image to a .tga file
pub fn write_tga(
    image_data: &[Vec<u8>],
    filename: &str,
) -> Result<(), Error> {
    let mut header = TGA_DEFAULT_HEADER.clone().to_vec();
    let height = image_data.len() as u16;
    let width = (image_data[0].len() / 3) as u16;
    // Put dimensions in the header
    header.splice(TGA_WIDTH_INDEX..TGA_WIDTH_INDEX + 2, width.to_le_bytes());
    header.splice(TGA_HEIGHT_INDEX..TGA_HEIGHT_INDEX + 2, height.to_le_bytes());
    // Create and write the file
    let mut output_file = File::create(filename)?;
    output_file.write(&header)?;
    for row in image_data {
        output_file.write(row)?;
    }
    Ok(())
}

// Output the generated image to a .bmp file
pub fn write_bmp(
    image_data: &[Vec<u8>],
    filename: &str,
) -> Result<(), Error> {
    let mut header = BMP_DEFAULT_HEADER.clone().to_vec();
    let height = image_data.len() as u16;
    let width = (image_data[0].len() / 3) as u16;
    header.splice(BMP_WIDTH_INDEX..BMP_WIDTH_INDEX + 2, width.to_le_bytes());
    header.splice(BMP_HEIGHT_INDEX..BMP_HEIGHT_INDEX + 2, height.to_le_bytes());
    let padding: Vec<u8> = vec![0; ((width * 3) % 4) as usize]; // The length of every row of image data must be a multiple of 4
    // Do the padding (i was tired when writing this)
    let image_data = image_data
        .to_owned()
        .into_iter()
        .map(|mut row| {
            row.extend_from_slice(&padding);
            row
        })
        .flatten()
        .collect::<Vec<u8>>();
    let filesize = (header.len() + image_data.len()) as u32;
    // Put filesize and dimensions in the header
    header.splice(
        BMP_FILESIZE_INDEX..BMP_FILESIZE_INDEX + 4,
        filesize.to_le_bytes(),
    );
    let mut output_file = File::create(filename)?;
    output_file.write(&header)?;
    output_file.write(&image_data)?;
    Ok(())
}
