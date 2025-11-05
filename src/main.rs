use eframe::egui;
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

    // Communication channel for async requests
    tx: Sender<HttpResponse>,
    rx: Receiver<HttpResponse>,
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
    Body,
    Headers,
}

struct HttpResponse {
    status: String,
    headers: String,
    body: String,
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
            active_request_tab: RequestTab::Body,
            active_response_tab: ResponseTab::Body,
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
        "CrabPie"
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

        // Check for response from background thread
        if let Ok(resp) = self.rx.try_recv() {
            self.response_status = resp.status;
            self.response_headers = resp.headers;
            self.response_body = resp.body;
            self.loading = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CrabPie HTTP Client");
            ui.add_space(10.0);

            // Request section
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Method dropdown
                    egui::ComboBox::from_id_salt("method")
                        .selected_text(format!("{:?}", self.method))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.method, HttpMethod::GET, "GET");
                            ui.selectable_value(&mut self.method, HttpMethod::POST, "POST");
                            ui.selectable_value(&mut self.method, HttpMethod::PUT, "PUT");
                            ui.selectable_value(&mut self.method, HttpMethod::DELETE, "DELETE");
                            ui.selectable_value(&mut self.method, HttpMethod::PATCH, "PATCH");
                        });

                    // URL input - take all remaining space except for Send button
                    let available_width = ui.available_width() - 55.0;
                    ui.add(
                        egui::TextEdit::singleline(&mut self.url).desired_width(available_width),
                    );

                    // Send button
                    if ui
                        .add_enabled(!self.loading, egui::Button::new("Send"))
                        .clicked()
                    {
                        self.send_request();
                    }
                });
            });

            ui.add_space(10.0);

            // Request tabs - show for all methods now
            ui.horizontal(|ui| {
                // Only show Body tab for methods that can have a body
                if matches!(
                    self.method,
                    HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
                ) {
                    ui.selectable_value(&mut self.active_request_tab, RequestTab::Body, "Body");
                }
                ui.selectable_value(&mut self.active_request_tab, RequestTab::Headers, "Headers");
                ui.selectable_value(&mut self.active_request_tab, RequestTab::Auth, "Auth"); // Add this
            });

            ui.separator();

            // Request body/headers/auth editor
            egui::ScrollArea::vertical()
                .id_salt("request_scroll")
                .max_height(200.0)
                .show(ui, |ui| match self.active_request_tab {
                    RequestTab::Body => {
                        if matches!(
                            self.method,
                            HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
                        ) {
                            // Content type selector and prettify button
                            ui.horizontal(|ui| {
                                ui.label("Body");

                                // Content type dropdown
                                egui::ComboBox::from_id_salt("content_type")
                                    .selected_text(match self.content_type {
                                        ContentType::Json => "JSON",
                                        ContentType::FormData => "Form Data",
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
                                            if ui.button("✨ Prettify JSON").clicked() {
                                                self.prettify_json();
                                            }
                                        }
                                    },
                                );
                            });

                            ui.add_space(5.0);

                            // Show different UI based on content type
                            match self.content_type {
                                ContentType::Json => {
                                    egui::TextEdit::multiline(&mut self.body)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(8)
                                        .code_editor()
                                        .show(ui);
                                }
                                ContentType::FormData => {
                                    // Form data key-value pairs with file support
                                    let mut to_remove = None;

                                    for (i, field) in self.form_data.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label("Key:");
                                            ui.add(
                                                egui::TextEdit::singleline(&mut field.key)
                                                    .desired_width(120.0),
                                            );

                                            // Type selector
                                            egui::ComboBox::from_id_salt(format!(
                                                "field_type_{}",
                                                i
                                            ))
                                            .selected_text(match field.field_type {
                                                FormFieldType::Text => "Text",
                                                FormFieldType::File => "File",
                                            })
                                            .width(60.0)
                                            .show_ui(
                                                ui,
                                                |ui| {
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
                                                },
                                            );

                                            match field.field_type {
                                                FormFieldType::Text => {
                                                    ui.label("Value:");
                                                    ui.add(
                                                        egui::TextEdit::singleline(
                                                            &mut field.value,
                                                        )
                                                        .desired_width(150.0),
                                                    );
                                                }
                                                FormFieldType::File => {
                                                    ui.label("File:");
                                                    if ui.button("📁 Choose Files").clicked() {
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

                                        // Show selected files
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
                                    }

                                    // Remove field if requested
                                    if let Some(i) = to_remove {
                                        self.form_data.remove(i);
                                    }

                                    // Add new field button
                                    if ui.button("➕ Add Field").clicked() {
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
                    }
                    RequestTab::Headers => {
                        egui::TextEdit::multiline(&mut self.headers)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                            .code_editor()
                            .show(ui);
                    }
                    RequestTab::Auth => {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("auth_type")
                                .selected_text(match self.auth_type {
                                    AuthType::None => "No Auth",
                                    AuthType::Bearer => "Bearer Token",
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
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label("Token:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.bearer_token)
                                        .desired_width(f32::INFINITY)
                                        .password(false),
                                );
                            });
                        }
                    }
                });
            ui.add_space(10.0);

            // Response section label
            ui.label(egui::RichText::new("Response").strong());

            // Response tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_response_tab, ResponseTab::Body, "Body");
                ui.selectable_value(
                    &mut self.active_response_tab,
                    ResponseTab::Headers,
                    "Headers",
                );

                if !self.response_status.is_empty() {
                    ui.separator();
                    ui.label(&self.response_status);
                }

                if self.loading {
                    ui.spinner();
                }
            });

            ui.separator();

            // Response display
            egui::ScrollArea::vertical()
                .id_salt("response_scroll")
                .show(ui, |ui| match self.active_response_tab {
                    ResponseTab::Body => {
                        egui::TextEdit::multiline(&mut self.response_body.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .code_editor()
                            .show(ui);
                    }
                    ResponseTab::Headers => {
                        egui::TextEdit::multiline(&mut self.response_headers.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .code_editor()
                            .show(ui);
                    }
                });
        });

        // Keep repainting while loading
        if self.loading {
            ctx.request_repaint();
        }
    }
}
