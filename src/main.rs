use iced::widget::{Image, column, container, text};
use iced::{
    Application, Command, Element, Event, Length, Settings, Size, Subscription, event, executor,
};
use image::GenericImageView;
use std::path::PathBuf;

// Principal entry
pub fn main() -> iced::Result {
    ImageProcessor::run(Settings {
        window: iced::window::Settings {
            size: Size::new(400.0, 500.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

// Define App status
#[derive(Debug, Default)]
struct ImageProcessor {
    message: String,
    processed_image: Option<PathBuf>,
    is_processing: bool,
}

// Define Messages (Events)
#[derive(Debug, Clone)]
enum Message {
    FileDropped(PathBuf),
    ImageProcessed(Result<PathBuf, String>),
    EventOccurred(Event),
}

// General Logic
impl Application for ImageProcessor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                message: "Drag an image here".to_string(),
                processed_image: None,
                is_processing: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("CoverArt Converter for iPod")
    }

    // Listen OS events
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    // Manage messages
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) => {
                if let Event::Window(_id, iced::window::Event::FileDropped(path)) = event {
                    return self.handle_file_drop(path);
                }
                Command::none()
            }

            // Process message
            Message::FileDropped(path) => {
                self.is_processing = true;
                self.processed_image = None;
                self.message = "Processing...".to_string();

                Command::perform(process_image(path), Message::ImageProcessed)
            }

            // Finish message
            Message::ImageProcessed(Ok(path)) => {
                self.is_processing = false;
                self.message = "Image processed and saved".to_string();
                self.processed_image = Some(path);
                Command::none()
            }

            // Failure message
            Message::ImageProcessed(Err(error_message)) => {
                self.is_processing = false;
                self.message = format!("Error: {}", error_message);
                Command::none()
            }
        }
    }

    // Draw UI
    fn view(&self) -> Element<Message> {
        let mut content = column![text(&self.message).size(24),]
            .spacing(20)
            .align_items(iced::Alignment::Center);

        if let Some(path) = &self.processed_image {
            let image_handle = iced::widget::image::Handle::from_path(path.clone());

            content = content.push(
                Image::new(image_handle)
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(300.0))
                    .content_fit(iced::ContentFit::Contain),
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

// Auxiliar actions

impl ImageProcessor {
    fn handle_file_drop(&mut self, path: PathBuf) -> Command<Message> {
        if !self.is_processing {
            match path.extension().and_then(|s| s.to_str()) {
                Some("png") | Some("jpg") | Some("jpeg") | Some("bmp") | Some("webp") => {
                    return Command::perform(async { path }, Message::FileDropped);
                }
                _ => {
                    self.message = "Error: only images are supported".to_string();
                }
            }
        }
        Command::none()
    }
}

// IMAGE PROCESS
async fn process_image(path: PathBuf) -> Result<PathBuf, String> {
    // Load image into disk
    let img = match image::open(&path) {
        Ok(img) => img,
        Err(e) => return Err(format!("Image cannot be oppened: {}", e)),
    };

    // Apply redimension 
    let (width, height) = img.dimensions();

    let (target_width, target_height) = if width > 300 || height > 300 {
        (300, 300)
    } else if width <= 200 && height <= 200 {
        (width, height)
    } else {
        (200, 200)
    };

    let processed_img;
    if (width, height) == (target_width, target_height) {
        processed_img = img;
    } else {
        processed_img = img.resize_exact(
            target_width,
            target_height,
            image::imageops::FilterType::Lanczos3,
        );
    }

    // Prepare save path
    let original_stem = path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("image");

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("png");
    let new_filename = format!("{}_processed.{}", original_stem, extension);
    let new_path = path.with_file_name(new_filename);

    // Save new image
    match processed_img.save(&new_path) {
        Ok(_) => Ok(new_path),
        Err(e) => Err(format!("No se pudo guardar la imagen: {}", e)),
    }
}
