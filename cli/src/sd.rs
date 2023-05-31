use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use surf::Client;

const STYLE: &str = "style of Anato Finnstark,iridescent, messy, streaks";
const NEGATIVE_PROMPT: &str = "nsfw, missing fingers, missing limbs, borderline, text, duplicate, error, out of frame, watermark, low quality, ugly, deformed, blur, bad-artist";

#[derive(Serialize, Deserialize)]
struct PromptParams {
    prompt: String,
    batch_size: usize,
    steps: usize,
    save_images: bool,
    sampler_index: String,
    negative_prompt: String,
}

#[derive(Serialize, Deserialize)]
struct ImageResponse {
    images: Vec<String>,
    info: String,
}

// DPM++ SDE Karras

pub async fn get_image(client: &Client, prompt: &str) {
    let prompt_params = PromptParams {
        batch_size: 4,
        prompt: format!("{},{}", STYLE, prompt),
        steps: 100,
        sampler_index: "DPM++ SDE Karras".to_string(),
        save_images: true,
        negative_prompt: NEGATIVE_PROMPT.to_string(),
    };

    let result: ImageResponse = client
        .post("http://127.0.0.1:7860/sdapi/v1/txt2img")
        .body_json(&prompt_params)
        .unwrap()
        .recv_json::<ImageResponse>()
        .await
        .unwrap();

    for (i, image) in result.images.iter().enumerate() {
        let image = general_purpose::STANDARD.decode(image).unwrap();
        std::fs::write(format!("image_{}.png", i), image).unwrap();
    }

    std::fs::write("info.txt", result.info).unwrap();
}
