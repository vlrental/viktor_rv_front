//! Lightweight previews for the current public RV media.
//! Unknown or newly uploaded photos continue to use their original URL.

const THUMB_IDS: &[&str] = &[
    "0543922d-40a8-44ed-a6a4-e9c139d34ab6",
    "f064605d-cbb1-4625-bb33-3ab601b0a1e6",
    "64d84eba-b4d8-4fb7-b863-de0798e0f756",
    "92676130-5e90-489d-a876-38cc8900c569",
    "57c5ad49-1622-48e9-9e30-05f2313ba631",
    "eb531ba0-59fb-4930-8aa4-057bd6daf8e1",
    "efba5f6e-e444-4d9a-8fc1-4516454147a4",
    "c126b58c-46b8-421a-959d-3b4e3f652046",
    "499dcf26-99f0-459f-87c1-728c5fc20b59",
    "3f44833f-0562-4f22-9847-13ffc46a45ef",
    "68ed1ebb-7cca-4456-9f60-cc78afe9c00d",
    "978a775f-cb15-42c7-8c0d-720e97a2df52",
    "4ec2ff5e-d8c9-4879-b0f2-dabab007188d",
    "017db61e-4b86-4c45-9bc2-770aa88cbf3c",
    "d147526c-bf9c-464a-8a21-71bd2d391e7b",
    "4295edf9-16fd-4d2a-bcd3-e44e0619c6aa",
    "29ad7e94-3c49-4ad2-a308-5329e5b1d66b",
    "2248ed75-2208-47e8-a723-4d67388cb1ad",
    "46f1d73b-e54b-48ad-81e8-3a725496672a",
    "6747330c-ea9d-4fe1-aaae-53585364a613",
    "8bc9baf6-2a17-48f6-8391-22843666bee5",
    "f69f3398-79d6-4674-8dd8-f5b25cad99b0",
    "21399f92-d571-4caa-ac04-f78bafdb2504",
    "3cb07955-3429-490d-a1a4-1500783f19c5",
    "61211640-6d31-426f-a642-159d1e5202e2",
    "107f5ebd-73d5-4790-8cfe-50a770db2a69",
    "68b79b27-f25b-4946-82de-7378ca7d49ab",
    "2d15a265-1622-4ac1-bc5a-52a55ff2738e",
    "53a83e6e-681d-4307-9b7d-e136c65791f1",
    "90f29d02-8e6d-4145-98bb-ff9a1a20e621",
    "2f4991ab-604c-4c48-bb92-4abb54495373",
    "48b16d9e-d2d6-41c2-be8c-8d0d71c1c02b",
    "0bcd7a76-53f1-4ac8-a91e-326623b9e913",
    "f7cbed10-77f0-4390-8676-58ef67b3a2a9",
    "da3f0f1e-8e6a-47e7-abba-2698ea757a32",
    "c01a25dc-c538-48a5-91f0-217e796ca118",
    "60e826cd-8446-4b21-8ac4-e10f88e57bfb",
    "1de3539b-f78d-40c3-bb77-16bf5ea53bac",
    "9071099c-0ab3-49a7-a2fd-857e478f95da",
    "85401026-b593-4cf8-8ecb-403cbb5c12c6",
    "c6cab577-f71c-4ac7-8662-50b51b123a54",
    "661afb4a-e8d5-454a-8bcf-91e86aad6907",
];

const COVER_IDS: &[&str] = &[
    "0543922d-40a8-44ed-a6a4-e9c139d34ab6",
    "c126b58c-46b8-421a-959d-3b4e3f652046",
    "d147526c-bf9c-464a-8a21-71bd2d391e7b",
    "f69f3398-79d6-4674-8dd8-f5b25cad99b0",
    "53a83e6e-681d-4307-9b7d-e136c65791f1",
    "c01a25dc-c538-48a5-91f0-217e796ca118",
];

fn image_id(source_url: &str) -> Option<&str> {
    source_url.rsplit('/').next()?.split('.').next()
}

fn preview_url_for_base(base_url: &str, image_id: &str, kind: &str) -> String {
    format!(
        "{}/rv-previews/{image_id}-{kind}.webp",
        base_url.trim_end_matches('/')
    )
}

fn preview_url(image_id: &str, kind: &str) -> String {
    preview_url_for_base(&crate::api::frontend_base_url(), image_id, kind)
}

pub fn thumbnail_for(source_url: &str) -> Option<String> {
    let id = image_id(source_url)?;
    THUMB_IDS.contains(&id).then(|| preview_url(id, "thumb"))
}

pub fn cover_preview_for(source_url: &str) -> Option<String> {
    let id = image_id(source_url)?;
    COVER_IDS.contains(&id).then(|| preview_url(id, "preview"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_urls_follow_production_and_pages_base_paths() {
        let id = COVER_IDS[0];
        assert_eq!(
            preview_url_for_base("https://vlrental.ca/", id, "preview"),
            format!("https://vlrental.ca/rv-previews/{id}-preview.webp")
        );
        assert_eq!(
            preview_url_for_base("https://vlrental.github.io/viktor_rv_front", id, "thumb"),
            format!("https://vlrental.github.io/viktor_rv_front/rv-previews/{id}-thumb.webp")
        );
    }

    #[test]
    fn only_current_media_has_local_previews() {
        assert_eq!(THUMB_IDS.len(), 42);
        assert_eq!(COVER_IDS.len(), 6);
        assert!(THUMB_IDS.iter().all(|id| !id.is_empty()));
        assert!(COVER_IDS.iter().all(|id| THUMB_IDS.contains(id)));
        assert!(thumbnail_for("https://example.com/new-photo.jpg").is_none());
        assert!(cover_preview_for("https://example.com/new-photo.jpg").is_none());
    }
}
