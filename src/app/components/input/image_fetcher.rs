use crate::app::math::image;
use egui::epaint::ColorImage;
use poll_promise::{Promise, Sender};
use std::string::String;

// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)]

pub struct Fetcher {
    pub url: String,

    // #[serde(skip)] // opt-out serialization
    pub promise: Option<Promise<Result<Option<ehttp::Response>, String>>>,

    // #[serde(skip)] // opt-out serialization
    pub image: ColorImage,
}

impl Default for Fetcher {
    fn default() -> Self {
        Self {
            url: "https://picsum.photos/seed/0/640".to_string(),
            promise: Default::default(),
            image: ColorImage::new([1, 1], egui::Color32::BLACK),
        }
    }
}

impl Clone for Fetcher {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            promise: None,
            image: self.image.clone(),
        }
    }
}

type HTTPSender = Sender<Result<Option<ehttp::Response>, String>>;
type HTTPPromise = Promise<Result<Option<ehttp::Response>, String>>;

impl Fetcher {
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let mut image_fetched = false;
        let mut promise_loading = false;
        let mut trigger_fetch = false;

        if let Some(promise) = &self.promise {
            // Extract and store the image in the state if the request was successfull
            if let Some(result) = promise.ready() {
                match result {
                    Ok(Some(response)) => {
                        if let Some(image) = image::load_image_bytes(&response.bytes) {
                            self.image = image
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }

                // We reset the promise to avoid recomputing
                self.promise = Default::default();

                image_fetched = true; // Notify frame update
            } else {
                promise_loading = true; // Notify loading
            }
        }

        if promise_loading {
            self.ui_loading(ui);
        } else {
            trigger_fetch = self.ui_url(ui);
        }

        if trigger_fetch {
            let (sender, promise): (HTTPSender, HTTPPromise) = Promise::new();

            let request = ehttp::Request::get(&self.url);

            // Fetch the content from the given url
            ehttp::fetch(request, move |response: ehttp::Result<ehttp::Response>| {
                let result = response.map(|response| -> Option<ehttp::Response> {
                    let content_type = response.content_type().unwrap_or_default();

                    // Keep the content only if it's of image type
                    if content_type.starts_with("image/") {
                        Some(response)
                    } else {
                        None
                    }
                });

                sender.send(result);
            });

            // We then store the promise in the cache
            self.promise = Some(promise);
        }

        ui.separator();

        image_fetched
    }

    pub fn ui_loading(&mut self, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(228.0, 1.0),
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.spinner();
            },
        );
    }

    pub fn ui_url(&mut self, ui: &mut egui::Ui) -> bool {
        let mut trigger_fetch = false;

        ui.horizontal(|ui| {
            ui.label("URL:");
            trigger_fetch |= ui
                .add(egui::TextEdit::singleline(&mut self.url).desired_width(f32::INFINITY))
                .lost_focus();
            if ui.button("🔃").clicked() {
                trigger_fetch = true;
            }

            if ui.button("🎲").clicked() {
                let seed = ui.input().time;
                let side = 640;
                self.url = format!("https://picsum.photos/seed/{}/{}", seed, side);
                trigger_fetch = true;
            }
        });

        trigger_fetch
    }
}
