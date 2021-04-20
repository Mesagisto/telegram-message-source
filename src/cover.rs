use anyhow::anyhow;
use std::path::Path;

pub fn extract_first_image(music_filename: &Path, image_filename: &Path) -> anyhow::Result<()> {
    let tag = read_tag(music_filename)?;
    let first_picture = tag.pictures().next();

    if let Some(p) = first_picture {
        match image::load_from_memory(&p.data) {
            Ok(image) => {
                image.save(&image_filename).map_err(|e| {
                    anyhow!("Couldn't write image file {:?}: {}", image_filename, e)
                })?;
            }
            Err(e) => return Err(anyhow!("Couldn't load image: {}", e)),
        };

        Ok(())
    } else {
        Err(anyhow!("No image found in music file"))
    }
}

fn read_tag(path: &Path) -> anyhow::Result<id3::Tag> {
    id3::Tag::read_from_path(&path).or_else(|e| {
        eprintln!(
            "Warning: file metadata is corrupted, trying to read partial tag: {}",
            path.display()
        );
        e.partial_tag
            .clone()
            .ok_or_else(|| anyhow!("Error reading music file {:?}: {}", path, e))
    })
}
