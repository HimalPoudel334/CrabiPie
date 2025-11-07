use eframe::egui;
use egui_extras::{Size, StripBuilder};
use std::sync::mpsc::{self, Receiver, Sender};

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
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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

                egui::ScrollArea::vertical()
                    .id_salt("request_scroll")
                    .show(ui, |ui| match self.active_request_tab {
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

                            match self.content_type {
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
                                    let mut to_remove = None;
                                    for (i, field) in self.form_data.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            let avail = ui.available_width();
                                            let key_w = (avail * 0.22).max(70.0);
                                            let type_w = 70.0;
                                            let value_w = (avail * 0.35).max(90.0);
                                            let btn_w = 32.0;

                                            ui.add(
                                                egui::TextEdit::singleline(&mut field.key)
                                                    .desired_width(key_w)
                                                    .hint_text("key"),
                                            );

                                            egui::ComboBox::from_id_salt(format!("type_{i}"))
                                                .selected_text(
                                                    if field.field_type == FormFieldType::Text {
                                                        "Text"
                                                    } else {
                                                        "File"
                                                    },
                                                )
                                                .width(type_w)
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
                                                    ui.add(
                                                        egui::TextEdit::singleline(
                                                            &mut field.value,
                                                        )
                                                        .desired_width(value_w)
                                                        .hint_text("value"),
                                                    );
                                                }
                                                FormFieldType::File => {
                                                    if ui
                                                        .add_sized(
                                                            [btn_w, 0.0],
                                                            egui::Button::new("Pick"),
                                                        )
                                                        .clicked()
                                                    {
                                                        if let Some(paths) =
                                                            rfd::FileDialog::new().pick_files()
                                                        {
                                                            field.files = paths
                                                                .into_iter()
                                                                .map(|p| p.display().to_string())
                                                                .collect();
                                                        }
                                                    }
                                                    if !field.files.is_empty() {
                                                        ui.label(format!(
                                                            "{} file{}",
                                                            field.files.len(),
                                                            if field.files.len() == 1 {
                                                                ""
                                                            } else {
                                                                "s"
                                                            }
                                                        ));
                                                    } else {
                                                        ui.label(
                                                            egui::RichText::new("No file")
                                                                .italics()
                                                                .weak(),
                                                        );
                                                    }
                                                }
                                            }

                                            if ui
                                                .add_sized(
                                                    [btn_w, 0.0],
                                                    egui::Button::new("Remove"),
                                                )
                                                .clicked()
                                            {
                                                to_remove = Some(i);
                                            }
                                        });

                                        if field.field_type == FormFieldType::File
                                            && !field.files.is_empty()
                                        {
                                            ui.indent(format!("files_{i}"), |ui| {
                                                for file in &field.files {
                                                    let name = std::path::Path::new(file)
                                                        .file_name()
                                                        .and_then(|n| n.to_str())
                                                        .unwrap_or(file);
                                                    ui.label(format!("• {name}"));
                                                }
                                            });
                                        }
                                        ui.add_space(2.0);
                                    }

                                    if let Some(i) = to_remove {
                                        self.form_data.remove(i);
                                    }

                                    if ui.button("Add Field").clicked() {
                                        self.form_data.push(FormField {
                                            key: String::new(),
                                            value: String::new(),
                                            files: Vec::new(),
                                            field_type: FormFieldType::Text,
                                        });
                                    }
                                }
                            }
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
                                    ui.label("Token:");
                                    ui.add_sized(
                                        ui.available_size(),
                                        egui::TextEdit::singleline(&mut self.bearer_token)
                                            .password(true),
                                    );
                                });
                            }
                        }
                    });
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
                        let headers = format!("{:#?}", resp.headers());
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

                        HttpResponse {
                            status,
                            headers,
                            body,
                        }
                    }
                    Err(e) => HttpResponse {
                        status: "Error".to_string(),
                        headers: String::new(),
                        body: format!("Request failed: {}", e),
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
            .with_inner_size((1200.0, 800.0))
            .with_min_inner_size((800.0, 500.0)),
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
        ctx.set_pixels_per_point(1.5);

        // Check for response
        if let Ok(resp) = self.rx.try_recv() {
            self.response_status = resp.status;
            self.response_headers = resp.headers;
            self.response_body = resp.body;
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
                            egui::Button::new("Send").min_size(egui::vec2(80.0, 0.0)),
                        );

                        // URL input - expands to fill available space
                        let url_response = ui.add(
                            egui::TextEdit::singleline(&mut self.url)
                                .desired_width(f32::INFINITY)
                                .hint_text("https://api.example.com/endpoint"),
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

            ui.add_space(12.0);

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
                    ui.add_space(12.0);
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
