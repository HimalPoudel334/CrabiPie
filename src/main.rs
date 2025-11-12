#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use egui::IconData;
use egui_extras::{Size, StripBuilder};
use std::sync::mpsc::{self, Receiver, Sender};

const CRABIPIE_ICON_BASE64: &str = "place a base64 encoded png string here";

#[derive(PartialEq)]
enum RequestTab {
    Body,
    Headers,
    Auth,
}

#[derive(PartialEq, Clone)]
enum ContentType {
    Json,
    FormData,
}

#[derive(Clone, PartialEq)]
enum FormFieldType {
    Text,
    File,
}

#[derive(Clone)]
struct FormField {
    key: String,
    value: String,
    files: Vec<String>,
    field_type: FormFieldType,
}

#[derive(PartialEq, Clone)]
enum AuthType {
    None,
    Bearer,
}

#[derive(PartialEq, Clone)]
enum LayoutMode {
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Clone, Debug)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(PartialEq)]
enum ResponseTab {
    None,
    Body,
    Headers,
}

struct HttpResponse {
    status: String,
    headers: String,
    body: String,
    is_binary: bool,
    filename: String,
    bytes: Vec<u8>,
    content_type: String,
}

struct MyApp {
    // Request configuration
    url: String,
    method: HttpMethod,
    headers: String,
    body: String,
    auth_type: AuthType,
    bearer_token: String,
    content_type: ContentType,
    form_data: Vec<FormField>,

    // Response data
    response_status: String,
    response_headers: String,
    response_body: String,
    is_response_binary: bool,
    response_filename: String,
    response_bytes: Vec<u8>,
    response_content_type: String,

    // UI state
    loading: bool,
    active_request_tab: RequestTab,
    active_response_tab: ResponseTab,
    layout_mode: LayoutMode,

    // Communication channel for async requests
    tx: Sender<HttpResponse>,
    rx: Receiver<HttpResponse>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            url: "https://jsonplaceholder.typicode.com/posts".to_string(),
            method: HttpMethod::GET,
            headers: "# Add headers as key: value pairs\n# Example:\n# X-Custom-Header: value"
                .to_string(),
            body: r#"{
  "title": "foo",
  "body": "bar",
  "userId": 1
}"#
            .to_string(),
            response_status: String::new(),
            response_headers: String::new(),
            response_body: String::new(),
            is_response_binary: false,
            response_filename: String::new(),
            response_bytes: Vec::new(),
            response_content_type: String::new(),
            loading: false,
            layout_mode: LayoutMode::Horizontal,
            active_request_tab: RequestTab::Body,
            active_response_tab: ResponseTab::None,
            auth_type: AuthType::None,
            bearer_token: String::new(),
            content_type: ContentType::Json,
            form_data: vec![FormField {
                key: String::new(),
                value: String::new(),
                files: Vec::new(),
                field_type: FormFieldType::Text,
            }],
            tx,
            rx,
        }
    }
}
impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::default()
    }

    fn name() -> &'static str {
        "CrabiPie"
    }

    fn prettify_json(&mut self) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.body) {
            if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                self.body = pretty;
            }
        }
    }

    fn parse_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        for line in self.headers.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                if let (Ok(header_name), Ok(header_value)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(value),
                ) {
                    headers.insert(header_name, header_value);
                }
            }
        }

        headers
    }

    fn render_request_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.expand_to_include_rect(ui.max_rect());
                ui.strong("Request");
                ui.add_space(6.0);

                // Tabs
                ui.horizontal(|ui| {
                    if matches!(
                        self.method,
                        HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
                    ) {
                        ui.selectable_value(&mut self.active_request_tab, RequestTab::Body, "Body");
                    }
                    ui.selectable_value(
                        &mut self.active_request_tab,
                        RequestTab::Headers,
                        "Headers",
                    );
                    ui.selectable_value(&mut self.active_request_tab, RequestTab::Auth, "Auth");
                });

                ui.separator();
                ui.add_space(4.0);

                match self.active_request_tab {
                    RequestTab::Body => {
                        if !matches!(
                            self.method,
                            HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
                        ) {
                            ui.label("Select POST, PUT, or PATCH to edit body.");
                            return;
                        }

                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("content_type")
                                .selected_text(if self.content_type == ContentType::Json {
                                    "JSON"
                                } else {
                                    "Form Data"
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.content_type,
                                        ContentType::Json,
                                        "JSON",
                                    );
                                    ui.selectable_value(
                                        &mut self.content_type,
                                        ContentType::FormData,
                                        "Form Data",
                                    );
                                });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if self.content_type == ContentType::Json {
                                        if ui.button("Prettify").clicked() {
                                            self.prettify_json();
                                        }
                                    }
                                },
                            );
                        });
                        ui.add_space(6.0);

                        egui::ScrollArea::vertical()
                            .id_salt("request_scroll")
                            .show(ui, |ui| match self.content_type {
                                ContentType::Json => {
                                    let line_height =
                                        ui.text_style_height(&egui::TextStyle::Monospace);
                                    let rows =
                                        (ui.available_height() / line_height).max(1.0) as usize;

                                    ui.expand_to_include_rect(ui.max_rect());

                                    egui::TextEdit::multiline(&mut self.body)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(rows)
                                        .show(ui);
                                }
                                ContentType::FormData => {
                                    // Limit how wide this section can expand to avoid pushing other panels
                                    ui.set_max_width(ui.available_width());

                                    // Make it scrollable so tall forms don’t push other layouts
                                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(
                                        ui,
                                        |ui| {
                                            let mut to_remove = None;

                                            for (i, field) in self.form_data.iter_mut().enumerate()
                                            {
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label("Key:");
                                                    ui.add(
                                                        egui::TextEdit::singleline(&mut field.key)
                                                            .hint_text("key")
                                                            .desired_width(
                                                                ui.available_width() * 0.3,
                                                            ),
                                                    );

                                                    egui::ComboBox::from_id_salt(format!(
                                                        "field_type_{}",
                                                        i
                                                    ))
                                                    .selected_text(match field.field_type {
                                                        FormFieldType::Text => "Text",
                                                        FormFieldType::File => "File",
                                                    })
                                                    .width(40.0)
                                                    .show_ui(ui, |ui| {
                                                        if ui
                                                            .selectable_value(
                                                                &mut field.field_type,
                                                                FormFieldType::Text,
                                                                "Text",
                                                            )
                                                            .clicked()
                                                        {
                                                            field.value.clear();
                                                            field.files.clear();
                                                        }
                                                        if ui
                                                            .selectable_value(
                                                                &mut field.field_type,
                                                                FormFieldType::File,
                                                                "File",
                                                            )
                                                            .clicked()
                                                        {
                                                            field.value.clear();
                                                            field.files.clear();
                                                        }
                                                    });

                                                    match field.field_type {
                                                        FormFieldType::Text => {
                                                            ui.label("Value:");
                                                            ui.add(
                                                                egui::TextEdit::singleline(
                                                                    &mut field.value,
                                                                )
                                                                .hint_text("value")
                                                                .desired_width(
                                                                    ui.available_width() * 0.4,
                                                                ),
                                                            );
                                                        }
                                                        FormFieldType::File => {
                                                            ui.label("File:");
                                                            if ui.button("📁 Choose").clicked() {
                                                                if let Some(paths) =
                                                                    rfd::FileDialog::new()
                                                                        .pick_files()
                                                                {
                                                                    field.files = paths
                                                                        .into_iter()
                                                                        .map(|p| {
                                                                            p.display().to_string()
                                                                        })
                                                                        .collect();
                                                                }
                                                            }
                                                            if !field.files.is_empty() {
                                                                ui.label(format!(
                                                                    "📎 {} file(s)",
                                                                    field.files.len()
                                                                ));
                                                            }
                                                        }
                                                    }

                                                    if ui.button("❌").clicked() {
                                                        to_remove = Some(i);
                                                    }
                                                });

                                                // Show selected files (if any)
                                                if field.field_type == FormFieldType::File
                                                    && !field.files.is_empty()
                                                {
                                                    ui.indent(format!("files_{}", i), |ui| {
                                                        for file in &field.files {
                                                            ui.label(format!(
                                                                "  • {}",
                                                                std::path::Path::new(file)
                                                                    .file_name()
                                                                    .and_then(|n| n.to_str())
                                                                    .unwrap_or(file)
                                                            ));
                                                        }
                                                    });
                                                }

                                                ui.add_space(4.0);
                                                ui.separator();
                                                ui.add_space(4.0);
                                            }

                                            // Remove field if requested
                                            if let Some(i) = to_remove {
                                                self.form_data.remove(i);
                                            }

                                            ui.add_space(6.0);

                                            // Add new field button
                                            if ui.button("➕ Add Field").clicked() {
                                                self.form_data.push(FormField {
                                                    key: String::new(),
                                                    value: String::new(),
                                                    files: Vec::new(),
                                                    field_type: FormFieldType::Text,
                                                });
                                            }
                                        },
                                    );
                                }
                            });
                    }
                    RequestTab::Headers => {
                        let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
                        let rows = (ui.available_height() / line_height).max(1.0) as usize;

                        ui.expand_to_include_rect(ui.max_rect());

                        egui::TextEdit::multiline(&mut self.headers)
                            .code_editor()
                            .hint_text("# Key: Value\n# Content-Type: application/json")
                            .desired_width(f32::INFINITY)
                            .desired_rows(rows)
                            .show(ui);
                    }
                    RequestTab::Auth => {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("auth_type")
                                .selected_text(if self.auth_type == AuthType::None {
                                    "No Auth"
                                } else {
                                    "Bearer Token"
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.auth_type,
                                        AuthType::None,
                                        "No Auth",
                                    );
                                    ui.selectable_value(
                                        &mut self.auth_type,
                                        AuthType::Bearer,
                                        "Bearer Token",
                                    );
                                });
                        });

                        if self.auth_type == AuthType::Bearer {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Token:").size(18.0));
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::TextEdit::singleline(&mut self.bearer_token)
                                        .min_size(egui::vec2(0.0, 30.0))
                                        .vertical_align(egui::Align::Center),
                                );
                            });
                        }
                    }
                }
            });
    }

    fn render_response_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.expand_to_include_rect(ui.max_rect());
                ui.horizontal(|ui| {
                    ui.strong("Response");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.loading {
                            ui.spinner();
                        }
                        if !self.response_status.is_empty() {
                            ui.label(&self.response_status);
                        }
                    });
                });
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_response_tab, ResponseTab::Body, "Body");
                    ui.selectable_value(
                        &mut self.active_response_tab,
                        ResponseTab::Headers,
                        "Headers",
                    );
                });
                ui.separator();
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .id_salt("response_scroll")
                    .show(ui, |ui| {
                        if self.active_response_tab == ResponseTab::None {
                            return;
                        }

                        if self.active_response_tab == ResponseTab::Body && self.is_response_binary
                        {
                            if !self.response_bytes.is_empty() {
                                if self.response_content_type.starts_with("image/") {
                                    ui.image(egui::ImageSource::Bytes {
                                        uri: format!("bytes://{}", self.response_filename).into(),
                                        bytes: egui::load::Bytes::from(self.response_bytes.clone()),
                                    });
                                } else {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 165, 0),
                                        format!(
                                            "📄 Binary file received: {}",
                                            self.response_filename
                                        ),
                                    );
                                    ui.label(&self.response_body);
                                    ui.add_space(8.0);

                                    if ui.button("💾 Save and Open").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_file_name(&self.response_filename)
                                            .save_file()
                                        {
                                            if std::fs::write(&path, &self.response_bytes).is_ok() {
                                                let _ = opener::open(&path);
                                            }
                                        }
                                    }
                                }
                            }
                            return;
                        }

                        let text = match self.active_response_tab {
                            ResponseTab::Body => &self.response_body,
                            ResponseTab::Headers => &self.response_headers,
                            ResponseTab::None => return,
                        };

                        let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
                        let rows = (ui.available_height() / line_height).max(1.0) as usize;
                        ui.expand_to_include_rect(ui.max_rect());
                        ui.add(
                            egui::TextEdit::multiline(&mut text.as_str())
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(rows),
                        );
                    });
            });
    }

    // Update send_request function
    fn send_request(&mut self) {
        self.loading = true;
        self.response_body = "Loading...".to_string();
        self.response_status = String::new();

        let url = self.url.clone();
        let method = self.method.clone();
        let body = self.body.clone();
        let mut headers = self.parse_headers();
        let auth_type = self.auth_type.clone();
        let bearer_token = self.bearer_token.clone();
        let content_type = self.content_type.clone();
        let form_data = self.form_data.clone();
        let tx = self.tx.clone();

        // Add Bearer token to headers if set
        if auth_type == AuthType::Bearer && !bearer_token.is_empty() {
            if let Ok(header_value) =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", bearer_token))
            {
                headers.insert(reqwest::header::AUTHORIZATION, header_value);
            }
        }

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let response = rt.block_on(async {
                let client = reqwest::Client::new();

                let mut request = match method {
                    HttpMethod::GET => client.get(&url),
                    HttpMethod::POST => {
                        let req = client.post(&url);
                        match content_type {
                            ContentType::Json => {
                                req.body(body).header("Content-Type", "application/json")
                            }
                            ContentType::FormData => {
                                let mut form = reqwest::multipart::Form::new();
                                for field in form_data {
                                    if !field.key.is_empty() {
                                        match field.field_type {
                                            FormFieldType::Text => {
                                                form = form.text(field.key, field.value);
                                            }
                                            FormFieldType::File => {
                                                if !field.value.is_empty() {
                                                    if let Ok(file_content) =
                                                        std::fs::read(&field.value)
                                                    {
                                                        let filename =
                                                            std::path::Path::new(&field.value)
                                                                .file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or("file")
                                                                .to_string();

                                                        let part = reqwest::multipart::Part::bytes(
                                                            file_content,
                                                        )
                                                        .file_name(filename);
                                                        form = form.part(field.key, part);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                req.multipart(form)
                            }
                        }
                    }
                    HttpMethod::PUT => {
                        let req = client.put(&url);
                        match content_type {
                            ContentType::Json => {
                                req.body(body).header("Content-Type", "application/json")
                            }
                            ContentType::FormData => {
                                let mut form = reqwest::multipart::Form::new();
                                for field in form_data {
                                    if !field.key.is_empty() {
                                        match field.field_type {
                                            FormFieldType::Text => {
                                                form = form.text(field.key, field.value);
                                            }
                                            FormFieldType::File => {
                                                if !field.value.is_empty() {
                                                    if let Ok(file_content) =
                                                        std::fs::read(&field.value)
                                                    {
                                                        let filename =
                                                            std::path::Path::new(&field.value)
                                                                .file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or("file")
                                                                .to_string();

                                                        let part = reqwest::multipart::Part::bytes(
                                                            file_content,
                                                        )
                                                        .file_name(filename);
                                                        form = form.part(field.key, part);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                req.multipart(form)
                            }
                        }
                    }
                    HttpMethod::DELETE => client.delete(&url),
                    HttpMethod::PATCH => {
                        let req = client.patch(&url);
                        match content_type {
                            ContentType::Json => {
                                req.body(body).header("Content-Type", "application/json")
                            }
                            ContentType::FormData => {
                                let mut form = reqwest::multipart::Form::new();
                                for field in form_data {
                                    if !field.key.is_empty() {
                                        match field.field_type {
                                            FormFieldType::Text => {
                                                form = form.text(field.key, field.value);
                                            }
                                            FormFieldType::File => {
                                                if !field.value.is_empty() {
                                                    if let Ok(file_content) =
                                                        std::fs::read(&field.value)
                                                    {
                                                        let filename =
                                                            std::path::Path::new(&field.value)
                                                                .file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or("file")
                                                                .to_string();

                                                        let part = reqwest::multipart::Part::bytes(
                                                            file_content,
                                                        )
                                                        .file_name(filename);
                                                        form = form.part(field.key, part);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                req.multipart(form)
                            }
                        }
                    }
                };

                // Add custom headers
                request = request.headers(headers);

                match request.send().await {
                    Ok(resp) => {
                        let status = format!(
                            "{} {}",
                            resp.status().as_u16(),
                            resp.status().canonical_reason().unwrap_or("")
                        );
                        let headers_map = resp.headers().clone();
                        let headers = format!("{:#?}", headers_map);

                        // Detect content type
                        let content_type = headers_map
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("")
                            .to_string();

                        let is_binary = content_type.starts_with("image/")
                            || content_type.starts_with("application/pdf")
                            || content_type.starts_with("application/octet-stream")
                            || content_type.starts_with("video/")
                            || content_type.starts_with("audio/");

                        // Extract filename from Content-Disposition or URL
                        let filename = headers_map
                            .get("content-disposition")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|s| {
                                s.split("filename=")
                                    .nth(1)
                                    .map(|f| f.trim_matches(|c| c == '"' || c == '\'').to_string())
                            })
                            .unwrap_or_else(|| {
                                url.split('/').last().unwrap_or("download").to_string()
                            });

                        let (body, bytes) = if is_binary {
                            match resp.bytes().await {
                                Ok(bytes) => {
                                    let body = format!(
                                        "Binary file ({} bytes)\n\nContent-Type: {}",
                                        bytes.len(),
                                        content_type
                                    );
                                    (body, bytes.to_vec())
                                }
                                Err(e) => (format!("Error reading binary data: {}", e), Vec::new()),
                            }
                        } else {
                            let body_text = resp
                                .text()
                                .await
                                .unwrap_or_else(|e| format!("Error reading body: {}", e));

                            // Try to pretty print JSON
                            let body = if let Ok(json) =
                                serde_json::from_str::<serde_json::Value>(&body_text)
                            {
                                serde_json::to_string_pretty(&json).unwrap_or(body_text)
                            } else {
                                body_text
                            };
                            (body, Vec::new())
                        };

                        HttpResponse {
                            status,
                            headers,
                            body,
                            is_binary,
                            filename,
                            bytes,
                            content_type,
                        }
                    }
                    Err(e) => HttpResponse {
                        status: "Error".to_string(),
                        headers: String::new(),
                        body: format!("Request failed: {}", e),
                        is_binary: false,
                        filename: String::new(),
                        bytes: Vec::new(),
                        content_type: String::new(),
                    },
                }
            });

            let _ = tx.send(response);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((1500.0, 900.0))
            .with_min_inner_size((285.0, 250.0))
            .with_icon(load_icon_from_base64()),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        MyApp::name(),
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for response
        if let Ok(resp) = self.rx.try_recv() {
            self.response_status = resp.status;
            self.response_headers = resp.headers;
            self.response_body = resp.body;
            self.is_response_binary = resp.is_binary;
            self.response_filename = resp.filename;
            self.response_bytes = resp.bytes;
            self.response_content_type = resp.content_type;
            self.loading = false;
            self.active_response_tab = ResponseTab::Body;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header: Title + Layout Toggle
            ui.horizontal(|ui| {
                ui.heading("CrabiPie HTTP Client");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let icon = if self.layout_mode == LayoutMode::Horizontal {
                        "Horizontal"
                    } else {
                        "Vertical"
                    };
                    if ui.button(icon).on_hover_text("Toggle Layout").clicked() {
                        self.layout_mode = match self.layout_mode {
                            LayoutMode::Horizontal => LayoutMode::Vertical,
                            LayoutMode::Vertical => LayoutMode::Horizontal,
                        };
                    }
                });
            });

            ui.add_space(8.0);

            // Request Method + URL + Send
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Set spacing to increase ComboBox button height
                    ui.style_mut().spacing.interact_size.y = 30.0;

                    // Method dropdown
                    egui::ComboBox::from_id_salt("method")
                        .selected_text(format!("{:?}", self.method))
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            for method in &[
                                HttpMethod::GET,
                                HttpMethod::POST,
                                HttpMethod::PUT,
                                HttpMethod::DELETE,
                                HttpMethod::PATCH,
                            ] {
                                ui.selectable_value(
                                    &mut self.method,
                                    method.clone(),
                                    format!("{:?}", method),
                                );
                            }
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Send button with proper minimum size
                        let send_button = ui.add_enabled(
                            !self.loading,
                            egui::Button::new("Send").min_size(egui::vec2(80.0, 30.0)),
                        );
                        // URL input - expands to fill available space
                        let url_response = ui.add(
                            egui::TextEdit::singleline(&mut self.url)
                                .desired_width(f32::INFINITY)
                                .min_size(egui::vec2(0.0, 30.0))
                                .hint_text(
                                    egui::RichText::new("https://api.example.com/endpoint")
                                        .size(18.0),
                                )
                                .vertical_align(egui::Align::Center)
                                .font(egui::FontId::proportional(18.0)),
                        );
                        if send_button.clicked()
                            || (url_response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                                && !self.url.is_empty()
                        {
                            self.send_request();
                        }
                    });
                });
            });

            ui.add_space(8.0);

            match self.layout_mode {
                LayoutMode::Horizontal => {
                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                self.render_request_section(ui);
                            });
                            strip.cell(|ui| {
                                self.render_response_section(ui);
                            });
                        });
                }
                LayoutMode::Vertical => {
                    self.render_request_section(ui);
                    ui.add_space(8.0);
                    self.render_response_section(ui);
                }
            }
        });

        // Keep repainting while loading
        if self.loading {
            ctx.request_repaint();
        }
    }
}

fn load_icon_from_base64() -> IconData {
    // Decode base64 string to bytes
    let icon_bytes = base64_decode(CRABIPIE_ICON_BASE64).expect("Failed to decode base64 icon");

    // Use image_crate feature from eframe to decode PNG
    let image = egui_extras::image::load_image_bytes(&icon_bytes).expect("Failed to load icon");

    IconData {
        rgba: image.as_raw().to_vec(),
        width: image.width() as u32,
        height: image.height() as u32,
    }
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input).ok()
}
